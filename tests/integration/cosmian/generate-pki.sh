#!/bin/sh
set -eu
umask 077

mkdir -p /output/ca /output/server /output/client /output/validation

if [ -s /output/ca/ca.crt ] \
    && [ -s /output/server/server.crt ] \
    && [ -s /output/server/server.key ] \
    && [ -s /output/client/client.crt ] \
    && [ -s /output/client/client.key ] \
    && [ -s /output/validation/intermediate.crt ] \
    && [ -s /output/validation/leaf.crt ] \
    && [ -s /output/validation/private-key-password-no-chain.p12 ] \
    && [ -s /output/validation/private-key-password-chain.p12 ] \
    && [ -s /output/validation/private-key-empty-password-no-chain.p12 ] \
    && [ -s /output/validation/private-key-empty-password-chain.p12 ] \
    && [ -s /output/validation/fixture-set-v2 ]; then
    exit 0
fi

apk add --no-cache openssl >/dev/null

rm -f /output/ca/* /output/server/* /output/client/* /output/validation/*

openssl req -x509 -newkey rsa:2048 -sha256 -nodes -days 30 \
    -keyout /output/ca/ca.key \
    -out /output/ca/ca.crt \
    -subj "/CN=KMIPKit local integration CA"

openssl req -new -newkey rsa:2048 -sha256 -nodes \
    -keyout /output/server/server.key \
    -out /output/server/server.csr \
    -subj "/CN=localhost"
openssl x509 -req -sha256 -days 30 \
    -in /output/server/server.csr \
    -CA /output/ca/ca.crt \
    -CAkey /output/ca/ca.key \
    -CAserial /output/ca/ca.srl \
    -CAcreateserial \
    -extfile /scripts/server.ext \
    -out /output/server/server.crt
cp /output/ca/ca.crt /output/server/clients-ca.crt
rm -f /output/server/server.csr

openssl req -new -newkey rsa:2048 -sha256 -nodes \
    -keyout /output/validation/intermediate.key \
    -out /output/validation/intermediate.csr \
    -subj "/CN=KMIPKit validation intermediate"
openssl x509 -req -sha256 -days 30 \
    -in /output/validation/intermediate.csr \
    -CA /output/ca/ca.crt \
    -CAkey /output/ca/ca.key \
    -CAserial /output/ca/ca.srl \
    -extfile /scripts/intermediate.ext \
    -out /output/validation/intermediate.crt
rm -f /output/validation/intermediate.csr

openssl req -new -newkey rsa:2048 -sha256 -nodes \
    -keyout /output/validation/leaf.key \
    -out /output/validation/leaf.csr \
    -subj "/CN=KMIPKit validation leaf"
openssl x509 -req -sha256 -days 30 \
    -in /output/validation/leaf.csr \
    -CA /output/validation/intermediate.crt \
    -CAkey /output/validation/intermediate.key \
    -CAcreateserial \
    -extfile /scripts/validation-leaf.ext \
    -out /output/validation/leaf.crt
rm -f /output/validation/leaf.csr

openssl req -new -newkey rsa:2048 -sha256 -nodes \
    -keyout /output/client/client.key \
    -out /output/client/client.csr \
    -subj "/CN=kmipkit-integration-client"
openssl x509 -req -sha256 -days 30 \
    -in /output/client/client.csr \
    -CA /output/ca/ca.crt \
    -CAkey /output/ca/ca.key \
    -CAserial /output/ca/ca.srl \
    -extfile /scripts/client.ext \
    -out /output/client/client.crt
cp /output/ca/ca.crt /output/client/ca.crt
rm -f /output/client/client.csr

# PrivateKey/PKCS#12 Import fixtures. These contain a test-only private key;
# the password variants are linked through a case-owned SecretData object.
openssl pkcs12 -export \
    -inkey /output/validation/leaf.key \
    -in /output/validation/leaf.crt \
    -out /output/validation/private-key-password-no-chain.p12 \
    -passout pass:kmipkit-test-password
openssl pkcs12 -export \
    -inkey /output/validation/leaf.key \
    -in /output/validation/leaf.crt \
    -certfile /output/validation/intermediate.crt \
    -out /output/validation/private-key-password-chain.p12 \
    -passout pass:kmipkit-test-password
openssl pkcs12 -export \
    -inkey /output/validation/leaf.key \
    -in /output/validation/leaf.crt \
    -out /output/validation/private-key-empty-password-no-chain.p12 \
    -passout pass:
openssl pkcs12 -export \
    -inkey /output/validation/leaf.key \
    -in /output/validation/leaf.crt \
    -certfile /output/validation/intermediate.crt \
    -out /output/validation/private-key-empty-password-chain.p12 \
    -passout pass:

# DER and PEM fixture inputs cover each transfer row's declared algorithm and
# key size/curve. KMIP requests always carry DER bytes where the format expects
# encoded key material; PEM names here describe only the source fixture.
generate_key_fixture() {
    name=$1
    algorithm=$2
    parameter=$3
    key=/output/validation/$name.key
    csr=/output/validation/$name.csr
    cert=/output/validation/$name.crt

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
            public_key=/output/validation/$name-public-key.pem
            openssl pkey -in "$key" -pubout -out "$public_key"
            openssl x509 -new -force_pubkey "$public_key" \
                -subj "/CN=KMIPKit $name fixture" -days 30 \
                -CA /output/validation/intermediate.crt \
                -CAkey /output/validation/intermediate.key \
                -CAserial /output/validation/ca.srl -CAcreateserial \
                -extfile /scripts/validation-xdh.ext -out "$cert"
            rm -f "$public_key"
            ;;
        DSA)
            openssl genpkey -genparam -algorithm DSA -pkeyopt "dsa_paramgen_bits:$parameter" \
                -out /output/validation/dsa-$parameter.params
            openssl genpkey -paramfile /output/validation/dsa-$parameter.params -out "$key"
            ;;
        *)
            return 1
            ;;
    esac

    case "$algorithm" in
        X25519|X448) ;;
        *)
            openssl req -new -key "$key" -out "$csr" -subj "/CN=KMIPKit $name fixture"
            openssl x509 -req -sha256 -days 30 -in "$csr" \
                -CA /output/validation/intermediate.crt \
                -CAkey /output/validation/intermediate.key \
                -CAcreateserial -extfile /scripts/validation-leaf.ext -out "$cert"
            rm -f "$csr"
            ;;
    esac

    openssl pkey -in "$key" -outform DER -out /output/validation/$name-private-pkcs8.der
    openssl pkey -in "$key" -pubout -outform DER -out /output/validation/$name-public-spki.der
    openssl x509 -in "$cert" -outform DER -out /output/validation/$name-certificate.der
    openssl pkey -in "$key" -pubout -out /output/validation/$name-public-spki.pem
    if [ "$algorithm" = RSA ]; then
        openssl rsa -in "$key" -traditional -outform DER -out /output/validation/$name-private-pkcs1.der
        openssl rsa -in "$key" -traditional -out /output/validation/$name-private-pkcs1.pem
        openssl rsa -in "$key" -RSAPublicKey_out -outform DER \
            -out /output/validation/$name-public-pkcs1.der
        openssl rsa -in "$key" -RSAPublicKey_out -out /output/validation/$name-public-pkcs1.pem
    elif [ "$algorithm" = EC ]; then
        openssl ec -in "$key" -outform DER -out /output/validation/$name-private-ec.der
        openssl ec -in "$key" -out /output/validation/$name-private-ec.pem
    fi

    case "$algorithm" in
        RSA|EC|ED25519|ED448|X25519|X448)
            for variant in password-no-chain password-chain empty-password-no-chain empty-password-chain; do
                case "$variant" in
                    password-no-chain) password=kmipkit-test-password; chain_args= ;;
                    password-chain) password=kmipkit-test-password; chain_args="-certfile /output/validation/intermediate.crt" ;;
                    empty-password-no-chain) password=; chain_args= ;;
                    empty-password-chain) password=; chain_args="-certfile /output/validation/intermediate.crt" ;;
                esac
                # chain_args contains only the fixed certificate path above.
                if [ -n "$chain_args" ]; then
                    openssl pkcs12 -export -inkey "$key" -in "$cert" $chain_args \
                        -out /output/validation/$name-$variant.p12 -passout "pass:$password"
                else
                    openssl pkcs12 -export -inkey "$key" -in "$cert" \
                        -out /output/validation/$name-$variant.p12 -passout "pass:$password"
                fi
            done
            ;;
    esac
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
touch /output/validation/fixture-set-v2
