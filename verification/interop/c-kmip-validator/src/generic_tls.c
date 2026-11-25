#define _POSIX_C_SOURCE 200809L

#include "generic_tls.h"

#include <arpa/inet.h>
#include <errno.h>
#include <poll.h>
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
