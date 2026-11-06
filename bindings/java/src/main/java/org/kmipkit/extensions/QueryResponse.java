package org.kmipkit.extensions;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.OptionalLong;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValueView;

/** Typed readers for the fields of a KMIP Query Response Payload. */
public final class QueryResponse implements AutoCloseable {
    private static final int OPERATION = 0x0042_005c;
    private static final int OBJECT_TYPE = 0x0042_0057;
    private static final int VENDOR_IDENTIFICATION = 0x0042_009d;
    private static final int SERVER_INFORMATION = 0x0042_0088;
    private static final int APPLICATION_NAMESPACE = 0x0042_0003;
    private static final int EXTENSION_INFORMATION = 0x0042_00a4;
    private static final int ATTESTATION_TYPE = 0x0042_00c7;
    private static final int RNG_PARAMETERS = 0x0042_00d9;
    private static final int PROFILE_INFORMATION = 0x0042_00eb;
    private static final int VALIDATION_INFORMATION = 0x0042_00df;
    private static final int CAPABILITY_INFORMATION = 0x0042_00f7;
    private static final int CLIENT_REGISTRATION_METHOD = 0x0042_00f6;
    private static final int DEFAULTS_INFORMATION = 0x0042_0152;
    private static final int PROTECTION_STORAGE_MASKS = 0x0042_015f;
    private static final int MAX_FIELDS = 100_000;

    static final class FieldData {
        final int tag;
        final QueryResponseFieldKind kind;
        final long enumeration;

        FieldData(int tag, QueryResponseFieldKind kind, long enumeration) {
            this.tag = tag;
            this.kind = kind;
            this.enumeration = enumeration;
        }
    }

    private final TtlvStructure payload;
    private final TtlvStructureView view;
    private final List<FieldData> fields;
    private boolean closed;

    private QueryResponse(TtlvStructure payload, List<FieldData> fields) {
        this.payload = payload;
        this.view = payload.view();
        this.fields = List.copyOf(fields);
        if (fields.size() > MAX_FIELDS || view.itemCount() != fields.size()) {
            view.close();
            payload.close();
            throw new ResourceLimitException("Query response field count is invalid");
        }
    }

    static QueryResponse fromClassifiedPayload(TtlvStructure payload, List<FieldData> fields) {
        return new QueryResponse(payload, fields);
    }

    /** Returns the number of fields classified by the Rust Query response model. */
    public long fieldCount() {
        ensureOpen();
        return fields.size();
    }

    /** Returns every raw tag in original TTLV wire order, including repeated and vendor tags. */
    public List<Integer> rawFieldTags() {
        ensureOpen();
        return fields.stream().map(field -> field.tag).toList();
    }

    /** Returns unknown and vendor-defined raw tags in original TTLV wire order. */
    public List<Integer> unknownFieldTags() {
        ensureOpen();
        return fields.stream()
                .filter(field -> field.kind == QueryResponseFieldKind.UNKNOWN)
                .map(field -> field.tag)
                .toList();
    }

    /** Returns one Rust-classified field at its decoded wire-order index. */
    public QueryResponseField fieldAt(long index) {
        ensureOpen();
        if (index < 0 || index >= fields.size()) {
            throw new InvalidInputException("Query response field index is outside the payload");
        }
        return new QueryResponseField(this, Math.toIntExact(index), fields.get(Math.toIntExact(index)));
    }

    /** Returns all Operation values in their original order, including future enum values. */
    public List<Long> operations() { return enumerations(OPERATION); }

    /** Returns all Object Type values in their original order, including future enum values. */
    public List<Long> objectTypes() { return enumerations(OBJECT_TYPE); }

    /** Returns every Vendor Identification value in field order. */
    public List<String> vendorIdentifications() { return textValues(VENDOR_IDENTIFICATION); }

    /** Returns every Application Namespace value in field order. */
    public List<String> applicationNamespaces() { return textValues(APPLICATION_NAMESPACE); }

    /** Returns every Attestation Type value in field order, including future enum values. */
    public List<Long> attestationTypes() { return enumerations(ATTESTATION_TYPE); }

    /** Returns every Client Registration Method value in field order, including future values. */
    public List<Long> clientRegistrationMethods() {
        return enumerations(CLIENT_REGISTRATION_METHOD);
    }

    /** Returns Structure-valued fields in their original order. */
    public List<TtlvItemView> serverInformations() { return items(SERVER_INFORMATION); }

    /** Returns Structure-valued fields in their original order. */
    public List<TtlvItemView> extensionInformations() { return items(EXTENSION_INFORMATION); }

    /** Returns Structure-valued fields in their original order. */
    public List<TtlvItemView> rngParameters() { return items(RNG_PARAMETERS); }

    /** Returns Structure-valued fields in their original order. */
    public List<TtlvItemView> profileInformation() { return items(PROFILE_INFORMATION); }

    /** Returns Structure-valued fields in their original order. */
    public List<TtlvItemView> validationInformation() { return items(VALIDATION_INFORMATION); }

    /** Returns Structure-valued fields in their original order. */
    public List<TtlvItemView> capabilityInformation() { return items(CAPABILITY_INFORMATION); }

    /** Returns the optional Defaults Information field. */
    public Optional<TtlvItemView> defaultsInformation() {
        return firstItem(DEFAULTS_INFORMATION);
    }

    /** Returns the optional Protection Storage Masks field. */
    public Optional<TtlvItemView> protectionStorageMasks() {
        return firstItem(PROTECTION_STORAGE_MASKS);
    }

    /** Returns unknown and vendor-defined fields in their original order. */
    public List<TtlvItemView> unknownItems() {
        List<TtlvItemView> result = new ArrayList<>();
        for (int index = 0; index < fields.size(); index++) {
            if (fields.get(index).kind == QueryResponseFieldKind.UNKNOWN) {
                result.add(itemAt(index));
            }
        }
        return List.copyOf(result);
    }

    /** Returns an independent read-only view of the complete generic payload. */
    public TtlvStructureView rawView() {
        ensureOpen();
        return payload.view();
    }

    TtlvItemView itemAt(int index) {
        ensureOpen();
        if (index < 0 || index >= fields.size()) {
            throw new InvalidInputException("Query response field index is outside the payload");
        }
        return view.itemAt(index);
    }

    private List<Long> enumerations(int tag) {
        List<Long> result = new ArrayList<>();
        for (int index = 0; index < fields.size(); index++) {
            FieldData field = fields.get(index);
            if (field.tag == tag && field.kind == QueryResponseFieldKind.ENUMERATION) {
                result.add(field.enumeration);
            }
        }
        return List.copyOf(result);
    }

    private List<String> textValues(int tag) {
        List<String> result = new ArrayList<>();
        for (int index = 0; index < fields.size(); index++) {
            FieldData field = fields.get(index);
            if (field.tag != tag || field.kind != QueryResponseFieldKind.TEXT_STRING) continue;
            try (TtlvItemView item = itemAt(index); TtlvValueView value = item.value()) {
                if (item.itemType() != TtlvItemType.TextString) {
                    throw new InvalidInputException("Rust Query text field had an invalid TTLV type");
                }
                byte[] bytes = value.bytes();
                result.add(new String(bytes, StandardCharsets.UTF_8));
            }
        }
        return List.copyOf(result);
    }

    private List<TtlvItemView> items(int tag) {
        List<TtlvItemView> result = new ArrayList<>();
        for (int index = 0; index < fields.size(); index++) {
            FieldData field = fields.get(index);
            if (field.tag == tag && field.kind == QueryResponseFieldKind.STRUCTURE) {
                result.add(itemAt(index));
            }
        }
        return List.copyOf(result);
    }

    private Optional<TtlvItemView> firstItem(int tag) {
        List<TtlvItemView> matches = items(tag);
        if (matches.isEmpty()) return Optional.empty();
        for (int index = 1; index < matches.size(); index++) matches.get(index).close();
        return Optional.of(matches.get(0));
    }

    private void ensureOpen() {
        if (closed) throw new InvalidInputException("Query response is closed");
    }

    /** Releases the owned payload and its root view. */
    @Override
    public void close() {
        if (closed) return;
        closed = true;
        view.close();
        payload.close();
    }
}
