#!/usr/bin/env python3
"""Execute installed C ABI routes and retain honest, sanitized KMIPKIT-0092 evidence.

The coordinator uses ctypes only for local opaque-handle ABI calls. Network
requests are executed by the compiled C consumer. Fixture bytes stay in memory;
only normalized tree digests and numeric outcomes enter route reports.
"""
from __future__ import annotations

import argparse
import ctypes as C
import hashlib
import json
from pathlib import Path
import re
import sys
import uuid

from artifact_support import INSTALL, PIN, ROOT, sha256, verify_installed_artifact
from route_evidence import cell_fields, pending_cell
from safe_paths import resolve_within_root

sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells
from validate_route_evidence import _key, validate_collection

ATTRIBUTE_ROUTE_TARGET_KINDS = frozenset({'attribute', 'tag', 'enumeration_value', 'bitmask_value'})
ATTRIBUTE_ROUTE_CASE_CATEGORIES = frozenset(
    {'attributes', 'managed_objects', 'enums_and_bitmasks', 'normative_inventory'}
)
REQUEST_HEADER_FIELD_FIXTURES = {
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-MAXIMUM-RESPONSE-SIZE': ('Maximum Response Size', 2, 1024),
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ASYNCHRONOUS-INDICATOR': ('Asynchronous Indicator', 5, None),
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ATTESTATION-CAPABLE-INDICATOR':
        ('Attestation Capable Indicator', 6, 0),
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ATTESTATION-TYPE': ('Attestation Type', 5, None),
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-CLIENT-CORRELATION-VALUE':
        ('Client Correlation Value', 7, b'client-correlation'),
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-SERVER-CORRELATION-VALUE':
        ('Server Correlation Value', 7, b'server-correlation'),
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-TIME-STAMP': ('Time Stamp', 9, 1_760_000_000),
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ERROR-CONTINUATION-OPTION':
        ('Batch Error Continuation Option', 5, None),
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ORDER-OPTION': ('Batch Order Option', 6, 1),
}
REQUEST_HEADER_FIELD_ALIASES = {
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-10-SERVER-CORRELATION-VALUE':
        'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-SERVER-CORRELATION-VALUE',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-12-MAXIMUM-RESPONSE-SIZE':
        'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-MAXIMUM-RESPONSE-SIZE',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-2-ASYNCHRONOUS-INDICATOR':
        'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ASYNCHRONOUS-INDICATOR',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-3-ATTESTATION-CAPABLE-INDICATOR':
        'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ATTESTATION-CAPABLE-INDICATOR',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP':
        'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-TIME-STAMP',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-9-CLIENT-CORRELATION-VALUE':
        'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-CLIENT-CORRELATION-VALUE',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-6-BATCH-ERROR-CONTINUATION-OPTION':
        'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ERROR-CONTINUATION-OPTION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-8-BATCH-ORDER-OPTION':
        'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ORDER-OPTION',
}
REQUEST_HEADER_MULTI_ITEM_OPTION_TARGETS = frozenset({
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ERROR-CONTINUATION-OPTION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ORDER-OPTION',
})
REQUEST_MESSAGE_AUTHENTICATION_TARGETS = frozenset({
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-AUTHENTICATION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-4-AUTHENTICATION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-4-CREDENTIAL-MAY-BE-REPEATED',
})
REQUEST_MESSAGE_EXTENSION_TARGETS = frozenset({
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-3-MESSAGE-EXTENSION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-13-CRITICALITY-INDICATOR',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-13-VENDOR-EXTENSION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-13-VENDOR-IDENTIFICATION',
})
REQUEST_MESSAGE_CONTEXT_TARGETS = (
    REQUEST_MESSAGE_AUTHENTICATION_TARGETS | REQUEST_MESSAGE_EXTENSION_TARGETS | frozenset({
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-1-BATCH-ITEM',
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-1-REQUEST-HEADER',
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-COUNT',
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-PROTOCOL-VERSION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-REQUEST-HEADER',
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-3-BATCH-ITEM',
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-3-OPERATION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-3-REQUEST-PAYLOAD',
    'KMIPKIT-ELEM-MESSAGE-FIELD-8-3-UNIQUE-BATCH-ITEM-ID',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-15-OPERATION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION-MAJOR',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION-MINOR',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-21-UNIQUE-BATCH-ITEM-ID',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT',
    'KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM',
    })
)
OUTPUT_ROOT = ROOT / 'interop/c-kmip-validator/build'


def resolve_output_path(path: Path) -> Path:
    """Keep route reports and sidecars within the validator build directory."""
    return resolve_within_root(path, OUTPUT_ROOT)
TTLV_TYPE_PATTERNS = (
    (r'\bstructure\b', 1),
    (r'\bbig[\s-]*integer\b', 4),
    (r'\blong[\s-]*integer\b', 3),
    (r'\bdate[\s-]*time[\s-]*extended\b|\bdatetimeextended\b', 11),
    (r'\benumeration\b', 5),
    (r'\bboolean\b', 6),
    (r'\btext[\s-]*string\b|\btextstring\b', 7),
    (r'\bbyte[\s-]*string\b|\bbytestring\b', 8),
    (r'\bdate[\s-]*time\b|\bdatetime\b', 9),
    (r'\binterval\b', 10),
    (r'(?<![\w-])integer\b', 2),
)


def pending_reason(cell: dict) -> str:
    """An existing public symbol without a recipe is a validator gap."""
    return 'validator_recipe_pending'


def should_execute_attribute_case(cell: dict, case: dict, target: dict) -> bool:
    """Exercise local typed attribute routes independently of peer scenarios."""
    return (
        cell['transport'] is None
        and target['kind'] in ATTRIBUTE_ROUTE_TARGET_KINDS
        and case['category'] in ATTRIBUTE_ROUTE_CASE_CATEGORIES
    )


def uses_scalar_target_fixture(cell: dict, case: dict, target: dict) -> bool:
    """Choose scalar fixtures only for routes that consume a scalar target value."""
    tag_target_case = (case.get('requires', {}).get('enum_type') == 'Tag'
                       and target['kind'] in {'tag', 'tag_range'})
    scalar_target_case = (
        case['category'] == 'normative_inventory' and not case.get('requires', {}).get('protocol_scenario')
    ) or target['kind'] in {'operation', 'object_type'} or tag_target_case
    request_message_parser = (
        target['kind'] == 'tag'
        and target.get('name') == 'Request Message'
        and cell['public_api_symbol'] == 'kmipkit_request_message_from_ttlv'
    )
    return scalar_target_case and not request_message_parser


def require_complete(events: list[dict], pending: list[dict]) -> None:
    if pending:
        raise ValueError(f'{len(pending)} pending validator recipe cells')
    keys = [_key(event) for event in events]
    if len(keys) != len(set(keys)):
        raise ValueError('duplicate route execution cells')
    if any(event['status'] != 'passed' for event in events):
        raise ValueError('failed route execution cells')


def _name(value: str) -> str:
    return re.sub('[^a-z0-9]', '', value.lower())


def catalog_tag_values(catalog: dict) -> dict[str, int]:
    return {target['name']: int(target['wire_value'], 16)
            for target in catalog['normative_targets'] if target['kind'] == 'tag'}


def catalog_enum_value(catalog: dict, enum_name: str, value_name: str,
                      protocol_version: str | None) -> int | None:
    parent_id = _enum_parent_id(enum_name)
    matches = [target for target in catalog['normative_targets']
               if target['kind'] == 'enumeration_value'
               and target['name'] == value_name
               and parent_id in target.get('parent_element_ids', [])
               and (protocol_version is None or protocol_version in target['protocol_versions'])]
    if len(matches) != 1:
        return None
    return int(matches[0]['wire_value'], 16)


def _enum_parent_id(enum_name: str) -> str:
    return f'KMIPKIT-ELEM-ENUMERATION-{enum_name.upper().replace(" ", "-")}'


def catalog_enum_values(catalog: dict, enum_name: str, protocol_version: str) -> list[int]:
    parent_id = _enum_parent_id(enum_name)
    return sorted(int(target['wire_value'], 16) for target in catalog['normative_targets']
                  if target['kind'] == 'enumeration_value'
                  and parent_id in target.get('parent_element_ids', [])
                  and protocol_version in target['protocol_versions']
                  and re.fullmatch(r'[0-9a-fA-F]{8}', target.get('wire_value', '')))


def catalog_parent_tag(target: dict, tags: dict[str, int]) -> int | None:
    parent = target.get('parent_name')
    if not parent:
        parents = target.get('parent_element_ids', [])
        parent = parents[0] if parents else ''
    name = re.sub(r'^KMIPKIT-ELEM-(?:ENUMERATION|BITMASK)-', '', parent)
    return {_name(tag_name): value for tag_name, value in tags.items()}.get(_name(name))


def numeric_boundaries(value: str, width: int = 8) -> tuple[int, int] | None:
    endpoints = re.findall(r'[0-9A-Fa-fXx]+', value)
    if not endpoints:
        return None
    lower_text, upper_text = endpoints[0], endpoints[-1]
    pattern = rf'[0-9A-Fa-fXx]{{{width}}}'
    if not re.fullmatch(pattern, lower_text) or not re.fullmatch(pattern, upper_text):
        return None
    lower = int(lower_text.replace('X', '0').replace('x', '0'), 16)
    upper = int(upper_text.replace('X', 'F').replace('x', 'F'), 16)
    if lower > upper:
        return None
    return lower, upper


def range_boundary_tags(target: dict) -> tuple[int, int]:
    boundaries = numeric_boundaries(target.get('wire_value', ''), width=6)
    if boundaries is None:
        raise ValueError('catalog_tag_range_boundaries_missing')
    return boundaries


def _type_codes(encoding: str) -> list[int]:
    """Read only TTLV types explicitly named in a catalog encoding declaration."""
    matches = []
    for pattern, code in TTLV_TYPE_PATTERNS:
        matches.extend((match.start(), match.end(), code)
                       for match in re.finditer(pattern, encoding, flags=re.IGNORECASE))
    selected = []
    for start, end, code in sorted(matches, key=lambda item: (item[0], -(item[1] - item[0]))):
        if any(start >= outer_start and end <= outer_end
               for outer_start, outer_end, _ in selected):
            continue
        selected.append((start, end, code))
    return [code for _, _, code in sorted(selected)]


def _fixture_value(target: dict, tag: int, kind: int, catalog: dict,
                   protocol_version: str) -> tuple[int, int, object]:
    if kind == 1:
        value = []
    elif kind in (2, 3, 4, 5, 6, 9, 10, 11):
        size = 8 if kind in (3, 4, 6, 9, 11) else 4
        number = 0
        if kind == 5:
            enum_name = target.get('field_name', target['name'])
            values = catalog_enum_values(catalog, enum_name, protocol_version)
            number = values[0] if values else 0
        value = number.to_bytes(size, 'big')
    elif kind == 7:
        value = b'fixture'
    elif kind == 8:
        value = bytes.fromhex('010203')
    else:
        raise ValueError('fixture_unsupported_ttlv_type')
    return tag, kind, value


def _required_structure_items(target: dict, catalog: dict, tags: dict[str, int],
                              protocol_version: str) -> list[tuple]:
    """Build child Items from required fields in the pinned normative inventory."""
    target_id = target.get('target_id')
    if not target_id:
        return []
    children = [candidate for candidate in catalog['normative_targets']
                if target_id in candidate.get('parent_element_ids', [])
                and protocol_version in candidate.get('protocol_versions', [])]
    required = [child for child in children
                if child.get('source_requiredness', '').strip().lower() in {'yes', 'required'}]
    selected = required or children
    items = []
    for child in selected:
        fixtures = generic_tree_fixtures(child, catalog, tags, protocol_version)
        if fixtures:
            items.append(fixtures[0])
    return items


def _request_message_fixture(protocol_version: str, tags: dict[str, int], catalog: dict,
                             *, ephemeral: int | None = None) -> tuple | None:
    """Build a complete Get request for the public Request Message parser."""
    operation = catalog_enum_value(catalog, 'Operation', 'Get', protocol_version)
    if operation is None:
        return None
    header = (tags['Request Header'], 1, [
        (tags['Protocol Version'], 1, [
            (tags['Protocol Version Major'], 2, (2).to_bytes(4, 'big')),
            (tags['Protocol Version Minor'], 2,
             int(protocol_version.split('.')[1]).to_bytes(4, 'big')),
        ]),
        (tags['Batch Count'], 2, (1).to_bytes(4, 'big')),
    ])
    identifier = (b'kmipkit-ephemeral-fixture' if ephemeral is not None
                  else b'kmipkit-request-message-fixture')
    payload = (tags['Request Payload'], 1, [
        (tags['Unique Identifier'], 7, identifier),
    ])
    batch_fields = [(tags['Operation'], 5, operation.to_bytes(4, 'big'))]
    if ephemeral is not None:
        batch_fields.append((tags['Ephemeral'], 6, ephemeral.to_bytes(8, 'big')))
    batch_fields.append(payload)
    return (tags['Request Message'], 1, [header, (tags['Batch Item'], 1, batch_fields)])


def _with_request_header_field(fixture: tuple, tags: dict[str, int], field: tuple,
                               *, after_tag: str) -> tuple:
    """Insert one optional field at its normative position in Request Header."""
    header_index = next(index for index, item in enumerate(fixture[2])
                        if item[0] == tags['Request Header'])
    header = fixture[2][header_index]
    fields = list(header[2])
    anchor_index = next(index for index, item in enumerate(fields)
                        if item[0] == tags[after_tag])
    fields.insert(anchor_index + 1, field)
    message = list(fixture[2])
    message[header_index] = (header[0], header[1], fields)
    return fixture[0], fixture[1], message


def _request_message_with_batch_ids(fixture: tuple, tags: dict[str, int],
                                    identifiers: tuple[bytes, ...]) -> tuple:
    """Build request Batch Items with the supplied unique identifiers and count."""
    message = list(fixture[2])
    header_index = next(index for index, item in enumerate(message)
                        if item[0] == tags['Request Header'])
    header = message[header_index]
    batch_index = next(index for index, item in enumerate(message)
                       if item[0] == tags['Batch Item'])
    batch_item = message[batch_index]
    payload_index = next(index for index, item in enumerate(batch_item[2])
                         if item[0] == tags['Request Payload'])
    items = []
    for identifier in identifiers:
        fields = list(batch_item[2])
        fields.insert(payload_index,
                      (tags['Unique Batch Item ID'], 8, identifier))
        items.append((batch_item[0], batch_item[1], fields))
    if len(items) > 1:
        header_fields = list(header[2])
        count_index = next(index for index, item in enumerate(header_fields)
                           if item[0] == tags['Batch Count'])
        header_fields[count_index] = (tags['Batch Count'], 2,
                                      len(items).to_bytes(4, 'big'))
        message[header_index] = (header[0], header[1], header_fields)
        message = [item for item in message if item[0] != tags['Batch Item']]
        message.extend(items)
    else:
        message[batch_index] = items[0]
    return fixture[0], fixture[1], message


def _request_message_with_authentication(fixture: tuple, catalog: dict,
                                         tags: dict[str, int],
                                         protocol_version: str) -> tuple | None:
    """Add a catalogued Username and Password Credential to the Request Header."""
    credential_type = catalog_enum_value(catalog, 'Credential Type',
                                         'Username and Password', protocol_version)
    if credential_type is None:
        return None
    credential_value = (tags['Credential Value'], 1, [
        (tags['Username'], 7, b'fixture-user'),
        (tags['Password'], 7, b'fixture-pass'),
    ])
    credential = (tags['Credential'], 1, [
        (tags['Credential Type'], 5, credential_type.to_bytes(4, 'big')),
        credential_value,
    ])
    authentication = (tags['Authentication'], 1, [credential])
    return _with_request_header_field(fixture, tags, authentication,
                                      after_tag='Protocol Version')


def _with_request_batch_item_field(fixture: tuple, tags: dict[str, int], field: tuple,
                                   *, after_tag: str) -> tuple:
    """Insert one optional field after its required Request Batch Item anchor."""
    message = list(fixture[2])
    batch_index = next(index for index, item in enumerate(message)
                       if item[0] == tags['Batch Item'])
    batch_item = message[batch_index]
    fields = list(batch_item[2])
    anchor_index = next(index for index, item in enumerate(fields)
                        if item[0] == tags[after_tag])
    fields.insert(anchor_index + 1, field)
    message[batch_index] = (batch_item[0], batch_item[1], fields)
    return fixture[0], fixture[1], message


def _request_message_with_extension(fixture: tuple, tags: dict[str, int]) -> tuple:
    """Add a structurally valid vendor Message Extension to the Get batch item."""
    extension = (tags['Message Extension'], 1, [
        (tags['Vendor Identification'], 7, b'kmipkit.test'),
        (tags['Criticality Indicator'], 6, bytes(8)),
        (tags['Vendor Extension'], 1, [
            (0x540001, 7, b'fixture-extension'),
        ]),
    ])
    return _with_request_batch_item_field(fixture, tags, extension,
                                          after_tag='Request Payload')


def _request_header_field_data(name: str, kind: int, value: object, catalog: dict,
                               protocol_version: str) -> bytes | None:
    if kind == 5:
        values = catalog_enum_values(catalog, name, protocol_version)
        if not values:
            return None
        value = values[0]
        return value.to_bytes(4, 'big')
    if kind in (2, 6, 9):
        width = 8 if kind in (6, 9) else 4
        data = int(value).to_bytes(width, 'big', signed=kind == 9)
        return data
    if kind in (7, 8):
        return value
    raise ValueError('unsupported_request_header_fixture_type')


def _request_header_field_fixture(target: dict, catalog: dict, tags: dict[str, int],
                                  protocol_version: str) -> tuple | None:
    target_id = target.get('target_id')
    canonical_target_id = REQUEST_HEADER_FIELD_ALIASES.get(target_id, target_id)
    specification = REQUEST_HEADER_FIELD_FIXTURES.get(canonical_target_id)
    if specification is None:
        return None
    name, kind, value = specification
    tag = tags.get(name)
    if tag is None:
        return None
    data = _request_header_field_data(name, kind, value, catalog, protocol_version)
    return (tag, kind, data) if data is not None else None


def generic_tree_fixtures(target: dict, catalog: dict, tags: dict[str, int],
                          protocol_version: str) -> list[tuple] | None:
    """Build item trees only where the pinned catalog identifies tag and type."""
    if target.get('kind') in {'tag', 'message_field'} and target.get('name') == 'Request Message':
        fixture = _request_message_fixture(protocol_version, tags, catalog)
        return [fixture] if fixture is not None else None
    if target.get('target_id') in REQUEST_MESSAGE_CONTEXT_TARGETS:
        fixture = _request_message_fixture(protocol_version, tags, catalog)
        if fixture is None:
            return None
        if target.get('target_id') in REQUEST_MESSAGE_EXTENSION_TARGETS:
            return [_request_message_with_extension(fixture, tags)]
        if target.get('target_id') in REQUEST_MESSAGE_AUTHENTICATION_TARGETS:
            fixture = _request_message_with_authentication(
                fixture, catalog, tags, protocol_version)
            return [fixture] if fixture is not None else None
        if target.get('target_id') in {
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-3-UNIQUE-BATCH-ITEM-ID',
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-21-UNIQUE-BATCH-ITEM-ID',
        }:
            fixture = _request_message_with_batch_ids(fixture, tags, (b'route-batch-item',))
        return [fixture]
    header_field = _request_header_field_fixture(target, catalog, tags, protocol_version)
    if header_field is not None:
        fixture = _request_message_fixture(protocol_version, tags, catalog)
        if fixture is None:
            return None
        canonical_target_id = REQUEST_HEADER_FIELD_ALIASES.get(
            target.get('target_id'), target.get('target_id'))
        if canonical_target_id in REQUEST_HEADER_MULTI_ITEM_OPTION_TARGETS:
            fixture = _request_message_with_batch_ids(
                fixture, tags, (b'route-batch-item-1', b'route-batch-item-2'))
        return [_with_request_header_field(fixture, tags, header_field,
                                           after_tag='Protocol Version')]
    if target.get('target_id') == 'KMIPKIT-ELEM-MESSAGE-FIELD-8-3-EPHEMERAL':
        fixtures = [_request_message_fixture(protocol_version, tags, catalog, ephemeral=value)
                    for value in (0, 1)]
        return fixtures if all(fixture is not None for fixture in fixtures) else None
    if target['kind'] == 'tag' and target.get('allocation') in {'reserved', 'unused'}:
        return None
    if target['kind'] == 'tag_range' and target.get('allocation') == 'extension':
        return [(tag, 5, (7).to_bytes(4, 'big')) for tag in range_boundary_tags(target)]
    if (target['kind'] in {'enumeration_value', 'bitmask_value', 'referenced_enum_value'}
            and target.get('allocation') in {'extension', 'reserved'}):
        boundaries = numeric_boundaries(target.get('wire_value', ''))
        tag = catalog_parent_tag(target, tags)
        if boundaries is not None and tag is not None:
            item_type = 2 if target['kind'] == 'bitmask_value' else 5
            return [(tag, item_type, value.to_bytes(4, 'big')) for value in dict.fromkeys(boundaries)]
    scalar = scalar_fixture(target, tags, catalog, protocol_version)
    if scalar is not None:
        return [scalar]

    target_kind = target['kind']

    field_name = target.get('field_name', target['name'])
    normalized_tags = {_name(name): value for name, value in tags.items()}
    tag = normalized_tags.get(_name(field_name))
    if target_kind == 'credential':
        tag = normalized_tags.get('credential')
        if target['name'] in {'Device', 'Attestation', 'Username and Password'}:
            credential_value = catalog_enum_value(catalog, 'Credential Type', target['name'], protocol_version)
            credential_type_tag = normalized_tags.get('credentialtype')
            if credential_value is None or credential_type_tag is None or tag is None:
                return None
            return [(tag, 1, [(credential_type_tag, 5, credential_value.to_bytes(4, 'big'))])]
    if tag is None:
        return None

    encodings = []
    direct_encoding = target.get('value_type') if target_kind == 'operation_payload_value_variant' else target.get('source_encoding')
    if direct_encoding:
        encodings.extend(_type_codes(direct_encoding))
    elif target_kind in {'operation_payload_field', 'message_field'}:
        for candidate in catalog['normative_targets']:
            candidate_name = candidate.get('field_name', candidate['name'])
            candidate_tag = normalized_tags.get(_name(candidate_name))
            encoding = candidate.get('value_type') if candidate['kind'] == 'operation_payload_value_variant' else candidate.get('source_encoding')
            if candidate_tag == tag and encoding and protocol_version in candidate['protocol_versions']:
                encodings.extend(_type_codes(encoding))
    elif target_kind in {'attribute_structure', 'operation_structure'}:
        encodings.append(1)

    encodings = list(dict.fromkeys(encodings))
    if not encodings:
        return None
    fixtures = []
    for kind in encodings:
        if kind == 1:
            fixtures.append((tag, kind, _required_structure_items(
                target, catalog, tags, protocol_version)))
        else:
            fixtures.append(_fixture_value(target, tag, kind, catalog, protocol_version))
    return fixtures


def execute_tag_range_case(tree_abi, target: dict, catalog: dict) -> dict:
    """Exercise both allocation boundaries with construction or decoder rejection."""
    from route_ttlv import digest, encode

    tags = catalog_tag_values(catalog)
    if target.get('allocation') == 'extension':
        trees = generic_tree_fixtures(target, catalog, tags, target['protocol_versions'][-1])
        if not trees:
            raise ValueError('extension_tag_range_fixture_missing')
        results = [tree_abi.roundtrip(encode(tree)) for tree in trees]
        return {'sample_count': len(results),
                'invoked_symbols': set().union(*(result['invoked_symbols'] for result in results)),
                'tree_sha256': digest([result['normalized_tree'] for result in results])}

    results, symbols, wires, rejection_codes = [], set(), [], []
    for tag in range_boundary_tags(target):
        wire = encode((tag, 5, (7).to_bytes(4, 'big')))
        code = tree_abi.reject_wire(wire)
        if code not in {11, 12}:
            raise ValueError('tag_range_rejection_code_mismatch')
        symbols.update(tree_abi.called)
        wires.append(wire)
        rejection_codes.append(code)
        results.append([tag, code])
    return {'sample_count': len(results), 'invoked_symbols': symbols, 'tree_sha256': digest(results),
            'rejection_codes': rejection_codes,
            'decode_input_sha256': hashlib.sha256(b''.join(wires)).hexdigest()}


def scalar_fixture(target: dict, tags: dict[str, int], catalog: dict | None = None,
                   protocol_version: str | None = None) -> tuple[int, int, bytes] | None:
    """Use each exact normative enum/flag/tag; never substitute an operation probe."""
    normalized_tags = {_name(name): value for name, value in tags.items()}
    kind = target['kind']
    if kind == 'tag':
        return int(target['wire_value'], 16), 2, (7).to_bytes(4, 'big')
    if kind == 'operation' and catalog is not None:
        value_version = protocol_version
        if value_version not in target['protocol_versions']:
            value_version = target['protocol_versions'][-1]
        value = catalog_enum_value(catalog, 'Operation', target['name'], value_version)
        tag = normalized_tags.get('operation')
        if tag is None or value is None:
            return None
        return tag, 5, value.to_bytes(4, 'big')
    if kind == 'object_type':
        tag = normalized_tags.get('objecttype')
        raw = target.get('wire_value', '')
        if tag is None or not re.fullmatch(r'[0-9a-fA-F]{1,8}', raw):
            return None
        return tag, 5, int(raw, 16).to_bytes(4, 'big')
    if kind not in {'enumeration_value', 'bitmask_value', 'referenced_enum_value'}:
        return None
    tag = catalog_parent_tag(target, tags)
    if tag is None or not re.fullmatch(r'[0-9a-fA-F]{1,8}', target.get('wire_value', '')):
        return None
    return tag, 2 if kind == 'bitmask_value' else 5, int(target['wire_value'], 16).to_bytes(4, 'big')


class DecodeError(C.Structure):
    _fields_ = [('struct_size', C.c_uint32), ('abi_version', C.c_uint32),
                ('kind', C.c_uint32), ('byte_offset', C.c_uint64)]


class InstalledAbi:
    """Typed signatures and exact ownership from the installed public C header."""
    def __init__(self, library: Path):
        self.library = C.CDLL(str(library))
        self.called: set[str] = set()
        self.bind('kmipkit_codec_limits_defaults', [C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_raw_tag_create', [C.c_uint32, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_raw_tag_try_checked', [C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_raw_tag_value', [C.c_void_p, C.POINTER(C.c_uint32)])
        self.bind('kmipkit_ttlv_item_create', [C.c_void_p, C.c_void_p, C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_decode', [C.c_void_p, C.c_uint64, C.POINTER(DecodeError), C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_item_view_from_item', [C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_item_view_tag', [C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_tag_value', [C.c_void_p, C.POINTER(C.c_uint32)])
        self.bind('kmipkit_ttlv_item_view_type', [C.c_void_p, C.POINTER(C.c_uint8)])
        self.bind('kmipkit_ttlv_item_view_value', [C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_value_integer', [C.c_int32, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_value_enumeration', [C.c_uint32, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_value_view_integer', [C.c_void_p, C.POINTER(C.c_int32)])
        self.bind('kmipkit_ttlv_value_view_enumeration', [C.c_void_p, C.POINTER(C.c_uint32)])
        self.bind('kmipkit_ttlv_structure_create', [C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_structure_with_item', [C.c_void_p, C.c_void_p, C.c_void_p, C.POINTER(C.c_void_p)])
        for name in ('codec_limits', 'raw_tag', 'tag', 'ttlv_value', 'ttlv_item',
                     'ttlv_item_view', 'ttlv_value_view', 'ttlv_structure'):
            function = getattr(self.library, f'kmipkit_{name}_release')
            function.argtypes, function.restype = [C.c_void_p], None

    def bind(self, name: str, signature: list) -> None:
        function = getattr(self.library, name)
        function.argtypes, function.restype = signature, C.c_int32

    def call(self, symbol: str, *arguments) -> None:
        self.called.add(symbol)
        status = getattr(self.library, symbol)(*arguments)
        if status != 0:
            raise ValueError('installed_abi_call_failed')

    def inspect(self, item: C.c_void_p, fixture: tuple[int, int, bytes]) -> None:
        expected_tag, expected_type, expected_value = fixture
        view, tag, value = C.c_void_p(), C.c_void_p(), C.c_void_p()
        try:
            self.call('kmipkit_ttlv_item_view_from_item', item, C.byref(view))
            self.call('kmipkit_ttlv_item_view_tag', view, C.byref(tag))
            actual_tag, actual_type = C.c_uint32(), C.c_uint8()
            self.call('kmipkit_ttlv_tag_value', tag, C.byref(actual_tag))
            self.call('kmipkit_ttlv_item_view_type', view, C.byref(actual_type))
            self.call('kmipkit_ttlv_item_view_value', view, C.byref(value))
            number = C.c_int32() if expected_type == 2 else C.c_uint32()
            self.call('kmipkit_ttlv_value_view_integer' if expected_type == 2
                      else 'kmipkit_ttlv_value_view_enumeration', value, C.byref(number))
            actual = int(number.value).to_bytes(4, 'big', signed=expected_type == 2)
            if (actual_tag.value, actual_type.value, actual) != fixture:
                raise ValueError('installed_abi_value_mismatch')
        finally:
            for name, handle in (('ttlv_value_view', value), ('tag', tag), ('ttlv_item_view', view)):
                if handle.value:
                    getattr(self.library, f'kmipkit_{name}_release')(handle)


    def execute_reserved_tag(self, symbol: str, tag_code: int) -> bytes:
        self.called.clear()
        wire = tag_code.to_bytes(3, 'big') + bytes.fromhex('02000000040000000700000000')
        raw_tag, checked_tag, item = C.c_void_p(), C.c_void_p(), C.c_void_p()
        try:
            if symbol == 'kmipkit_ttlv_decode':
                error = DecodeError(C.sizeof(DecodeError), 1, 0, 0)
                data = C.create_string_buffer(wire)
                self.called.add(symbol)
                status = self.library.kmipkit_ttlv_decode(data, len(wire), C.byref(error), C.byref(item))
                if status == 0 or item.value or error.kind != 11:
                    raise ValueError('reserved_tag_not_rejected')
            else:
                self.call('kmipkit_ttlv_raw_tag_create', tag_code, C.byref(raw_tag))
                if symbol == 'kmipkit_ttlv_raw_tag_create':
                    actual = C.c_uint32()
                    self.call('kmipkit_ttlv_raw_tag_value', raw_tag, C.byref(actual))
                    if actual.value != tag_code:
                        raise ValueError('reserved_raw_tag_not_preserved')
                elif symbol == 'kmipkit_ttlv_raw_tag_try_checked':
                    self.called.add(symbol)
                    status = self.library.kmipkit_ttlv_raw_tag_try_checked(raw_tag, C.byref(checked_tag))
                    if status == 0 or checked_tag.value:
                        raise ValueError('reserved_tag_not_rejected')
                else:
                    raise ValueError('reserved_tag_checked_constructor_not_representable')
            return wire
        finally:
            for name, handle in (('raw_tag', raw_tag), ('tag', checked_tag), ('ttlv_item', item)):
                if handle.value:
                    getattr(self.library, f'kmipkit_{name}_release')(handle)

    def execute(self, symbol: str, fixture: tuple[int, int, bytes]) -> bytes:
        """Construct and decode independently; compare both public views exactly.

        C exposes no TTLV encoder. The harness encodes the independently supplied
        vector and records no public encode event. Handle transfer follows the
        installed header, including transfers before a late validation failure.
        """
        self.called.clear()
        tag_code, item_type, value_bytes = fixture
        wire = tag_code.to_bytes(3, 'big') + bytes([item_type]) + (4).to_bytes(4, 'big') + value_bytes + bytes(4)
        handles = {name: C.c_void_p() for name in ('codec_limits', 'raw_tag', 'tag', 'ttlv_value', 'ttlv_item', 'ttlv_structure')}
        decoded = C.c_void_p()
        try:
            self.call('kmipkit_codec_limits_defaults', C.byref(handles['codec_limits']))
            self.call('kmipkit_ttlv_raw_tag_create', tag_code, C.byref(handles['raw_tag']))
            self.call('kmipkit_ttlv_raw_tag_try_checked', handles['raw_tag'], C.byref(handles['tag']))
            number = int.from_bytes(value_bytes, 'big', signed=item_type == 2)
            self.call('kmipkit_ttlv_value_integer' if item_type == 2 else 'kmipkit_ttlv_value_enumeration',
                      number, C.byref(handles['ttlv_value']))
            consumed_value = handles['ttlv_value']
            handles['ttlv_value'] = C.c_void_p()
            self.call('kmipkit_ttlv_item_create', handles['tag'], consumed_value,
                      handles['codec_limits'], C.byref(handles['ttlv_item']))
            self.inspect(handles['ttlv_item'], fixture)
            error = DecodeError(C.sizeof(DecodeError), 1, 0, 0)
            data = C.create_string_buffer(wire)
            self.call('kmipkit_ttlv_decode', data, len(wire), C.byref(error), C.byref(decoded))
            self.inspect(decoded, fixture)
            if symbol in {'kmipkit_ttlv_structure_create', 'kmipkit_ttlv_structure_with_item'}:
                self.call('kmipkit_ttlv_structure_create', C.byref(handles['ttlv_structure']))
                original_structure, original_item = handles['ttlv_structure'], handles['ttlv_item']
                handles['ttlv_structure'], handles['ttlv_item'] = C.c_void_p(), C.c_void_p()
                self.call('kmipkit_ttlv_structure_with_item', original_structure, original_item,
                          handles['codec_limits'], C.byref(handles['ttlv_structure']))
            if symbol not in self.called:
                raise ValueError('named_route_not_invoked')
            return wire
        finally:
            if decoded.value:
                self.library.kmipkit_ttlv_item_release(decoded)
            for name, handle in reversed(list(handles.items())):
                if handle.value:
                    getattr(self.library, f'kmipkit_{name}_release')(handle)


LOCAL_SYMBOLS = {
    'kmipkit_ttlv_decode', 'kmipkit_ttlv_item_create', 'kmipkit_ttlv_raw_tag_create',
    'kmipkit_request_message_from_ttlv',
    'kmipkit_ttlv_raw_tag_try_checked', 'kmipkit_ttlv_value_integer',
    'kmipkit_ttlv_value_long_integer', 'kmipkit_ttlv_value_enumeration',
    'kmipkit_ttlv_value_boolean', 'kmipkit_ttlv_value_date_time',
    'kmipkit_ttlv_value_interval', 'kmipkit_ttlv_value_date_time_extended',
    'kmipkit_ttlv_value_big_integer', 'kmipkit_ttlv_value_text_string',
    'kmipkit_ttlv_value_byte_string', 'kmipkit_ttlv_value_structure',
    'kmipkit_ttlv_structure_create',
    'kmipkit_ttlv_structure_with_item', 'kmipkit_ttlv_item_view_from_item',
}


def invoked_event(cell: dict, index: int) -> dict:
    event = cell_fields(cell)
    event.update(invocation_id=f'c-abi-{index}', status='passed', invocation_count=1)
    return event


def event_from_cell(cell: dict, wire: bytes, index: int) -> dict:
    event = invoked_event(cell, index)
    tree = {'tag': int.from_bytes(wire[:3], 'big'), 'type': wire[3], 'value': wire[8:12].hex()}
    tree_hash = hashlib.sha256(json.dumps(tree, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    event.update(wire_bytes_deterministic=True, wire_bytes_sha256=hashlib.sha256(wire).hexdigest())
    event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = tree_hash
    return event


def tag_decode_rejection_code(allocation: str) -> str:
    return 'tag_reserved' if allocation == 'reserved' else 'tag_not_allocated'


def rejected_input_event(cell: dict, input_sha256: str, index: int, *, allocation: str) -> dict:
    event = invoked_event(cell, index)
    event.update(decode_outcome='rejected',
                 decode_rejection_code=tag_decode_rejection_code(allocation),
                 decode_input_sha256=input_sha256)
    return event


def rejected_tag_event(cell: dict, wire: bytes, index: int, *, allocation: str) -> dict:
    return rejected_input_event(cell, hashlib.sha256(wire).hexdigest(), index,
                                allocation=allocation)


def run(output: Path, binary: Path | None = None) -> int:
    provenance = verify_installed_artifact()
    library = INSTALL / 'lib/libkmipkit_ffi.so'
    abi = InstalledAbi(library)
    catalog_path = ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json'
    manifest_path = ROOT / 'interop/shared/coverage/route-capabilities.json'
    catalog, manifest = json.loads(catalog_path.read_text()), json.loads(manifest_path.read_text())
    from probe_route_artifact import probe
    availability = probe(manifest, 'c', INSTALL, provenance)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.with_suffix('.artifact-probe.json').write_text(json.dumps(availability, indent=2) + '\n')
    targets = {target['target_id']: target for target in catalog['normative_targets']}
    cases = {case['case_id']: case for case in catalog['test_cases']}
    tags = catalog_tag_values(catalog)
    events, pending = [], []
    from route_ttlv import TreeAbi, digest, encode
    tree_abi = TreeAbi(library)
    boundaries = json.loads((ROOT / 'interop/shared/vectors/ttlv-boundaries.json').read_text())
    boundary_vectors = {int(vector['type_code'], 16): vector for vector in boundaries['vectors']}
    branch_evidence = []
    from route_values import ValueCases
    value_cases = ValueCases(library, catalog)
    value_cache = {}
    from route_attributes import AttributeCases
    attribute_cases = AttributeCases(library, catalog)
    attribute_cache = {}
    from c_route_models import ModelCases
    model_cases = ModelCases(library, catalog)
    generic_cache = {}
    for index, cell in enumerate(iter_cells(catalog, manifest, languages=('c',))):
        target = targets[cell['target_id']]
        fixture = scalar_fixture(target, tags, catalog, cell['protocol_version'])
        case = cases[cell['case_id']]
        tag_target_case = (case.get('requires', {}).get('enum_type') == 'Tag'
                           and target['kind'] in {'tag', 'tag_range'})
        scalar_target_case = uses_scalar_target_fixture(cell, case, target)
        item_type_target_case = (
            target['kind'] == 'enumeration_value'
            and 'KMIPKIT-ELEM-ENUMERATION-ITEM-TYPE'
            in target.get('parent_element_ids', [])
        )
        if (cell['transport'] is None and not tag_target_case
                and (any(key in case.get('requires', {}) for key in ('enum_type', 'bitmask_type'))
                     or item_type_target_case)):
            value_key = (case['case_id'], cell['protocol_version'])
            if value_key not in value_cache:
                value_cache[value_key] = value_cases.execute(
                    case, cell['protocol_version'], target if item_type_target_case else None)
            result = value_cache[value_key]
            if result is not None and cell['public_api_symbol'] in result['invoked_symbols']:
                event = cell_fields(cell)
                event.update(status='passed', invocation_count=1, invocation_id=f'c-values-{index}')
                event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = result['tree_sha256']
                events.append(event)
                branch_evidence.append({**cell_fields(cell), **{key: result[key] for key in
                    ('sample_count', 'known_values_checked', 'unknown_value_preserved', 'combination_count', 'tree_sha256') if key in result}})
                continue
        if not tag_target_case and should_execute_attribute_case(cell, case, target):
            attribute_key = (case['case_id'], cell['target_id'], cell['protocol_version'])
            if attribute_key not in attribute_cache:
                attribute_cache[attribute_key] = attribute_cases.execute(target, cell['protocol_version'])
            result = attribute_cache[attribute_key]
            if result is not None and cell['public_api_symbol'] in result['invoked_symbols']:
                event = cell_fields(cell)
                event.update(status='passed', invocation_count=1, invocation_id=f'c-attribute-{index}')
                event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = result['tree_sha256']
                events.append(event)
                branch_evidence.append({**cell_fields(cell), 'sample_count': result['sample_count'],
                                        'negative_checks': result['negative_checks'], 'tree_sha256': result['tree_sha256']})
                continue
        if cell['transport'] is None and not tag_target_case:
            result = model_cases.execute(target, cell['protocol_version'], cell['public_api_symbol'])
            if result is not None and cell['public_api_symbol'] in result['invoked_symbols']:
                event = cell_fields(cell)
                event.update(status='passed', invocation_count=1, invocation_id=f'c-model-{index}')
                event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = result['tree_sha256']
                events.append(event)
                branch_evidence.append({**cell_fields(cell), 'sample_count': result['sample_count'],
                                        'tree_sha256': result['tree_sha256']})
                continue
        if (cell['transport'] is None and target['kind'] == 'tag_range'
                and cell['action'] == 'decode' and cell['public_api_symbol'] == 'kmipkit_ttlv_decode'):
            result = execute_tag_range_case(tree_abi, target, catalog)
            if cell['public_api_symbol'] in result['invoked_symbols']:
                event = cell_fields(cell)
                if target.get('allocation') in {'reserved', 'unused'}:
                    event = rejected_input_event(cell, result['decode_input_sha256'], index,
                                                 allocation=target.get('allocation', 'unused'))
                else:
                    event.update(status='passed', invocation_count=1, invocation_id=f'c-tag-range-{index}')
                    event['response_tree_sha256'] = result['tree_sha256']
                events.append(event)
                branch_evidence.append({**cell_fields(cell), 'sample_count': result['sample_count'],
                                        'rejection_codes': result.get('rejection_codes'),
                                        'tree_sha256': result['tree_sha256']})
                continue
        if target['kind'] == 'data_type' and cell['transport'] is None:
            result = tree_abi.boundary_case(boundary_vectors[int(target['wire_value'], 16)])
            if cell['public_api_symbol'] in result['invoked_symbols']:
                event = cell_fields(cell)
                event.update(status='passed', invocation_count=1, invocation_id=f'c-boundary-{index}')
                event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = result['tree_sha256']
                events.append(event)
                branch_evidence.append({**cell_fields(cell), 'sample_count': result['sample_count'],
                                        'branches': result['branches'], 'tree_sha256': result['tree_sha256']})
                continue
        if (cell['transport'] is None and fixture is not None and scalar_target_case
                and (target['kind'] != 'tag' or target.get('allocation') != 'reserved' or
                     (cell['public_api_symbol'] in
                      {'kmipkit_ttlv_raw_tag_create', 'kmipkit_ttlv_raw_tag_try_checked', 'kmipkit_ttlv_decode'}))
                and cell['public_api_symbol'] in LOCAL_SYMBOLS):
            try:
                if target['kind'] == 'tag' and target.get('allocation') == 'reserved':
                    wire = abi.execute_reserved_tag(cell['public_api_symbol'], fixture[0])
                    if cell['action'] == 'decode':
                        event = rejected_tag_event(cell, wire, index,
                                                   allocation=target.get('allocation', 'unused'))
                    else:
                        event = event_from_cell(cell, wire, index)
                    events.append(event)
                else:
                    wire = abi.execute(cell['public_api_symbol'], fixture)
                    events.append(event_from_cell(cell, wire, index))
            except ValueError:
                if cell['public_api_symbol'] not in abi.called:
                    pending.append(pending_cell(cell, 'fixture_preparation_failed'))
                    continue
                event = cell_fields(cell)
                event.update(status='failed', failure_code='installed_abi_call_failed',
                             invocation_id=f'c-abi-{index}', invocation_count=1)
                events.append(event)
        elif (cell['transport'] is None and cell['public_api_symbol'] in LOCAL_SYMBOLS
                and target['kind'] != 'data_type'):
            generic_key = (cell['case_id'], cell['target_id'], cell['protocol_version'])
            if generic_key not in generic_cache:
                trees = generic_tree_fixtures(target, catalog, tags, cell['protocol_version'])
                if trees:
                    try:
                        generic_cache[generic_key] = [tree_abi.roundtrip(encode(tree)) for tree in trees]
                    except ValueError:
                        if cell['public_api_symbol'] in tree_abi.called:
                            raise
                        generic_cache[generic_key] = None
                else:
                    generic_cache[generic_key] = None
            results = generic_cache[generic_key]
            if results and all(cell['public_api_symbol'] in result['invoked_symbols'] for result in results):
                tree_hash = digest([result['normalized_tree'] for result in results])
                event = cell_fields(cell)
                event.update(status='passed', invocation_count=1, invocation_id=f'c-generic-{index}')
                event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = tree_hash
                events.append(event)
                branch_evidence.append({**cell_fields(cell), 'sample_count': len(results), 'tree_sha256': tree_hash})
                continue
            pending.append(pending_cell(cell, pending_reason(cell)))
        else:
            pending.append(pending_cell(cell, pending_reason(cell)))
    output.parent.mkdir(parents=True, exist_ok=True)
    output.with_suffix('.branches.json').write_text(json.dumps(branch_evidence, indent=2) + '\n')
    if binary is not None:
        from route_peer import execute_network_cells
        network_events, invoked_keys, peer_checks = execute_network_cells(binary, catalog, manifest)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.with_suffix(".peer-checks.json").write_text(json.dumps(peer_checks, indent=2) + "\n")
        events.extend(network_events)
        pending = [cell for cell in pending if _key(cell) not in invoked_keys]
    report = {'schema_version': '1.0', 'run_id': f'c-route-{uuid.uuid4()}', 'language': 'c',
              'server': {'id': 'controlled-c-peer', 'product': 'KMIPKit controlled C route fixture',
                         'version': 'fixture-2', 'configuration_fingerprint': hashlib.sha256(b'controlled-c-route-fixture-v2').hexdigest()},
              'library_commit': PIN, 'artifact_sha256': sha256(library),
              'catalog_sha256': sha256(catalog_path), 'route_manifest_sha256': sha256(manifest_path),
              'protocol_versions': ['2.0', '2.1'],
              'network_combinations': [{'encoding': encoding, 'transport': transport}
                                       for encoding in ('TTLV', 'JSON', 'XML') for transport in ('raw_tls', 'https')],
              'events': events}
    output.parent.mkdir(parents=True, exist_ok=True)
    from jsonschema import Draft202012Validator
    schema = json.loads((ROOT / 'interop/shared/schemas/route-execution-evidence.schema.json').read_text())
    Draft202012Validator(schema).validate(report)
    output.write_text(json.dumps(report, indent=2) + '\n')
    gaps_path = output.with_suffix('.gaps.json')
    gaps_path.write_text(json.dumps({'expected_cells': len(events) + len(pending),
        'invoked_cells': len(events), 'failed_cells': sum(e['status'] != 'passed' for e in events),
        'pending_cells': len(pending), 'pending': pending}, indent=2) + '\n')
    print(json.dumps({'evidence': str(output), 'invoked_cells': len(events), 'pending_cells': len(pending)}))
    try:
        require_complete(events, pending)
        validate_collection([report], catalog, manifest, catalog_sha256=report['catalog_sha256'],
                            route_manifest_sha256=report['route_manifest_sha256'],
                            expected_artifact_sha256=report['artifact_sha256'])
    except ValueError as error:
        print(f'C route matrix incomplete: {error}', file=sys.stderr)
        return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path)
    args = parser.parse_args()
    return run(resolve_output_path(args.output), args.binary)


if __name__ == '__main__':
    raise SystemExit(main())
