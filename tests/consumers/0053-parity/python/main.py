"""External Python package consumer; all data stays local to this process."""

from kmipkit import (
    client,
    credentials,
    cryptographic_parameters,
    extensions,
    managed_objects,
    messages,
    ttlv,
    typed_attributes,
    validation,
)


def _item(raw_tag: int, value: ttlv.Value, limits: ttlv.CodecLimits) -> ttlv.TtlvItem:
    with ttlv.raw_tag_from_raw(raw_tag) as raw:
        tag = ttlv.raw_tag_try_checked(raw)
        try:
            return ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()


def _append(
    structure: ttlv.Structure,
    raw_tag: int,
    value: ttlv.Value,
    limits: ttlv.CodecLimits,
) -> ttlv.Structure:
    return ttlv.ttlv_structure_with_item(structure, _item(raw_tag, value, limits), limits)


def _synthetic_ping_response(limits: ttlv.CodecLimits) -> ttlv.Structure:
    version = ttlv.ttlv_structure_create()
    version = _append(version, 0x0042_006A, ttlv.ttlv_value_integer(2), limits)
    version = _append(version, 0x0042_006B, ttlv.ttlv_value_integer(1), limits)
    header = ttlv.ttlv_structure_create()
    header = _append(header, 0x0042_0069, ttlv.ttlv_value_structure(version, limits), limits)
    header = _append(header, 0x0042_0092, ttlv.ttlv_value_date_time(1_751_100_123), limits)
    header = _append(header, 0x0042_000D, ttlv.ttlv_value_integer(1), limits)
    batch_item = ttlv.ttlv_structure_create()
    batch_item = _append(batch_item, 0x0042_005C, ttlv.ttlv_value_enumeration(0x3B), limits)
    batch_item = _append(batch_item, 0x0042_007F, ttlv.ttlv_value_enumeration(0), limits)
    batch_item = _append(
        batch_item, 0x0042_007C, ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), limits), limits
    )
    root = ttlv.ttlv_structure_create()
    root = _append(root, 0x0042_007A, ttlv.ttlv_value_structure(header, limits), limits)
    return _append(root, 0x0042_000F, ttlv.ttlv_value_structure(batch_item, limits), limits)


def _demo_authentication(limits: ttlv.CodecLimits) -> credentials.Authentication:
    value = ttlv.ttlv_structure_create()
    value = _append(value, 0x0042_00BB, ttlv.ttlv_value_text_string(b"demo-user", limits), limits)
    envelope = ttlv.ttlv_structure_create()
    envelope = _append(envelope, 0x0042_0024, ttlv.ttlv_value_enumeration(1), limits)
    envelope = _append(envelope, 0x0042_0025, ttlv.ttlv_value_structure(value, limits), limits)
    return credentials.Authentication.create([credentials.Credential.from_ttlv(envelope)])


def main() -> None:
    limits = ttlv.codec_limits_defaults()
    with limits:
        with ttlv.ttlv_value_text_string(b"local example", limits) as value:
            with typed_attributes.TypedAttribute.create(
                typed_attributes.StandardAttribute.CONTACT_INFORMATION, value
            ) as attribute:
                assert attribute.attribute is typed_attributes.StandardAttribute.CONTACT_INFORMATION

        opaque_fields = [
            _item(0x0042_0059, ttlv.ttlv_value_enumeration(1), limits),
            _item(0x0042_005A, ttlv.ttlv_value_byte_string(b"demo bytes", limits), limits),
        ]
        with managed_objects.ManagedObject.opaque_object(opaque_fields) as managed_object:
            assert managed_object.object_type == 8
        for field in opaque_fields:
            field.close()

        raw_material = _item(
            0x0042_0043, ttlv.ttlv_value_byte_string(b"demo key bytes", limits), limits
        )
        with managed_objects.KeyMaterial.from_item(1, raw_material) as key_material:
            assert key_material.format_raw == 1
        raw_material.close()

        with cryptographic_parameters.CryptographicParameters.builder() as builder:
            with builder.cryptographic_algorithm(0x8000_0001).build() as parameters:
                assert parameters.cryptographic_algorithm() == 0x8000_0001

        ping = extensions.client_batch_item_ping()
        with validation.validate_request(ping) as validation_report:
            assert validation_report.is_valid
        batch = client.Batch.create()
        batch.add_item(ping)
        batch.set_authentication(_demo_authentication(limits))
        assert batch.item_count == 1
        batch.close()

        with messages.ResponseMessage.from_ttlv(_synthetic_ping_response(limits)) as response:
            assert response.batch_item_count == 1
            with response.batch_item_at(0) as item:
                assert item.result_status == 0
                with item.to_ttlv() as response_ttlv:
                    with ttlv.ttlv_structure_view(response_ttlv) as response_view:
                        assert ttlv.ttlv_structure_view_item_count(response_view) == 3

if __name__ == "__main__":
    main()
