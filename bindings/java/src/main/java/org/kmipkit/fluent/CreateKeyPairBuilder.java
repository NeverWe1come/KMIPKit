package org.kmipkit.fluent;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TypedAttribute;

/** Builds a typed Create Key Pair request from explicit algorithm and usage choices. */
public final class CreateKeyPairBuilder {
    private final KmipClient client;
    private Long algorithm;
    private Integer length;
    private Integer privateUsageMask;
    private Integer publicUsageMask;
    private RequestOptions options;
    private Long asynchronousIndicator;
    private final List<TypedAttribute> commonAttributes = new ArrayList<>();

    CreateKeyPairBuilder(KmipClient client) {
        this.client = client;
    }

    public CreateKeyPairBuilder algorithm(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("Cryptographic Algorithm is outside uint32");
        }
        algorithm = value;
        return this;
    }

    public CreateKeyPairBuilder length(int bits) {
        length = bits;
        return this;
    }

    public CreateKeyPairBuilder privateUsageMask(int value) {
        privateUsageMask = value;
        return this;
    }

    public CreateKeyPairBuilder publicUsageMask(int value) {
        publicUsageMask = value;
        return this;
    }

    /** Adds a typed common attribute, borrowed until build or execute completes. */
    public CreateKeyPairBuilder commonAttribute(TypedAttribute value) {
        if (value == null) throw new InvalidInputException("common attribute is required");
        commonAttributes.add(value);
        return this;
    }

    public CreateKeyPairBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return this;
    }

    /** Sets the explicit Request Header Asynchronous Indicator on the one-item batch. */
    public CreateKeyPairBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        asynchronousIndicator = value;
        return this;
    }

    /** Builds the existing typed Create Key Pair batch item. */
    public ClientBatchItem build() {
        if (algorithm == null || length == null) {
            throw new InvalidInputException("Create Key Pair requires algorithm and length");
        }
        try (TtlvStructure common = commonAttributes();
             TtlvStructure privateAttributes = optionalUsage(privateUsageMask);
             TtlvStructure publicAttributes = optionalUsage(publicUsageMask)) {
            return ClientBatchItem.createKeyPair(
                    common, privateAttributes, publicAttributes, null, null, null);
        }
    }

    private TtlvStructure commonAttributes() {
        try (TtlvStructure base = attributes(
                     StandardAttribute.CRYPTOGRAPHIC_ALGORITHM,
                     TtlvValue.Enumeration(algorithm),
                     StandardAttribute.CRYPTOGRAPHIC_LENGTH,
                     TtlvValue.Integer(length));
             AttributeSet values = AttributeSet.tryNew(base)) {
            for (TypedAttribute attribute : commonAttributes) {
                values.tryPushTyped(attribute.attribute(), attribute);
            }
            return values.intoItems();
        }
    }

    /** Alias for {@link #build()}. */
    public ClientBatchItem toBatchItem() {
        return build();
    }

    /** Executes one typed Create Key Pair exchange without retrying it. */
    public ClientBatchResponse execute(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are invalid");
        if (client == null) throw new InvalidInputException("client is required to execute this builder");
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(build());
            if (asynchronousIndicator != null) batch.asynchronousIndicator(asynchronousIndicator);
            return options == null ? client.execute(batch, limits)
                    : client.executeWithOptions(batch, limits, options);
        }
    }

    private static TtlvStructure optionalUsage(Integer usageMask) {
        if (usageMask == null) return null;
        return attributes(StandardAttribute.CRYPTOGRAPHIC_USAGE_MASK, TtlvValue.Integer(usageMask));
    }

    private static TtlvStructure attributes(
            StandardAttribute firstAttribute, TtlvValue firstValue,
            StandardAttribute secondAttribute, TtlvValue secondValue) {
        try (TtlvValue first = firstValue;
             TtlvValue second = secondValue;
             TypedAttribute firstTyped = TypedAttribute.create(firstAttribute, first);
             TypedAttribute secondTyped = TypedAttribute.create(secondAttribute, second);
             TtlvStructure empty = TtlvStructure.create();
             AttributeSet values = AttributeSet.tryNew(empty)) {
            values.tryPushTyped(firstAttribute, firstTyped);
            values.tryPushTyped(secondAttribute, secondTyped);
            return values.intoItems();
        }
    }

    private static TtlvStructure attributes(StandardAttribute attribute, TtlvValue value) {
        try (TtlvValue ownedValue = value;
             TypedAttribute typed = TypedAttribute.create(attribute, ownedValue);
             TtlvStructure empty = TtlvStructure.create();
             AttributeSet values = AttributeSet.tryNew(empty)) {
            values.tryPushTyped(attribute, typed);
            return values.intoItems();
        }
    }

}
