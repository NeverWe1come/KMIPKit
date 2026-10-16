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
    && [ -s /output/validation/leaf.crt ]; then
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
