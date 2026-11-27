package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.UniqueIdentifier;

/** Explicit stateful multipart cryptographic streams ({@code Init -> Update* -> Final}). */
public final class StreamsClient {
    private final KmipClient client;

    public StreamsClient(KmipClient client) {
        if (client == null) throw new InvalidInputException("client is required");
        this.client = client;
    }

    public HashStream hash(HashAlgorithm algorithm) {
        return new HashStream(client, algorithm);
    }

    public EncryptStream encrypt(UniqueIdentifier id, CipherSpec spec) {
        return new EncryptStream(client, id, spec);
    }

    /** Stateful multipart Hash stream. */
    public static final class HashStream {
        private final KmipClient client;
        private final HashAlgorithm algorithm;
        private byte[] correlation;
        private boolean started;

        HashStream(KmipClient client, HashAlgorithm algorithm) {
            this.client = client;
            this.algorithm = algorithm;
        }

        public void update(byte[] chunk) throws org.kmipkit.ClientOperationException {
            try (HashBuilder b = new HashBuilder(client).withHashAlgorithm(algorithm).data(chunk)) {
                if (!started) {
                    b.initIndicator(true);
                    started = true;
                } else if (correlation != null) {
                    b.correlationValue(correlation);
                }
                try (ClientBatchResponse resp = b.execute(client.codecLimits());
                     ClientBatchItemResult item = resp.itemAt(0)) {
                    ClientBatchItemResult.HashResponse h = item.hashResponse().orElse(null);
                    if (h != null) {
                        correlation = h.correlationValue().orElse(correlation);
                    }
                }
            }
        }

        public byte[] finish() throws org.kmipkit.ClientOperationException {
            try (HashBuilder b = new HashBuilder(client).withHashAlgorithm(algorithm).finalIndicator(true)) {
                if (!started) b.initIndicator(true);
                else if (correlation != null) b.correlationValue(correlation);
                try (ClientBatchResponse resp = b.execute(client.codecLimits());
                     ClientBatchItemResult item = resp.itemAt(0)) {
                    return item.hashResponse()
                            .flatMap(ClientBatchItemResult.HashResponse::data)
                            .orElseThrow(() -> new InvalidInputException("Hash omitted Data"));
                }
            }
        }
    }

    /** Stateful multipart Encrypt stream. */
    public static final class EncryptStream {
        private final KmipClient client;
        private final UniqueIdentifier id;
        private final CipherSpec spec;
        private byte[] correlation;
        private boolean started;

        EncryptStream(KmipClient client, UniqueIdentifier id, CipherSpec spec) {
            this.client = client;
            this.id = id;
            this.spec = spec;
        }

        public byte[] update(byte[] chunk) throws org.kmipkit.ClientOperationException {
            try (EncryptBuilder b = new EncryptBuilder(client)
                    .uniqueIdentifier(id)
                    .withCipherSpec(spec)
                    .data(chunk)) {
                if (!started) {
                    b.initIndicator(true);
                    started = true;
                } else if (correlation != null) {
                    b.correlationValue(correlation);
                }
                try (ClientBatchResponse resp = b.execute(client.codecLimits());
                     ClientBatchItemResult item = resp.itemAt(0)) {
                    ClientBatchItemResult.EncryptResponse enc = item.encryptResponse().orElse(null);
                    if (enc == null) return new byte[0];
                    try (enc) {
                        correlation = enc.correlationValue().orElse(correlation);
                        return enc.data().orElse(new byte[0]);
                    }
                }
            }
        }

        public byte[] finish() throws org.kmipkit.ClientOperationException {
            try (EncryptBuilder b = new EncryptBuilder(client)
                    .uniqueIdentifier(id)
                    .withCipherSpec(spec)
                    .finalIndicator(true)) {
                if (!started) b.initIndicator(true);
                else if (correlation != null) b.correlationValue(correlation);
                try (ClientBatchResponse resp = b.execute(client.codecLimits());
                     ClientBatchItemResult item = resp.itemAt(0)) {
                    ClientBatchItemResult.EncryptResponse enc = item.encryptResponse().orElse(null);
                    if (enc == null) return new byte[0];
                    try (enc) {
                        return enc.data().orElse(new byte[0]);
                    }
                }
            }
        }
    }
}
