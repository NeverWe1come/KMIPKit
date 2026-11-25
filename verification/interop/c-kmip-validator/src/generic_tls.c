#define _POSIX_C_SOURCE 200809L

#include "generic_tls.h"

#include <arpa/inet.h>
#include <errno.h>
#include <poll.h>
#include <fcntl.h>
#include <netdb.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include <openssl/ssl.h>
#include <openssl/x509v3.h>

static int64_t monotonic_ms(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) return -1;
    return (int64_t)now.tv_sec * 1000 + now.tv_nsec / 1000000;
}

static bool wait_until(int fd, short events, int64_t deadline) {
    if (fd < 0) return false;
    struct pollfd descriptor = {.fd = fd, .events = events};
    for (;;) {
        int64_t now = monotonic_ms();
        if (now < 0 || now >= deadline) return false;
        int result = poll(&descriptor, 1, (int)(deadline - now));
        if (result >= 0) return result == 1;
        if (errno != EINTR) return false;
    }
}

static bool wait_ssl(SSL *tls, int result, int64_t deadline) {
    int error = SSL_get_error(tls, result);
    if (error == SSL_ERROR_WANT_READ)
        return wait_until(SSL_get_fd(tls), POLLIN, deadline);
    if (error == SSL_ERROR_WANT_WRITE)
        return wait_until(SSL_get_fd(tls), POLLOUT, deadline);
    return false;
}

static bool read_exact(SSL *tls, uint8_t *buffer, size_t length, int64_t deadline) {
    while (length > 0) {
        size_t received = 0;
        int result = SSL_read_ex(tls, buffer, length, &received);
        if (result == 1 && received > 0) {
            buffer += received;
            length -= received;
        } else if (result == 1 || !wait_ssl(tls, result, deadline)) {
            return false;
        }
    }
    return true;
}

static bool set_peer_identity(SSL *tls, const char *name) {
    uint8_t address[16];
    if (inet_pton(AF_INET, name, address) == 1 || inet_pton(AF_INET6, name, address) == 1)
        return X509_VERIFY_PARAM_set1_ip_asc(SSL_get0_param(tls), name) == 1;

    X509_VERIFY_PARAM_set_hostflags(SSL_get0_param(tls), X509_CHECK_FLAG_NO_PARTIAL_WILDCARDS);
    return SSL_set1_host(tls, name) == 1 && SSL_set_tlsext_host_name(tls, name) == 1;
}

static int reject_encrypted_key(char *buffer, int size, int writing, void *context) {
    (void)buffer;
    (void)size;
    (void)writing;
    (void)context;
    return 0;
}

bool kmip_generic_tls_exchange(const kmip_connection_config_t *configuration,
                               const uint8_t *request, size_t request_size, size_t response_limit,
                               uint8_t **response, size_t *response_size) {
    *response = NULL;
    *response_size = 0;
    int64_t start = monotonic_ms();
    if (start < 0 || response_limit < 8) return false;

    /* Bound the complete connect, TLS handshake and one request/response exchange. */
    int64_t connect_deadline = start + 10000;
    int64_t io_deadline = start + 30000;
    SSL_CTX *context = SSL_CTX_new(TLS_client_method());
    SSL *tls = NULL;
    BIO *tcp = NULL;
    uint8_t *wire = NULL;
    bool valid = false;
    if (!context) goto cleanup;
    SSL_CTX_set_default_passwd_cb(context, reject_encrypted_key);
    if (SSL_CTX_set_min_proto_version(context, TLS1_3_VERSION) != 1 ||
        SSL_CTX_load_verify_locations(context, configuration->ca_cert_path, NULL) != 1 ||
        SSL_CTX_use_certificate_chain_file(context, configuration->client_cert_path) != 1 ||
        SSL_CTX_use_PrivateKey_file(context, configuration->client_key_path, SSL_FILETYPE_PEM) != 1 ||
        SSL_CTX_check_private_key(context) != 1) goto cleanup;
    SSL_CTX_set_verify(context, SSL_VERIFY_PEER, NULL);

    tcp = BIO_new(BIO_s_connect());
    char port[6];
    snprintf(port, sizeof(port), "%u", (unsigned)configuration->port);
    if (!tcp || BIO_set_conn_hostname(tcp, configuration->host) <= 0 ||
        BIO_set_conn_port(tcp, port) <= 0 || BIO_set_nbio(tcp, 1) <= 0) goto cleanup;
    while (BIO_do_connect(tcp) <= 0) {
        int fd = -1;
        if (!BIO_should_retry(tcp) || BIO_get_fd(tcp, &fd) < 0 ||
            !wait_until(fd, POLLOUT, connect_deadline)) goto cleanup;
    }

    tls = SSL_new(context);
    const char *peer = configuration->server_name_override[0] ?
        configuration->server_name_override : configuration->host;
    if (!tls || !set_peer_identity(tls, peer)) goto cleanup;
    SSL_set_bio(tls, tcp, tcp);
    tcp = NULL;
    int result;
    while ((result = SSL_connect(tls)) != 1) {
        if (!wait_ssl(tls, result, connect_deadline)) goto cleanup;
    }
    size_t sent = 0;
    while ((result = SSL_write_ex(tls, request, request_size, &sent)) != 1) {
        if (!wait_ssl(tls, result, io_deadline)) goto cleanup;
    }
    if (sent != request_size) goto cleanup;

    uint8_t header[8];
    if (!read_exact(tls, header, sizeof(header), io_deadline)) goto cleanup;
    uint64_t body = ((uint64_t)header[4] << 24) | ((uint64_t)header[5] << 16) |
                    ((uint64_t)header[6] << 8) | header[7];
    body = (body + 7) & ~UINT64_C(7);
    if (body > response_limit - sizeof(header)) goto cleanup;
    size_t size = sizeof(header) + (size_t)body;
    wire = malloc(size);
    if (!wire) goto cleanup;
    memcpy(wire, header, sizeof(header));
    if (!read_exact(tls, wire + sizeof(header), (size_t)body, io_deadline)) goto cleanup;
    *response = wire;
    *response_size = size;
    wire = NULL;
    valid = true;

cleanup:
    free(wire);
    SSL_free(tls);
    BIO_free(tcp);
    SSL_CTX_free(context);
    return valid;
}

struct kmip_generic_listener {
    SSL_CTX *context;
    int socket_fd;
    uint32_t timeout_seconds;
};

static bool parse_listener_address(const char *address, char host[INET6_ADDRSTRLEN], const char **port) {
    const char *separator = strrchr(address, ':');
    if (!separator || separator == address || !separator[1]) return false;
    *port = separator + 1;
    char *end = NULL;
    unsigned long value = strtoul(*port, &end, 10);
    if (*end || strspn(*port, "0123456789") != strlen(*port) || value > UINT16_MAX) return false;
    size_t length = (size_t)(separator - address);
    if (address[0] == '[' && length > 2 && address[length - 1] == ']') {
        ++address;
        length -= 2;
    }
    if (length >= INET6_ADDRSTRLEN) return false;
    memcpy(host, address, length);
    host[length] = '\0';
    return true;
}

static bool publish_listener(int fd, const char *path) {
    struct sockaddr_storage address;
    socklen_t length = sizeof(address);
    char host[INET6_ADDRSTRLEN], port[6];
    if (getsockname(fd, (struct sockaddr *)&address, &length) != 0 ||
        getnameinfo((struct sockaddr *)&address, length, host, sizeof(host), port, sizeof(port),
                    NI_NUMERICHOST | NI_NUMERICSERV) != 0) return false;
    FILE *file = fopen(path, "wx");
    if (!file) return false;
    bool written = fprintf(file, address.ss_family == AF_INET6 ? "[%s]:%s" : "%s:%s", host, port) > 0;
    return fclose(file) == 0 && written;
}

kmip_generic_listener_t *kmip_generic_tls_listen(
    const kmip_connection_config_t *configuration, const char *address, const char *certificate,
    const char *key, const char *ready_path) {
    char host[INET6_ADDRSTRLEN];
    const char *port = NULL;
    if (!parse_listener_address(address, host, &port)) return NULL;
    struct addrinfo hints = {.ai_family = AF_UNSPEC, .ai_socktype = SOCK_STREAM,
                            .ai_flags = AI_NUMERICHOST | AI_NUMERICSERV};
    struct addrinfo *socket_address = NULL;
    if (getaddrinfo(host, port, &hints, &socket_address) != 0) return NULL;
    int fd = socket(socket_address->ai_family, SOCK_STREAM, 0);
    bool listening = fd >= 0 && bind(fd, socket_address->ai_addr, socket_address->ai_addrlen) == 0 &&
        listen(fd, 8) == 0 && fcntl(fd, F_SETFL, O_NONBLOCK) != -1;
    freeaddrinfo(socket_address);

    SSL_CTX *context = SSL_CTX_new(TLS_server_method());
    if (context) SSL_CTX_set_default_passwd_cb(context, reject_encrypted_key);
    if (listening && context && SSL_CTX_set_min_proto_version(context, TLS1_3_VERSION) == 1 &&
        SSL_CTX_load_verify_locations(context, configuration->ca_cert_path, NULL) == 1 &&
        SSL_CTX_use_certificate_chain_file(context, certificate) == 1 &&
        SSL_CTX_use_PrivateKey_file(context, key, SSL_FILETYPE_PEM) == 1 &&
        SSL_CTX_check_private_key(context) == 1) {
        SSL_CTX_set_verify(context, SSL_VERIFY_PEER | SSL_VERIFY_FAIL_IF_NO_PEER_CERT, NULL);
        kmip_generic_listener_t *listener = malloc(sizeof(*listener));
        if (listener && (!ready_path || publish_listener(fd, ready_path))) {
            *listener = (kmip_generic_listener_t){
                .context = context, .socket_fd = fd, .timeout_seconds = configuration->read_timeout_seconds};

            return listener;
        }
        free(listener);
    }
    if (fd >= 0) close(fd);
    SSL_CTX_free(context);
    return NULL;
}

void kmip_generic_tls_listener_close(kmip_generic_listener_t *listener) {
    if (!listener) return;
    close(listener->socket_fd);
    SSL_CTX_free(listener->context);
    free(listener);
}

kmip_generic_reception_t kmip_generic_tls_receive(
    kmip_generic_listener_t *listener, size_t limit, kmip_generic_reply_fn reply, void *context) {
    int64_t start = monotonic_ms();
    if (start < 0 || limit < 8) return KMIP_GENERIC_INCOMING_FAILED;
    int64_t deadline = start + (int64_t)listener->timeout_seconds * 1000;
    if (!wait_until(listener->socket_fd, POLLIN, deadline)) return KMIP_GENERIC_PEER_UNAVAILABLE;
    int fd = accept(listener->socket_fd, NULL, NULL);
    if (fd < 0) return KMIP_GENERIC_INCOMING_FAILED;
    SSL *tls = SSL_new(listener->context);
    uint8_t *request = NULL, *response = NULL;
    size_t request_size = 0, response_size = 0;
    bool valid = false;
    if (!tls || fcntl(fd, F_SETFL, O_NONBLOCK) == -1 || SSL_set_fd(tls, fd) != 1) goto cleanup;
    int result;
    while ((result = SSL_accept(tls)) != 1) {
        if (!wait_ssl(tls, result, deadline)) goto cleanup;
    }
    uint8_t header[8];
    if (!read_exact(tls, header, sizeof(header), deadline)) goto cleanup;
    uint64_t padded = ((uint64_t)header[4] << 24) | ((uint64_t)header[5] << 16) |
                      ((uint64_t)header[6] << 8) | header[7];
    padded = (padded + 7) & ~UINT64_C(7);
    if (padded > limit - sizeof(header)) goto cleanup;
    request_size = sizeof(header) + (size_t)padded;
    request = malloc(request_size);
    if (!request) goto cleanup;
    memcpy(request, header, sizeof(header));
    if (!read_exact(tls, request + sizeof(header), (size_t)padded, deadline) ||
        !reply(request, request_size, &response, &response_size, context)) goto cleanup;
    size_t sent = 0;
    while ((result = SSL_write_ex(tls, response, response_size, &sent)) != 1) {
        if (!wait_ssl(tls, result, deadline)) goto cleanup;
    }
    valid = sent == response_size;
cleanup:
    if (request) OPENSSL_cleanse(request, request_size);
    if (response) OPENSSL_cleanse(response, response_size);
    free(request);
    free(response);
    SSL_free(tls);
    close(fd);
    return valid ? KMIP_GENERIC_REPLIED : KMIP_GENERIC_INCOMING_FAILED;
}
