package org.kmipkit.fluent;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;

/** Fluent builder for a typed KMIP Discover Versions request. */
public final class DiscoverVersionsBuilder extends TypedOperationBuilder<DiscoverVersionsBuilder> {
    private final List<ProtocolVersion> versions = new ArrayList<>();

    DiscoverVersionsBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected DiscoverVersionsBuilder self() { return this; }

    public DiscoverVersionsBuilder protocolVersion(ProtocolVersion version) {
        ensureOpen();
        versions.add(Objects.requireNonNull(version, "version"));
        return this;
    }

    public DiscoverVersionsBuilder protocolVersions(List<ProtocolVersion> protocolVersions) {
        ensureOpen();
        Objects.requireNonNull(protocolVersions, "protocolVersions");
        for (ProtocolVersion version : protocolVersions) {
            versions.add(Objects.requireNonNull(version, "version"));
        }
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        ClientBatchItem item = versions.isEmpty()
                ? ClientBatchItem.discoverVersions()
                : ClientBatchItem.discoverVersions(versions);
        return validated(item);
    }
}
