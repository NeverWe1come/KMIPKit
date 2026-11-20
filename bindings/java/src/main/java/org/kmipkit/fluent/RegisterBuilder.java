package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.OpaqueObjectContext;
import org.kmipkit.extensions.RegisterRequest;
import org.kmipkit.ttlv.ManagedObject;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent typed Register request retaining copies of the object and ordered attributes. */
public final class RegisterBuilder extends AttributeCollectionBuilder<RegisterBuilder> {
    private Long objectType;
    private TtlvItem object;
    private TtlvStructure protectionStorageMasks;
    private OpaqueObjectContext opaqueObjectContext;

    RegisterBuilder(KmipClient client) {
        super(client);
    }
    @Override
    protected RegisterBuilder self() {
        return this;
    }

    public RegisterBuilder object(ManagedObject value) {
        ensureOpen();
        if (value == null) throw new InvalidInputException("managed object is required");
        try (TtlvItemView view = value.view(); TtlvItem item = view.copy()) {
            return objectItem(value.objectType(), item);
        }
    }

    public RegisterBuilder objectItem(long type, TtlvItem value) {
        ensureOpen();
        unsigned32(type, "Object Type");
        if (value == null) throw new InvalidInputException("object is required");
        TtlvItem copy = value.copy();
        discard(object);
        object = own(copy);
        objectType = type;
        return this;
    }

    public RegisterBuilder protectionStorageMasks(TtlvStructure value) {
        protectionStorageMasks = replace(protectionStorageMasks, value);
        return this;
    }

    public RegisterBuilder opaqueObjectContext(OpaqueObjectContext value) {
        ensureOpen();
        if (value == null) throw new InvalidInputException("Opaque Object context is required");
        opaqueObjectContext = value;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (object == null) throw new InvalidInputException("Register object is required");
        try (TtlvStructure attributes = buildAttributes(); TtlvItem copy = object.copy()) {
            RegisterRequest request = RegisterRequest.create(objectType.intValue(), attributes, copy);
            try {
                if (opaqueObjectContext != null) request = request.withOpaqueObjectContext(opaqueObjectContext);
                if (protectionStorageMasks != null) request = request.withProtectionStorageMasks(protectionStorageMasks);
                return validated(ClientBatchItem.register(request));
            } finally {
                request.close();
            }
        }
    }
}
