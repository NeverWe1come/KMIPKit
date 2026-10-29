#!/bin/sh
set -eu
umask 077
output_dir="${KMIPKIT_PKI_OUTPUT_DIR:-/output}"
pki_marker="$output_dir/.kmipkit-cosmian-pki"
pki_marker_contents="KMIPKit Cosmian integration PKI v1"

mkdir -p "$output_dir"

refuse_unowned_output() {
    printf '%s\n' "Refusing to clean an unowned PKI output directory" >&2
    exit 1
}

if [ -L "$output_dir" ]; then
    refuse_unowned_output
fi

if [ -L "$pki_marker" ]; then
    refuse_unowned_output
elif [ -f "$pki_marker" ]; then
    if [ "$(cat "$pki_marker")" != "$pki_marker_contents" ]; then
        refuse_unowned_output
    fi
elif [ -e "$pki_marker" ]; then
    refuse_unowned_output
else
    for output_entry in "$output_dir"/* "$output_dir"/.[!.]* "$output_dir"/..?*; do
        if [ -e "$output_entry" ] || [ -L "$output_entry" ]; then
            refuse_unowned_output
        fi
    done
    printf '%s\n' "$pki_marker_contents" > "$pki_marker"
fi

for pki_subdirectory in ca server client validation; do
    pki_subdirectory_path="$output_dir/$pki_subdirectory"
    if [ -L "$pki_subdirectory_path" ]; then
        refuse_unowned_output
    fi
    if [ -e "$pki_subdirectory_path" ] && [ ! -d "$pki_subdirectory_path" ]; then
        refuse_unowned_output
    fi
done

if [ -s "$output_dir/ca/ca.crt" ] \
    && [ -s "$output_dir/server/server.crt" ] \
    && [ -s "$output_dir/server/server.key" ] \
    && [ -s "$output_dir/client/client.crt" ] \
    && [ -s "$output_dir/client/client.key" ] \
    && [ -s "$output_dir/validation/intermediate.crt" ] \
    && [ -s "$output_dir/validation/leaf.crt" ] \
    && [ -s "$output_dir/validation/private-key-password-no-chain.p12" ] \
    && [ -s "$output_dir/validation/private-key-password-chain.p12" ] \
    && [ -s "$output_dir/validation/private-key-empty-password-no-chain.p12" ] \
    && [ -s "$output_dir/validation/private-key-empty-password-chain.p12" ] \
    && [ -s "$output_dir/validation/rsa-2048-csr-pkcs10.der" ] \
    && [ -s "$output_dir/validation/rsa-2048-csr-pem.pem" ] \
    && [ -s "$output_dir/validation/ec-P256-csr-pkcs10.der" ] \
    && [ -s "$output_dir/validation/ed25519-csr-pkcs10.der" ] \
    && [ -s "$output_dir/validation/cryptographic-fixture-capabilities.txt" ] \
    && [ -s "$output_dir/validation/fixture-set-v3" ]; then
    exit 0
fi

mkdir -p "$output_dir/ca" "$output_dir/server" "$output_dir/client" "$output_dir/validation"
apk add --no-cache openssl >/dev/null

rm -f "$output_dir/ca/"* "$output_dir/server/"* "$output_dir/client/"* "$output_dir/validation/"*

openssl req -x509 -newkey rsa:2048 -sha256 -nodes -days 30 \
    -keyout "$output_dir/ca/ca.key" \
    -out "$output_dir/ca/ca.crt" \
    -subj "/CN=KMIPKit local integration CA"

openssl req -new -newkey rsa:2048 -sha256 -nodes \
    -keyout "$output_dir/server/server.key" \
    -out "$output_dir/server/server.csr" \
    -subj "/CN=localhost"
openssl x509 -req -sha256 -days 30 \
    -in "$output_dir/server/server.csr" \
    -CA "$output_dir/ca/ca.crt" \
    -CAkey "$output_dir/ca/ca.key" \
    -CAserial "$output_dir/ca/ca.srl" \
    -CAcreateserial \
    -extfile /scripts/server.ext \
    -out "$output_dir/server/server.crt"
cp "$output_dir/ca/ca.crt" "$output_dir/server/clients-ca.crt"
rm -f "$output_dir/server/server.csr"

openssl req -new -newkey rsa:2048 -sha256 -nodes \
    -keyout "$output_dir/validation/intermediate.key" \
    -out "$output_dir/validation/intermediate.csr" \
    -subj "/CN=KMIPKit validation intermediate"
openssl x509 -req -sha256 -days 30 \
    -in "$output_dir/validation/intermediate.csr" \
    -CA "$output_dir/ca/ca.crt" \
    -CAkey "$output_dir/ca/ca.key" \
    -CAserial "$output_dir/ca/ca.srl" \
    -extfile /scripts/intermediate.ext \
    -out "$output_dir/validation/intermediate.crt"
rm -f "$output_dir/validation/intermediate.csr"

openssl req -new -newkey rsa:2048 -sha256 -nodes \
    -keyout "$output_dir/validation/leaf.key" \
    -out "$output_dir/validation/leaf.csr" \
    -subj "/CN=KMIPKit validation leaf"
openssl x509 -req -sha256 -days 30 \
    -in "$output_dir/validation/leaf.csr" \
    -CA "$output_dir/validation/intermediate.crt" \
    -CAkey "$output_dir/validation/intermediate.key" \
    -CAcreateserial \
    -extfile /scripts/validation-leaf.ext \
    -out "$output_dir/validation/leaf.crt"
rm -f "$output_dir/validation/leaf.csr"

openssl req -new -newkey rsa:2048 -sha256 -nodes \
    -keyout "$output_dir/client/client.key" \
    -out "$output_dir/client/client.csr" \
    -subj "/CN=kmipkit-integration-client"
openssl x509 -req -sha256 -days 30 \
    -in "$output_dir/client/client.csr" \
    -CA "$output_dir/ca/ca.crt" \
    -CAkey "$output_dir/ca/ca.key" \
    -CAserial "$output_dir/ca/ca.srl" \
    -extfile /scripts/client.ext \
    -out "$output_dir/client/client.crt"
cp "$output_dir/ca/ca.crt" "$output_dir/client/ca.crt"
rm -f "$output_dir/client/client.csr"

# PrivateKey/PKCS#12 Import fixtures. These contain a test-only private key;
# the password variants are linked through a case-owned SecretData object.
openssl pkcs12 -export \
    -inkey "$output_dir/validation/leaf.key" \
    -in "$output_dir/validation/leaf.crt" \
    -out "$output_dir/validation/private-key-password-no-chain.p12" \
    -passout pass:kmipkit-test-password
openssl pkcs12 -export \
    -inkey "$output_dir/validation/leaf.key" \
    -in "$output_dir/validation/leaf.crt" \
    -certfile "$output_dir/validation/intermediate.crt" \
    -out "$output_dir/validation/private-key-password-chain.p12" \
    -passout pass:kmipkit-test-password
openssl pkcs12 -export \
    -inkey "$output_dir/validation/leaf.key" \
    -in "$output_dir/validation/leaf.crt" \
    -out "$output_dir/validation/private-key-empty-password-no-chain.p12" \
    -passout pass:
openssl pkcs12 -export \
    -inkey "$output_dir/validation/leaf.key" \
    -in "$output_dir/validation/leaf.crt" \
    -certfile "$output_dir/validation/intermediate.crt" \
    -out "$output_dir/validation/private-key-empty-password-chain.p12" \
    -passout pass:

# DER and PEM fixture inputs cover each transfer row's declared algorithm and
# key size/curve. KMIP requests always carry DER bytes where the format expects
# encoded key material; PEM names here describe only the source fixture.
generate_key_fixture() {
    name=$1
    algorithm=$2
    parameter=$3
    key="$output_dir/validation/$name.key"
    csr="$output_dir/validation/$name.csr"
    cert="$output_dir/validation/$name.crt"

    case "$algorithm" in
        RSA)
            openssl genpkey -algorithm RSA -pkeyopt "rsa_keygen_bits:$parameter" -out "$key"
            ;;
        EC)
            openssl genpkey -algorithm EC -pkeyopt "ec_paramgen_curve:$parameter" -out "$key"
            ;;
        ED25519|ED448)
            openssl genpkey -algorithm "$algorithm" -out "$key"
            ;;
        X25519|X448)
            openssl genpkey -algorithm "$algorithm" -out "$key"
            public_key="$output_dir/validation/$name-public-key.pem"
            openssl pkey -in "$key" -pubout -out "$public_key"
            openssl x509 -new -force_pubkey "$public_key" \
                -subj "/CN=KMIPKit $name fixture" -days 30 \
                -CA "$output_dir/validation/intermediate.crt" \
                -CAkey "$output_dir/validation/intermediate.key" \
                -CAserial "$output_dir/validation/ca.srl" -CAcreateserial \
                -extfile /scripts/validation-xdh.ext -out "$cert"
            rm -f "$public_key"
            ;;
        DSA)
            openssl genpkey -genparam -algorithm DSA -pkeyopt "dsa_paramgen_bits:$parameter" \
                -out "$output_dir/validation/dsa-$parameter.params"
            openssl genpkey -paramfile "$output_dir/validation/dsa-$parameter.params" -out "$key"
            ;;
        *)
            return 1
            ;;
    esac

    case "$algorithm" in
        X25519|X448) ;;
        *)
            openssl req -new -key "$key" -out "$csr" -subj "/CN=KMIPKit $name fixture"
            cp "$csr" "$output_dir/validation/$name-csr-pem.pem"
            openssl req -in "$csr" -outform DER -out "$output_dir/validation/$name-csr-pkcs10.der"
            openssl x509 -req -sha256 -days 30 -in "$csr" \
                -CA "$output_dir/validation/intermediate.crt" \
                -CAkey "$output_dir/validation/intermediate.key" \
                -CAcreateserial -extfile /scripts/validation-leaf.ext -out "$cert"
            rm -f "$csr"
            ;;
    esac

    openssl pkey -in "$key" -outform DER -out "$output_dir/validation/$name-private-pkcs8.der"
    openssl pkey -in "$key" -pubout -outform DER -out "$output_dir/validation/$name-public-spki.der"
    openssl x509 -in "$cert" -outform DER -out "$output_dir/validation/$name-certificate.der"
    openssl pkey -in "$key" -pubout -out "$output_dir/validation/$name-public-spki.pem"
    if [ "$algorithm" = RSA ]; then
        openssl rsa -in "$key" -traditional -outform DER -out "$output_dir/validation/$name-private-pkcs1.der"
        openssl rsa -in "$key" -traditional -out "$output_dir/validation/$name-private-pkcs1.pem"
        openssl rsa -in "$key" -RSAPublicKey_out -outform DER \
            -out "$output_dir/validation/$name-public-pkcs1.der"
        openssl rsa -in "$key" -RSAPublicKey_out -out "$output_dir/validation/$name-public-pkcs1.pem"
    elif [ "$algorithm" = EC ]; then
        openssl ec -in "$key" -outform DER -out "$output_dir/validation/$name-private-ec.der"
        openssl ec -in "$key" -out "$output_dir/validation/$name-private-ec.pem"
    fi

    case "$algorithm" in
        RSA|EC|ED25519|ED448|X25519|X448)
            for variant in password-no-chain password-chain empty-password-no-chain empty-password-chain; do
                case "$variant" in
                    password-no-chain) password=kmipkit-test-password; include_chain=no ;;
                    password-chain) password=kmipkit-test-password; include_chain=yes ;;
                    empty-password-no-chain) password=; include_chain=no ;;
                    empty-password-chain) password=; include_chain=yes ;;
                esac
                if [ "$include_chain" = yes ]; then
                    openssl pkcs12 -export -inkey "$key" -in "$cert" \
                        -certfile "$output_dir/validation/intermediate.crt" \
                        -out "$output_dir/validation/$name-$variant.p12" -passout "pass:$password"
                else
                    openssl pkcs12 -export -inkey "$key" -in "$cert" \
                        -out "$output_dir/validation/$name-$variant.p12" -passout "pass:$password"
                fi
            done
            ;;
    esac
}

# Some OpenSSL builds include ML-DSA/SLH-DSA and others do not. Record only
# capability names, never command output or generated key material. A missing
# fixture remains a setup limitation for the exact algorithm rows.
generate_post_quantum_csr_fixture() {
    fixture_name=$1
    openssl_algorithm=$2
    key="$output_dir/validation/$fixture_name-private.key"
    csr="$output_dir/validation/$fixture_name.csr"
    if openssl genpkey -algorithm "$openssl_algorithm" -out "$key" >/dev/null 2>&1 \
        && openssl req -new -key "$key" -out "$csr" \
            -subj "/CN=KMIPKit $fixture_name fixture" >/dev/null 2>&1; then
        cp "$csr" "$output_dir/validation/$fixture_name-csr-pem.pem"
        openssl req -in "$csr" -outform DER \
            -out "$output_dir/validation/$fixture_name-csr-pkcs10.der" >/dev/null 2>&1
        printf '%s=available\n' "$fixture_name" >> "$output_dir/validation/cryptographic-fixture-capabilities.txt"
    else
        printf '%s=unavailable\n' "$fixture_name" >> "$output_dir/validation/cryptographic-fixture-capabilities.txt"
    fi
    rm -f "$key" "$csr"
}

for bits in 2048 3072 4096; do
    generate_key_fixture rsa-$bits RSA "$bits"
done
for curve in P192 P224 P256 P384 P521; do
    case "$curve" in
        P192) openssl_curve=prime192v1 ;;
        P224) openssl_curve=secp224r1 ;;
        P256) openssl_curve=prime256v1 ;;
        P384) openssl_curve=secp384r1 ;;
        P521) openssl_curve=secp521r1 ;;
    esac
    generate_key_fixture ec-$curve EC "$openssl_curve"
done
for curve in SECP224K1 SECP256K1; do
    case "$curve" in
        SECP224K1) openssl_curve=secp224k1 ;;
        SECP256K1) openssl_curve=secp256k1 ;;
    esac
    generate_key_fixture ec-$curve EC "$openssl_curve"
done
generate_key_fixture ed25519 ED25519 ""
generate_key_fixture ed448 ED448 ""
generate_key_fixture x25519 X25519 ""
generate_key_fixture x448 X448 ""
for bits in 2048 3072; do
    generate_key_fixture dsa-$bits DSA "$bits"
done
for algorithm in ML-DSA-44 ML-DSA-65 ML-DSA-87; do
    generate_post_quantum_csr_fixture "${algorithm}" "$algorithm"
done
for algorithm in \
    SLH-DSA-SHA2-128f SLH-DSA-SHA2-128s \
    SLH-DSA-SHA2-192f SLH-DSA-SHA2-192s \
    SLH-DSA-SHA2-256f SLH-DSA-SHA2-256s \
    SLH-DSA-SHAKE-128f SLH-DSA-SHAKE-128s \
    SLH-DSA-SHAKE-192f SLH-DSA-SHAKE-192s \
    SLH-DSA-SHAKE-256f SLH-DSA-SHAKE-256s; do
    generate_post_quantum_csr_fixture "${algorithm}" "$algorithm"
done
touch "$output_dir/validation/fixture-set-v3"
