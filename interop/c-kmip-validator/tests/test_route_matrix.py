"""Matrix coverage must be backed by actual installed ABI calls (0092 FR-002/009)."""
from pathlib import Path
import importlib.util
import hashlib
import json
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parents[1] / 'tools'
sys.path.insert(0, str(TOOLS))


def get_usage_allocation_identifier_variants_request(protocol_minor=1, second_identifier=None):
    def integer(tag, value, width=4):
        return (tag, 2 if width == 4 else 3, value.to_bytes(width, 'big', signed=True))

    def enumeration(tag, value):
        return (tag, 5, value.to_bytes(4, 'big'))

    def text(tag, value):
        return (tag, 7, value.encode())

    def batch_item(item_id, identifier):
        payload = [integer(0x420096, 1, width=8)]
        if identifier is not None:
            payload.insert(0, text(0x420094, identifier))
        return (0x42000F, 1, [
            enumeration(0x42005C, 17),
            (0x420093, 8, item_id),
            (0x420079, 1, payload),
        ])

    locate_item = (0x42000F, 1, [
        enumeration(0x42005C, 8),
        (0x420093, 8, b'usage-locate'),
        (0x420079, 1, [integer(0x42004F, 3)]),
    ])

    return (0x420078, 1, [
        (0x420077, 1, [
            (0x420069, 1, [integer(0x42006A, 2), integer(0x42006B, protocol_minor)]),
            integer(0x42000D, 3),
        ]),
        locate_item,
        batch_item(b'usage-explicit', 'fixture-key-1'),
        batch_item(b'usage-placeholder', second_identifier),
    ])


class RouteMatrixTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(importlib.util.find_spec("route_matrix"),
                             "C matrix executor is missing")
        import route_matrix
        self.runner = route_matrix

    def test_nonexistent_recipe_is_pending_and_never_library_unavailable(self):
        self.assertEqual(self.runner.pending_reason({'public_api_symbol': 'kmipkit_client_batch_item_sign'}),
                         'validator_recipe_pending')

    def test_request_message_tag_uses_a_complete_typed_request_fixture(self):
        """OASIS KMIP 2.0 §§8.4, 11.54; KMIP 2.1 §§8.4, 11.56 (0092-FR-003)."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-TAG-420078')
        tags = self.runner.catalog_tag_values(catalog)

        fixtures = self.runner.generic_tree_fixtures(target, catalog, tags, '2.0')

        self.assertIsNotNone(fixtures)
        request = fixtures[0]
        self.assertEqual((request[0], request[1]), (tags['Request Message'], 1))
        self.assertEqual([item[0] for item in request[2]],
                         [tags['Request Header'], tags['Batch Item']])
        result = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so').roundtrip(encode(request))
        self.assertIn('kmipkit_request_message_from_ttlv', result['invoked_symbols'])

    def test_request_message_field_uses_a_complete_typed_request_fixture(self):
        """OASIS KMIP 2.0 §8.4; KMIP 2.1 §8.4 (0092-FR-003)."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-MESSAGE-FIELD-8-1-REQUEST-MESSAGE')
        tags = self.runner.catalog_tag_values(catalog)

        fixtures = self.runner.generic_tree_fixtures(target, catalog, tags, '2.0')

        self.assertIsNotNone(fixtures)
        request = fixtures[0]
        self.assertEqual((request[0], request[1]), (tags['Request Message'], 1))
        self.assertEqual([item[0] for item in request[2]],
                         [tags['Request Header'], tags['Batch Item']])
        result = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so').roundtrip(encode(request))
        self.assertIn('kmipkit_request_message_from_ttlv', result['invoked_symbols'])

    def test_request_message_parser_uses_the_structure_fixture_dispatch(self):
        from artifact_support import ROOT
        from coverage_matrix import iter_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-norm-tag-tag-420078')
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-TAG-420078')
        cell = next(item for item in iter_cells(catalog, manifest, languages=('c',))
                    if item['case_id'] == case['case_id'] and item['target_id'] == target['target_id']
                    and item['public_api_symbol'] == 'kmipkit_request_message_from_ttlv')

        self.assertTrue(hasattr(self.runner, 'uses_scalar_target_fixture'),
                        'fixture routing decision must be independently testable')
        self.assertFalse(self.runner.uses_scalar_target_fixture(cell, case, target))
        generic_cell = {**cell, 'public_api_symbol': 'kmipkit_ttlv_decode'}
        self.assertTrue(self.runner.uses_scalar_target_fixture(generic_cell, case, target))

    def test_request_header_field_uses_the_complete_typed_request_fixture(self):
        """OASIS KMIP 2.0/2.1 §8.2 defines Maximum Response Size in Request Header (0092-FR-003)."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-MAXIMUM-RESPONSE-SIZE')
        tags = self.runner.catalog_tag_values(catalog)

        fixtures = self.runner.generic_tree_fixtures(target, catalog, tags, '2.0')

        self.assertIsNotNone(fixtures)
        request = fixtures[0]
        self.assertEqual(request[0], tags['Request Message'])
        header = next(item for item in request[2] if item[0] == tags['Request Header'])
        maximum_response_size = next(item for item in header[2]
                                     if item[0] == tags['Maximum Response Size'])
        self.assertEqual((maximum_response_size[1], maximum_response_size[2]), (2, (1024).to_bytes(4, 'big')))
        result = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so').roundtrip(encode(request))
        self.assertIn('kmipkit_request_message_from_ttlv', result['invoked_symbols'])

    def test_optional_request_header_fields_use_typed_request_fixtures(self):
        """OASIS KMIP 2.0/2.1 §8.2 defines optional Request Header branches (0092-FR-003)."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        tags = self.runner.catalog_tag_values(catalog)
        values = {
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ASYNCHRONOUS-INDICATOR':
                ('Asynchronous Indicator', 5, (1).to_bytes(4, 'big')),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ATTESTATION-CAPABLE-INDICATOR':
                ('Attestation Capable Indicator', 6, bytes(8)),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ATTESTATION-TYPE':
                ('Attestation Type', 5, (1).to_bytes(4, 'big')),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ERROR-CONTINUATION-OPTION':
                ('Batch Error Continuation Option', 5, (1).to_bytes(4, 'big')),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ORDER-OPTION':
                ('Batch Order Option', 6, (1).to_bytes(8, 'big')),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-CLIENT-CORRELATION-VALUE':
                ('Client Correlation Value', 7, b'client-correlation'),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-SERVER-CORRELATION-VALUE':
                ('Server Correlation Value', 7, b'server-correlation'),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-TIME-STAMP':
                ('Time Stamp', 9, (1760000000).to_bytes(8, 'big', signed=True)),
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-10-SERVER-CORRELATION-VALUE':
                ('Server Correlation Value', 7, b'server-correlation'),
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-12-MAXIMUM-RESPONSE-SIZE':
                ('Maximum Response Size', 2, (1024).to_bytes(4, 'big')),
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-2-ASYNCHRONOUS-INDICATOR':
                ('Asynchronous Indicator', 5, (1).to_bytes(4, 'big')),
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-3-ATTESTATION-CAPABLE-INDICATOR':
                ('Attestation Capable Indicator', 6, bytes(8)),
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP':
                ('Time Stamp', 9, (1760000000).to_bytes(8, 'big', signed=True)),
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-9-CLIENT-CORRELATION-VALUE':
                ('Client Correlation Value', 7, b'client-correlation'),
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-6-BATCH-ERROR-CONTINUATION-OPTION':
                ('Batch Error Continuation Option', 5, (1).to_bytes(4, 'big')),
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-8-BATCH-ORDER-OPTION':
                ('Batch Order Option', 6, (1).to_bytes(8, 'big')),
        }
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        for target_id, (name, kind, value) in values.items():
            target = next(item for item in catalog['normative_targets']
                          if item['target_id'] == target_id)
            for version in ('2.0', '2.1'):
                with self.subTest(target_id=target_id, protocol_version=version):
                    fixtures = self.runner.generic_tree_fixtures(target, catalog, tags, version)
                    self.assertIsNotNone(fixtures)
                    request = fixtures[0]
                    self.assertEqual(request[0], tags['Request Message'])
                    header = next(item for item in request[2] if item[0] == tags['Request Header'])
                    field = next(item for item in header[2] if item[0] == tags[name])
                    self.assertEqual((field[1], field[2]), (kind, value))
                    result = abi.roundtrip(encode(request))
                    self.assertIn('kmipkit_request_message_from_ttlv', result['invoked_symbols'])

    def test_request_message_context_fields_use_complete_parser_fixtures(self):
        """OASIS KMIP 2.0/2.1 §§8.1–8.3 define these Request Message branches (0092-FR-003)."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        tags = self.runner.catalog_tag_values(catalog)
        target_ids = {
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
        }
        targets = {target['target_id']: target for target in catalog['normative_targets']
                   if target['target_id'] in target_ids}
        self.assertEqual(set(targets), target_ids)
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        for target_id, target in targets.items():
            target_tag = tags.get(target['name'])
            self.assertIsNotNone(target_tag)
            for version in ('2.0', '2.1'):
                with self.subTest(target_id=target_id, protocol_version=version):
                    fixtures = self.runner.generic_tree_fixtures(target, catalog, tags, version)
                    self.assertIsNotNone(fixtures)
                    request = fixtures[0]
                    self.assertEqual((request[0], request[1]), (tags['Request Message'], 1))
                    pending = list(request[2])
                    found = False
                    while pending:
                        tag, kind, value = pending.pop()
                        if tag == target_tag:
                            found = True
                            break
                        if kind == 1:
                            pending.extend(value)
                    self.assertTrue(found, f'{target_id} is absent from the request tree')
                    result = abi.roundtrip(encode(request))
                    self.assertIn('kmipkit_request_message_from_ttlv', result['invoked_symbols'])

    def test_authentication_fields_use_a_complete_typed_request_fixture(self):
        """OASIS KMIP 2.0/2.1 §§8.2, 9.4 define Authentication and Credential (0092-FR-003)."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        tags = self.runner.catalog_tag_values(catalog)
        targets = {
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-AUTHENTICATION': 'Authentication',
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-4-AUTHENTICATION': 'Authentication',
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-4-CREDENTIAL-MAY-BE-REPEATED': 'Credential',
        }
        target_by_id = {target['target_id']: target for target in catalog['normative_targets']
                        if target['target_id'] in targets}
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        for target_id, tag_name in targets.items():
            for version in ('2.0', '2.1'):
                with self.subTest(target_id=target_id, protocol_version=version):
                    fixtures = self.runner.generic_tree_fixtures(
                        target_by_id[target_id], catalog, tags, version)
                    self.assertIsNotNone(fixtures)
                    fixture = fixtures[0]
                    self.assertEqual((fixture[0], fixture[1]), (tags['Request Message'], 1))
                    pending = list(fixture[2])
                    found = False
                    while pending:
                        tag, kind, value = pending.pop()
                        if tag == tags[tag_name]:
                            found = True
                            break
                        if kind == 1:
                            pending.extend(value)
                    self.assertTrue(found, f'{target_id} is absent from the request tree')
                    result = abi.roundtrip(encode(fixture))
                    self.assertIn('kmipkit_request_message_from_ttlv', result['invoked_symbols'])

    def test_message_extension_fields_use_a_complete_typed_request_fixture(self):
        """OASIS KMIP 2.0/2.1 §9.13 defines Message Extension fields (0092-FR-003)."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        tags = self.runner.catalog_tag_values(catalog)
        targets = {
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-3-MESSAGE-EXTENSION': 'Message Extension',
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-13-CRITICALITY-INDICATOR': 'Criticality Indicator',
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION': 'Message Extension',
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-13-VENDOR-EXTENSION': 'Vendor Extension',
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-13-VENDOR-IDENTIFICATION': 'Vendor Identification',
        }
        target_by_id = {target['target_id']: target for target in catalog['normative_targets']
                        if target['target_id'] in targets}
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        for target_id, tag_name in targets.items():
            for version in ('2.0', '2.1'):
                with self.subTest(target_id=target_id, protocol_version=version):
                    fixtures = self.runner.generic_tree_fixtures(
                        target_by_id[target_id], catalog, tags, version)
                    self.assertIsNotNone(fixtures)
                    fixture = fixtures[0]
                    self.assertEqual((fixture[0], fixture[1]), (tags['Request Message'], 1))
                    pending = list(fixture[2])
                    found = False
                    while pending:
                        tag, kind, value = pending.pop()
                        if tag == tags[tag_name]:
                            found = True
                            break
                        if kind == 1:
                            pending.extend(value)
                    self.assertTrue(found, f'{target_id} is absent from the request tree')
                    result = abi.roundtrip(encode(fixture))
                    self.assertIn('kmipkit_request_message_from_ttlv', result['invoked_symbols'])

    def test_response_message_root_uses_a_complete_typed_response_fixture(self):
        """OASIS KMIP 2.0/2.1 §8.4 defines Response Message shape (0092-FR-003)."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        tags = self.runner.catalog_tag_values(catalog)
        target_ids = {
            'KMIPKIT-ELEM-TAG-42007B',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-4-RESPONSE-MESSAGE',
        }
        targets = {target['target_id']: target for target in catalog['normative_targets']
                   if target['target_id'] in target_ids}
        self.assertEqual(set(targets), target_ids)
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        for target_id, target in targets.items():
            for version in ('2.0', '2.1'):
                with self.subTest(target_id=target_id, protocol_version=version):
                    fixtures = self.runner.generic_tree_fixtures(target, catalog, tags, version)
                    self.assertIsNotNone(fixtures)
                    response = fixtures[0]
                    self.assertEqual((response[0], response[1]), (tags['Response Message'], 1))
                    result = abi.roundtrip(encode(response))
                    self.assertIn('kmipkit_response_message_from_ttlv', result['invoked_symbols'])

    def test_response_message_context_fields_use_complete_parser_fixtures(self):
        """OASIS KMIP 2.0/2.1 §§8.4–8.6 define the response message tree (0092-FR-003)."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        tags = self.runner.catalog_tag_values(catalog)
        target_ids = {
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-4-BATCH-ITEM',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-4-RESPONSE-HEADER',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-5-BATCH-COUNT',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-5-PROTOCOL-VERSION',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-5-RESPONSE-HEADER',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-5-TIME-STAMP',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-OPERATION',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESPONSE-PAYLOAD',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESULT-STATUS',
        }
        targets = {target['target_id']: target for target in catalog['normative_targets']
                   if target['target_id'] in target_ids}
        self.assertEqual(set(targets), target_ids)
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        for target_id, target in targets.items():
            target_tag = tags.get(target['name'])
            self.assertIsNotNone(target_tag)
            for version in ('2.0', '2.1'):
                with self.subTest(target_id=target_id, protocol_version=version):
                    fixtures = self.runner.generic_tree_fixtures(target, catalog, tags, version)
                    self.assertIsNotNone(fixtures)
                    response = fixtures[0]
                    self.assertEqual((response[0], response[1]), (tags['Response Message'], 1))
                    pending = list(response[2])
                    found = False
                    while pending:
                        tag, kind, value = pending.pop()
                        if tag == target_tag:
                            found = True
                            break
                        if kind == 1:
                            pending.extend(value)
                    self.assertTrue(found, f'{target_id} is absent from the response tree')
                    result = abi.roundtrip(encode(response))
                    self.assertIn('kmipkit_response_message_from_ttlv', result['invoked_symbols'])

    def test_response_message_optional_fields_use_complete_parser_fixtures(self):
        """OASIS KMIP 2.0/2.1 §8.6 Table 399 and §9.17 define response fields (0092-FR-003)."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        tags = self.runner.catalog_tag_values(catalog)
        values = {
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-5-ATTESTATION-TYPE': ('Attestation Type', 5, (1).to_bytes(4, 'big')),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-5-CLIENT-CORRELATION-VALUE':
                ('Client Correlation Value', 7, b'client-correlation'),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-5-NONCE': ('Nonce', 1, None),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-5-SERVER-CORRELATION-VALUE':
                ('Server Correlation Value', 7, b'server-correlation'),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-5-SERVER-HASHED-PASSWORD':
                ('Server Hashed Password', 8, b'fixture-hashed-password'),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-UNIQUE-BATCH-ITEM-ID':
                ('Unique Batch Item ID', 8, b'route-response-batch-item'),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESULT-MESSAGE':
                ('Result Message', 7, b'fixture-result-message'),
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-MESSAGE-EXTENSION': ('Message Extension', 1, None),
        }
        targets = {target['target_id']: target for target in catalog['normative_targets']
                   if target['target_id'] in values}
        self.assertEqual(set(targets), set(values))
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        for target_id, (name, kind, expected_value) in values.items():
            target = targets[target_id]
            for version in ('2.0', '2.1'):
                with self.subTest(target_id=target_id, protocol_version=version):
                    fixtures = self.runner.generic_tree_fixtures(target, catalog, tags, version)
                    self.assertIsNotNone(fixtures)
                    response = fixtures[0]
                    self.assertEqual((response[0], response[1]), (tags['Response Message'], 1))
                    pending = list(response[2])
                    actual = None
                    while pending:
                        tag, item_kind, value = pending.pop()
                        if tag == tags[name]:
                            actual = (item_kind, value)
                            break
                        if item_kind == 1:
                            pending.extend(value)
                    self.assertIsNotNone(actual, f'{target_id} is absent from the response tree')
                    self.assertEqual(actual[0], kind)
                    if expected_value is not None:
                        self.assertEqual(actual[1], expected_value)
                    if name == 'Result Message':
                        batch_fields = {field[0]: field for field in response[2][1][2]}
                        failure = self.runner.catalog_enum_value(
                            catalog, 'Result Status', 'Operation Failed', version)
                        general_failure = self.runner.catalog_enum_value(
                            catalog, 'Result Reason', 'General Failure', version)
                        self.assertEqual(batch_fields[tags['Result Status']][2],
                                         failure.to_bytes(4, 'big'))
                        self.assertEqual(batch_fields[tags['Result Reason']][2],
                                         general_failure.to_bytes(4, 'big'))
                        self.assertNotIn(tags['Response Payload'], batch_fields)
                    result = abi.roundtrip(encode(response))
                    self.assertIn('kmipkit_response_message_from_ttlv', result['invoked_symbols'])

    def test_get_usage_allocation_has_an_executable_controlled_peer_recipe(self):
        import route_peer
        from artifact_support import ROOT

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-op-get-usage-allocation')
        recipe = route_peer.recipe_for_case(case, None)

        self.assertIsNotNone(recipe)
        self.assertEqual(recipe[0], (1, 17, 19, 20))
        self.assertEqual(recipe[1], 'get_usage_allocation')

    def test_cancel_has_a_typed_peer_recipe_and_echoes_the_exact_correlation(self):
        """OASIS KMIP 2.1 §6.1.5 Tables 176–177 require Cancel correlation echo and result."""
        import route_peer
        from artifact_support import ROOT

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases'] if item['case_id'] == 'test-op-cancel')
        correlation = b'fixture-cancel-correlation'

        self.assertEqual(route_peer.recipe_for_case(case, None), ((25,), 'cancel_async'))
        pending = route_peer.parse(route_peer.pending_create_response(correlation, 1))
        pending_item = next(child for child in pending[2] if child[0] == route_peer.BATCH_ITEM_TAG)
        self.assertEqual(route_peer.numbers(pending_item, 0x42005C), [1])
        self.assertEqual(route_peer.numbers(pending_item, 0x42007F), [2])
        self.assertEqual(route_peer.byte_string_values(pending_item, 0x420006), [correlation])
        response = route_peer.parse(route_peer.cancel_response(correlation, 1, 1))
        payload = next(child for child in response[2][1][2] if child[0] == route_peer.RESPONSE_PAYLOAD_TAG)
        self.assertEqual(payload[2], [
            (route_peer.Codec(catalog).tag('Asynchronous Correlation Value'), 8, correlation),
            (route_peer.Codec(catalog).tag('Cancellation Result'), 5, (1).to_bytes(4, 'big')),
        ])

    def test_cancel_executes_all_installed_encoding_and_transport_routes(self):
        """OASIS KMIP 2.0/2.1 §6.1.5 Cancel runs once for each configured wire route."""
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == 'test-op-cancel']}

        events, keys, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertEqual(len(events), 38)
        self.assertEqual(len(keys), 38)
        self.assertEqual(sum(event['public_api_symbol'] ==
                             'kmipkit_client_async_operation_outcome_cancel_response_v1'
                             for event in events), 2)

    def test_each_async_cancel_result_case_uses_the_pending_cancel_peer(self):
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        cases = [case for case in catalog['test_cases']
                 if case['case_id'].startswith('test-async-cancel-result-')]

        self.assertEqual(len(cases), 5)
        for case in cases:
            with self.subTest(case_id=case['case_id']):
                self.assertEqual(route_peer.recipe_for_case(case, None), ((25,), 'cancel_async'))

    def test_async_cancel_result_cases_execute_the_catalogued_value_over_all_routes(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected_cases = [case for case in catalog['test_cases']
                          if case['case_id'].startswith('test-async-cancel-result-')]
        selected = {**catalog, 'test_cases': selected_cases}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 60)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        for case in selected_cases:
            case_events = [event for event in events if event['case_id'] == case['case_id']]
            for symbol, action, count in (
                    ('kmipkit_client_batch_item_create', 'construct', 12),
                    ('kmipkit_client_batch_item_create', 'send', 12),
                    ('kmipkit_client_batch_item_create', 'decode', 12),
                    ('kmipkit_client_batch_item_response_create', 'decode', 2)):
                with self.subTest(case_id=case['case_id'], symbol=symbol, action=action):
                    self.assertEqual(
                        sum(event['public_api_symbol'] == symbol and event['action'] == action
                            for event in case_events),
                        count)
        self.assertEqual(sum(event['public_api_symbol'] ==
                             'kmipkit_client_async_operation_outcome_cancel_response_v1'
                             for event in events), 30)

    def test_async_cancel_poll_race_interleaves_poll_before_cancel(self):
        """OASIS KMIP 2.1 §§6.1.5, 6.1.38 define the interleaved requests and responses."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-cancel-vs-poll-race')

        self.assertEqual(route_peer.recipe_for_case(case, None), ((25,), 'cancel_poll_race'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        from coverage_matrix import iter_cells
        from validate_route_evidence import _key
        covered_local_cells = {
            _key(cell) for cell in iter_cells(selected, manifest, languages=('c',))
            if cell['transport'] is None and cell['public_api_symbol'] in {
                'kmipkit_cancel_request_create',
                'kmipkit_cancel_request_to_ttlv_payload',
            }
        }
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            covered_keys=covered_local_cells)

        self.assertEqual(len(checks), 6)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertEqual(sum(event['public_api_symbol'] == 'kmipkit_client_execute_poll' and
                             event['action'] == 'send' for event in events), 6)
        self.assertEqual(sum(event['public_api_symbol'] == 'kmipkit_client_execute_cancel' and
                             event['action'] == 'send' for event in events), 12)
        for symbol, action, count in (
                ('kmipkit_poll_request_create', 'construct', 1),
                ('kmipkit_poll_request_to_ttlv_payload', 'encode', 1),
                ('kmipkit_cancel_request_create', 'construct', 0),
                ('kmipkit_cancel_request_to_ttlv_payload', 'encode', 0),
                ('kmipkit_client_async_operation_outcome_poll_response_v1', 'decode', 1),
                ('kmipkit_client_async_operation_outcome_cancel_response_v1', 'decode', 1),
                ('kmipkit_client_batch_item_response_create', 'decode', 1)):
            with self.subTest(symbol=symbol, action=action):
                self.assertEqual(sum(event['public_api_symbol'] == symbol and
                                     event['action'] == action for event in events), count)
        self.assertTrue(all(_key(event) not in covered_local_cells for event in events))
        self.assertTrue(all(check.get('peer_async_interleaving_operations') == [1, 26, 25] and
                            check.get('peer_async_response_statuses') == [2, 2, 0] and
                            check.get('peer_async_terminal_state_count') == 1
                            for check in checks), checks)

    def test_async_mandatory_create_returns_pending_with_correlation(self):
        """OASIS KMIP 2.0/2.1 §§8.2–8.3, 8.6, 9.1–9.2, 11.3; Create is §6.1.8 Tables 171 (2.0) and 186 (2.1)."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-indicator-mandatory-pending')

        self.assertEqual(route_peer.recipe_for_case(case, None), ((1,), 'async_create_pending'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        for symbol, action, target_id, count in (
                ('kmipkit_client_batch_item_create', 'construct', 'KMIPKIT-ELEM-OP-C2S-CREATE', 12),
                ('kmipkit_client_batch_item_create', 'send', 'KMIPKIT-ELEM-OP-C2S-CREATE', 12),
                ('kmipkit_client_batch_item_create', 'decode', 'KMIPKIT-ELEM-OP-C2S-CREATE', 12),
                ('kmipkit_client_batch_item_response_create', 'decode', 'KMIPKIT-ELEM-OP-C2S-CREATE', 2)):
            with self.subTest(symbol=symbol, action=action):
                self.assertEqual(sum(event['public_api_symbol'] == symbol and
                                     event['action'] == action and
                                     event['target_id'] == target_id for event in events), count)
        self.assertTrue(all(check.get('peer_async_response_statuses') == [2] and
                            check.get('peer_async_correlation_match') is True
                            and check.get('peer_async_empty_response_payload') is True
                            for check in checks), checks)

    def test_async_indicator_omitted_defaults_to_prohibited_and_completes(self):
        """OASIS KMIP 2.0/2.1 §§8.2–8.3, 8.6, 9.1–9.2, 11.3; Create: 2.0 §6.1.8 Table 171; 2.1 §6.1.8 Table 186."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-indicator-omitted-default')

        self.assertEqual(route_peer.recipe_for_case(case, None), ((1,), 'async_create_sync_default'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        for symbol, action, target_id, count in (
                ('kmipkit_client_batch_item_create', 'construct', 'KMIPKIT-ELEM-OP-C2S-CREATE', 12),
                ('kmipkit_client_batch_item_create', 'send', 'KMIPKIT-ELEM-OP-C2S-CREATE', 12),
                ('kmipkit_client_batch_item_create', 'decode', 'KMIPKIT-ELEM-OP-C2S-CREATE', 12),
                ('kmipkit_client_batch_item_response_create', 'decode', 'KMIPKIT-ELEM-OP-C2S-CREATE', 2)):
            with self.subTest(symbol=symbol, action=action):
                self.assertEqual(sum(event['public_api_symbol'] == symbol and
                                     event['action'] == action and
                                     event['target_id'] == target_id for event in events), count)
        self.assertTrue(all(check.get('peer_async_response_statuses') == [0] and
                            check.get('peer_async_indicator_omitted') is True and
                            check.get('peer_async_correlation_absent') is True
                            for check in checks), checks)

    def test_async_optional_create_may_complete_without_poll(self):
        """OASIS KMIP 2.0/2.1 §§8.2–8.3, 8.6, 9.1–9.2, 11.3; Create: 2.0 §6.1.8 Table 171; 2.1 §6.1.8 Table 186."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-indicator-optional-completes-or-pends')

        self.assertEqual(route_peer.recipe_for_case(case, None), ((1,), 'async_create_optional_complete'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertTrue(all(check.get('peer_async_response_statuses') == [0] and
                            check.get('peer_async_indicator_optional') is True and
                            check.get('peer_async_correlation_absent') is True and
                            check.get('peer_async_no_poll_after_completion') is True and
                            'kmipkit_client_batch_set_asynchronous_indicator' in
                            check['public_api_success_symbols']
                            for check in checks), checks)
        self.assertFalse(any(event.get('workflow_operation') == 'Poll' for event in events))

    def test_async_prohibited_create_completes_synchronously(self):
        """OASIS KMIP 2.0/2.1 §§8.2–8.3, 8.6, 9.1–9.2, 11.3; Create: 2.0 §6.1.8 Table 171; 2.1 §6.1.8 Table 186."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-indicator-prohibited-synchronous')

        self.assertEqual(route_peer.recipe_for_case(case, None), ((1,), 'async_create_prohibited_sync'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertTrue(all(check.get('peer_async_response_statuses') == [0] and
                            check.get('peer_async_indicator_prohibited') is True and
                            check.get('peer_async_correlation_absent') is True and
                            check.get('peer_async_no_poll_after_completion') is True and
                            'kmipkit_client_batch_set_asynchronous_indicator' in
                            check['public_api_success_symbols']
                            for check in checks), checks)
        self.assertFalse(any(event.get('workflow_operation') == 'Poll' for event in events))

    def test_async_lifecycle_queries_and_polls_the_correlated_create(self):
        """OASIS KMIP 2.1 §§6.1.8, 6.1.38, 6.1.41, 8.2–8.3, 8.6, 9.1–9.2."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-lifecycle-poll')

        self.assertEqual(
            route_peer.recipe_for_case(case, None),
            ((1, 57, 26, 20), 'async_create_poll_lifecycle'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 6)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertTrue(all(check.get('peer_async_interleaving_operations') == [1, 57, 26, 20] and
                            check.get('peer_async_response_statuses') == [2, 0, 0, 0] and
                            check.get('peer_async_query_correlation_match') is True and
                            check.get('peer_async_poll_correlation_match') is True and
                            check.get('peer_async_poll_completed') is True and
                            check.get('peer_cleanup_operations') == [20] and
                            check.get('peer_final_state_matches_setup') is True
                            for check in checks), checks)
        for symbol, action, count in (
                ('kmipkit_client_execute_query_async_requests', 'send', 6),
                ('kmipkit_client_execute_poll', 'send', 6),
                ('kmipkit_client_async_operation_outcome_query_async_requests_response_v1', 'decode', 1),
                ('kmipkit_client_async_operation_outcome_poll_response_v1', 'decode', 1)):
            with self.subTest(symbol=symbol, action=action):
                self.assertEqual(sum(event['public_api_symbol'] == symbol and
                                     event['action'] == action for event in events), count)

    def test_async_query_filters_intersect_only_outstanding_requests(self):
        """OASIS KMIP 2.1 §6.1.41 Tables 285–287 and §7.2 Table 353."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-query-outstanding-filter')

        self.assertEqual(
            route_peer.recipe_for_case(case, None),
            ((1, 57), 'async_query_outstanding_filter'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 6)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertTrue(all(check.get('peer_async_query_operation_filter') is True and
                            check.get('peer_async_query_correlation_filter') is True and
                            check.get('peer_async_query_filters_intersect') is True and
                            check.get('peer_async_query_excludes_completed') is True and
                            check.get('peer_async_query_excludes_other_operation') is True and
                            check.get('peer_async_query_result_is_exact') is True and
                            check.get('peer_final_state_matches_setup') is True
                            for check in checks), checks)
        self.assertEqual(sum(event['public_api_symbol'] ==
                             'kmipkit_client_execute_query_async_requests' and
                             event['action'] == 'send' for event in events), 6)
        self.assertEqual(sum(event['public_api_symbol'] ==
                             'kmipkit_client_batch_item_response_create' and
                             event['action'] == 'decode' for event in events), 1)

    def test_async_unknown_correlation_is_rejected_with_typed_reason(self):
        """OASIS KMIP 2.0 §6.1.36 Table 257 and KMIP 2.1 §6.1.38 Table 277."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-unknown-correlation-rejected')

        self.assertEqual(
            route_peer.recipe_for_case(case, None),
            ((26,), 'async_unknown_correlation_rejected'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertTrue(all(check.get('peer_async_unknown_correlation_request') is True and
                            check.get('peer_async_unknown_correlation_failure') is True and
                            check.get('peer_async_unknown_correlation_not_remapped') is True and
                            check.get('peer_final_state_matches_setup') is True and
                            check.get('consumer_result_status_code') == 1 and
                            check.get('consumer_result_reason_code') == 43 and
                            check.get('consumer_library_correctness') == 'passed'
                            for check in checks), checks)
        for symbol, action, count in (
                ('kmipkit_client_execute_poll', 'send', 12),
                ('kmipkit_client_execute_poll', 'decode', 12),
                ('kmipkit_client_async_operation_outcome_poll_response_v1', 'decode', 2)):
            with self.subTest(symbol=symbol, action=action):
                self.assertEqual(sum(event['public_api_symbol'] == symbol and
                                     event['action'] == action for event in events), count)
        self.assertEqual(sum(event['public_api_symbol'] ==
                             'kmipkit_poll_request_to_ttlv_payload' and
                             event['action'] == 'encode' for event in events), 2)
        self.assertEqual(sum(event['public_api_symbol'] ==
                             'kmipkit_poll_request_create' and
                             event['action'] == 'construct' for event in events), 2)

    def test_async_poll_and_cancel_responses_do_not_recurse(self):
        """OASIS KMIP 2.1 §§6.1.5 and 6.1.38 keep Poll and Cancel terminal."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-no-recursive-async-poll-cancel')

        self.assertEqual(
            route_peer.recipe_for_case(case, None),
            ((26, 25), 'async_poll_cancel_no_recursion'))

        cancel_response_target = next(item for item in catalog['normative_targets']
                                      if item['target_id'] ==
                                      'KMIPKIT-PAYLOAD-CLIENT-CANCEL-RESPONSE-21-002')
        tags = self.runner.catalog_tag_values(catalog)
        self.assertEqual(
            self.runner.generic_tree_fixtures(
                cancel_response_target, catalog, tags, '2.1', case=case),
            [(tags['Cancellation Result'], 5, (1).to_bytes(4, 'big'))])

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 6)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertTrue(all(check.get('peer_async_interleaving_operations') == [26, 25] and
                            check.get('peer_async_response_statuses') == [0, 0] and
                            check.get('peer_async_poll_no_correlation') is True and
                            check.get('peer_async_cancel_echo_matches') is True and
                            check.get('peer_async_indicator_absent') is True
                            for check in checks), checks)
        for symbol, action, count in (
                ('kmipkit_client_execute_cancel', 'send', 12),
                ('kmipkit_client_execute_poll', 'send', 6),
                ('kmipkit_client_async_operation_outcome_cancel_response_v1', 'decode', 2),
                ('kmipkit_client_async_operation_outcome_poll_response_v1', 'decode', 1)):
            with self.subTest(symbol=symbol, action=action):
                self.assertEqual(sum(event['public_api_symbol'] == symbol and
                                     event['action'] == action for event in events), count)
        self.assertEqual(sum(event['public_api_symbol'] == 'kmipkit_poll_request_create' and
                             event['action'] == 'construct' and event['transport'] is None
                             for event in events), 1)

    def test_async_poll_final_failure_preserves_the_original_create_failure(self):
        """OASIS KMIP 2.0/2.1 §§6.1.8 and 6.1.38 preserve a final Create failure."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-poll-final-operation-failure')

        self.assertEqual(
            route_peer.recipe_for_case(case, None),
            ((1, 26), 'async_poll_final_failure'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertTrue(all(check.get('peer_async_interleaving_operations') == [1, 26, 26] and
                            check.get('peer_async_response_statuses') == [2, 2, 1] and
                            check.get('peer_async_create_indicator_mandatory') is True and
                            check.get('peer_async_poll_correlations_match') is True and
                            check.get('peer_async_final_failure_shape') is True and
                            check.get('peer_final_state_matches_setup') is True
                            for check in checks), checks)
        self.assertEqual(sum(event['public_api_symbol'] == 'kmipkit_client_execute_poll' and
                             event['action'] == 'send' for event in events), 12)

    def test_async_repeated_pending_polls_keep_correlation_until_create_success(self):
        """OASIS KMIP 2.0/2.1 §6.1.38 keeps one correlation across Pending Polls."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-poll-repeated-pending-to-success')

        self.assertEqual(
            route_peer.recipe_for_case(case, None),
            ((1, 26), 'async_poll_repeated_pending_to_success'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertTrue(all(check.get('peer_async_interleaving_operations') == [1, 26, 26, 26, 20] and
                            check.get('peer_async_response_statuses') == [2, 2, 2, 0, 0] and
                            check.get('peer_async_create_indicator_mandatory') is True and
                            check.get('peer_async_poll_correlations_match') is True and
                            check.get('peer_async_final_poll_create_shape') is True and
                            check.get('peer_final_state_matches_setup') is True
                            for check in checks), checks)
        self.assertEqual(sum(event['public_api_symbol'] == 'kmipkit_client_execute_poll' and
                             event['action'] == 'send' for event in events), 12)

    def test_async_process_changes_state_seen_by_correlated_poll(self):
        """OASIS KMIP 2.1 §6.1.39 Tables 278–279 require correlated Process and Poll."""
        from artifact_support import ROOT
        import route_peer

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-async-process-then-poll-state')

        self.assertEqual(
            route_peer.recipe_for_case(case, None),
            ((1, 58, 26), 'async_process_then_poll_state'))

        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = route_peer.execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 6)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertTrue(all(check.get('peer_async_interleaving_operations') == [1, 58, 26, 20] and
                            check.get('peer_async_response_statuses') == [2, 0, 0, 0] and
                            check.get('peer_async_process_correlation_match') is True and
                            check.get('peer_async_poll_correlation_match') is True and
                            check.get('peer_async_process_is_synchronous') is True and
                            check.get('peer_async_final_poll_create_shape') is True and
                            check.get('peer_final_state_matches_setup') is True
                            for check in checks), checks)
        self.assertEqual(sum(event['public_api_symbol'] == 'kmipkit_client_execute_process' and
                             event['action'] == 'send' for event in events), 6)
        self.assertEqual(sum(event['public_api_symbol'] == 'kmipkit_client_execute_poll' and
                             event['action'] == 'send' for event in events), 6)

    def test_stop_on_first_failure_case_has_a_controlled_peer_recipe(self):
        import route_peer
        from artifact_support import ROOT

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-batch-stop-on-first-failure')

        self.assertEqual(
            route_peer.recipe_for_case(case, None),
            ((10, 10), 'batch_stop_on_first_failure'),
        )

    def test_stop_batch_peer_does_not_execute_items_after_first_failure(self):
        from route_peer import PeerState

        get = lambda identifier: (0x42000F, 1, [
            (0x42005C, 5, (10).to_bytes(4, 'big')),
            (0x420094, 7, identifier.encode()),
        ])
        request = (0x420078, 1, [get('missing-key'), get('fixture-key-1')])
        peer = PeerState(existing_object=True)

        evidence = peer.execute(request, stop_after_failure=True)

        self.assertEqual(evidence['statuses'], [1])
        self.assertEqual(evidence['reasons'], [55])
        self.assertEqual(evidence['effects'], ['object_not_found'])
        self.assertEqual(
            evidence['peer_observed_state_before_sha256'],
            evidence['peer_observed_state_after_sha256'],
        )

    def test_get_usage_allocation_peer_verifies_its_object_lifecycle(self):
        from controlled_kms_test import response
        from route_peer import parse, verify_lifecycle

        response_tree = parse(response((1, 17, 19, 20), 1))

        self.assertEqual(
            verify_lifecycle((1, 17, 19, 20), response_tree),
            ['created', 'usage_allocation_returned', 'revoked', 'destroyed'],
        )

    def test_get_usage_allocation_optional_identifier_cases_have_peer_recipes(self):
        import route_peer

        for case_id in (
            'test-norm-payload-client-get-usage-allocation-request-20-001',
            'test-norm-payload-client-get-usage-allocation-request-21-001',
        ):
            with self.subTest(case_id=case_id):
                self.assertEqual(
                    route_peer.RECIPES.get(case_id),
                    ((8, 17, 17), 'get_usage_allocation_uid_variants'),
                )

    def test_ping_v20_boundary_has_a_typed_pre_send_recipe(self):
        """KMIP 2.1 §6.1.36 defines Ping; KMIP 2.0 does not define the operation."""
        import route_peer
        from artifact_support import ROOT

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-v20-reject-ping')

        self.assertEqual(route_peer.recipe_for_case(case, None), ((59,), 'ping'))

    def test_ping_v20_boundary_records_typed_construction_and_zero_transport_calls(self):
        """KMIP 2.1 §6.1.36 Ping is validated locally and rejected before transport in KMIP 2.0."""
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case = next(item for item in catalog['test_cases']
                    if item['case_id'] == 'test-v20-reject-ping')

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c',
            {**catalog, 'test_cases': [case]}, manifest, protocol_versions=('2.0',))

        self.assertEqual(len(events), 12)
        self.assertEqual(len(checks), 6)
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertTrue(all(check['consumer_server_compatibility'] == 'unknown' for check in checks))
        self.assertTrue(all('kmipkit_client_batch_item_ping' in check['public_api_trace_symbols']
                            and 'kmipkit_client_execute' in check['public_api_trace_symbols']
                            for check in checks))
        send_events = [event for event in events if event['action'] == 'send']
        self.assertEqual(len(send_events), 6)
        self.assertTrue(all(event['request_delivery_state'] == 'not_sent'
                            and event['transport_invocation_count'] == 0 for event in send_events))

    def test_get_usage_allocation_recipe_validates_present_then_omitted_identifier(self):
        from route_peer import assert_recipe

        request = get_usage_allocation_identifier_variants_request()

        assert_recipe(request, 'test-norm-payload-client-get-usage-allocation-request-21-001', '2.1')

    def test_get_usage_allocation_recipe_rejects_identifier_on_omitted_branch(self):
        from route_peer import assert_recipe

        request = get_usage_allocation_identifier_variants_request(second_identifier='fixture-key-1')

        with self.assertRaisesRegex(ValueError, 'fixture_usage_allocation_identifier_variants_mismatch'):
            assert_recipe(request, 'test-norm-payload-client-get-usage-allocation-request-21-001', '2.1')

    def test_get_usage_allocation_identifier_forms_execute_after_locate_placeholder(self):
        """KMIP v2.0 §6.1.22 Table 214 and v2.1 §6.1.23 Table 232 specify the optional identifier; §6.1 defines the Locate placeholder producer."""
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {
            'test-norm-payload-client-get-usage-allocation-request-20-001',
            'test-norm-payload-client-get-usage-allocation-request-21-001',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] in case_ids]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 24)
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertEqual(
            {check['protocol_version'] for check in checks},
            {'2.0', '2.1'},
        )
        self.assertEqual(
            {check['constructor_variant'] for check in checks},
            {'usage-allocation-explicit', 'usage-allocation-omitted'},
        )
        required_symbols = {
            'kmipkit_client_batch_item_locate',
            'kmipkit_client_batch_item_get_usage_allocation',
            'kmipkit_client_batch_item_response_locate_unique_identifier_at',
            'kmipkit_client_batch_item_response_get_usage_allocation',
            'kmipkit_client_batch_item_response_get_usage_allocation_ttlv',
            'kmipkit_client_execute',
        }
        self.assertTrue(all(required_symbols <= set(check['public_api_trace_symbols'])
                            for check in checks))
        self.assertTrue(all(check['peer_request_unique_identifier_present_by_batch_item'] ==
                            [False, True, False] for check in checks))
        self.assertTrue(all(check['peer_response_unique_identifier_present_by_batch_item'] ==
                            [True, True, True] for check in checks))
        send_events = [event for event in events if event['action'] == 'send']
        self.assertTrue(all(event['batch_operation_sequence'] ==
                            ['Locate', 'Get Usage Allocation', 'Get Usage Allocation']
                            for event in send_events))

    def test_get_usage_allocation_identifier_variant_response_checks_each_correlated_result(self):
        from controlled_kms_test import response
        from route_peer import parse, verify_usage_allocation_identifier_variants

        response_tree = parse(response(
            (8, 17, 17), 1,
            unique_batch_item_ids=(b'usage-locate', b'usage-explicit', b'usage-placeholder'),
        ))

        self.assertEqual(
            verify_usage_allocation_identifier_variants(response_tree),
            ['fixture-key-1', 'fixture-key-1'],
        )

    def test_get_usage_allocation_identifier_variant_response_rejects_failed_placeholder_result(self):
        from controlled_kms_test import response
        from route_peer import parse, verify_usage_allocation_identifier_variants

        response_tree = parse(response(
            (8, 17, 17), 1,
            unique_batch_item_ids=(b'usage-locate', b'usage-explicit', b'usage-placeholder'),
            result_statuses=(0, 0, 1),
            result_reasons=(None, None, 55),
        ))

        with self.assertRaisesRegex(ValueError, 'fixture_usage_allocation_identifier_response_mismatch'):
            verify_usage_allocation_identifier_variants(response_tree)

    def test_get_usage_allocation_omitted_identifier_resolves_to_prior_response_identifier(self):
        from route_peer import PeerState

        request = get_usage_allocation_identifier_variants_request()

        evidence = PeerState(existing_object=True, usage_allocation_ready=True).execute(request)

        self.assertEqual(evidence['statuses'], [0, 0, 0])
        self.assertEqual(evidence['reasons'], [0, 0, 0])
        self.assertEqual(
            evidence['effects'],
            ['located', 'usage_allocation_returned', 'usage_allocation_returned'],
        )

    def test_route_evidence_output_cannot_escape_its_build_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, 'path_outside_allowed_root'):
                self.runner.resolve_output_path(Path(directory) / 'outside.json')

    def test_route_wrapper_inputs_and_output_are_confined_to_their_workspace_roots(self):
        import generate_route_wrappers

        with tempfile.TemporaryDirectory() as directory:
            outside = Path(directory) / 'outside.json'
            with self.assertRaisesRegex(ValueError, 'path_outside_allowed_root'):
                generate_route_wrappers.resolve_manifest_path(outside)
            with self.assertRaisesRegex(ValueError, 'path_outside_allowed_root'):
                generate_route_wrappers.resolve_output_path(outside)

    def test_route_wrapper_parameter_names_parse_without_regular_expressions(self):
        import generate_route_wrappers

        self.assertEqual(generate_route_wrappers.parameter_name('const char *label'), 'label')
        self.assertEqual(generate_route_wrappers.parameter_name('uint8_t output[32]'), 'output')
        with self.assertRaisesRegex(ValueError, 'unsupported_installed_signature'):
            generate_route_wrappers.parameter_name('void (*callback)(void)')

    def test_known_enum_fixture_preserves_exact_named_target_value(self):
        target = {'kind': 'enumeration_value', 'wire_value': '00000003',
                  'parent_element_ids': ['KMIPKIT-ELEM-ENUMERATION-CRYPTOGRAPHIC-ALGORITHM']}
        fixture = self.runner.scalar_fixture(target, {'Cryptographic Algorithm': 0x420028})
        self.assertEqual(fixture, (0x420028, 5, (3).to_bytes(4, 'big')))

    def test_operation_and_object_type_fixtures_use_exact_normative_values(self):
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        tags = {target['name']: int(target['wire_value'], 16) for target in catalog['normative_targets']
                if target['kind'] == 'tag'}
        operation = targets['KMIPKIT-ELEM-OP-C2S-ACTIVATE']
        object_type = targets['KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY']
        self.assertEqual(self.runner.scalar_fixture(operation, tags, catalog, '2.1'),
                         (tags['Operation'], 5, bytes.fromhex('00000012')))
        self.assertEqual(self.runner.scalar_fixture(object_type, tags, catalog, '2.0'),
                         (tags['Object Type'], 5, bytes.fromhex(object_type['wire_value'])))

    def test_version_boundary_operation_item_uses_its_supported_catalog_value(self):
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        tags = {target['name']: int(target['wire_value'], 16) for target in catalog['normative_targets']
                if target['kind'] == 'tag'}
        operation = targets['KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS']
        value = next(target for target in catalog['normative_targets']
                     if target['kind'] == 'enumeration_value' and target['name'] == 'Get Constraints'
                     and target['parent_element_ids'] == ['KMIPKIT-ELEM-ENUMERATION-OPERATION'])
        self.assertEqual(self.runner.scalar_fixture(operation, tags, catalog, '2.0'),
                         (tags['Operation'], 5, bytes.fromhex(value['wire_value'])))

    def test_generic_ttlv_fixture_uses_structure_member_tag_and_declared_type(self):
        from artifact_support import ROOT, INSTALL
        from route_ttlv import TreeAbi, encode
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-STRUCTURE-MEMBER-2-1-CERTIFICATE-CERTIFICATE-TYPE')
        tags = {item['name']: int(item['wire_value'], 16) for item in catalog['normative_targets']
                if item['kind'] == 'tag'}
        trees = self.runner.generic_tree_fixtures(target, catalog, tags, '2.1')
        self.assertEqual(trees, [(tags['Certificate Type'], 5, bytes.fromhex('00000001'))])
        result = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so').roundtrip(encode(trees[0]))
        self.assertEqual(result['constructed_wire'], result['decoded_wire'])
        self.assertIn('kmipkit_ttlv_item_create', result['invoked_symbols'])

    def test_ephemeral_boolean_field_roundtrips_for_both_kmip_versions(self):
        """OASIS KMIP 2.0 §8.3/Table 358 and KMIP 2.1 §8.3/Table 396 define Ephemeral as Boolean; §10.1 defines its eight-octet encoding."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-MESSAGE-FIELD-8-3-EPHEMERAL')
        tags = {item['name']: int(item['wire_value'], 16)
                for item in catalog['normative_targets'] if item['kind'] == 'tag'}
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        self.assertIn('kmipkit_request_message_from_ttlv', self.runner.LOCAL_SYMBOLS)

        for version in ('2.0', '2.1'):
            with self.subTest(protocol_version=version):
                trees = self.runner.generic_tree_fixtures(target, catalog, tags, version)
                self.assertEqual([tree[0] for tree in trees], [tags['Request Message']] * 2)
                booleans = [next(child for child in tree[2][1][2] if child[0] == tags['Ephemeral'])
                            for tree in trees]
                self.assertEqual(booleans, [
                    (tags['Ephemeral'], 6, bytes(8)),
                    (tags['Ephemeral'], 6, bytes.fromhex('0000000000000001')),
                ])
                for tree in trees:
                    result = abi.roundtrip(encode(tree))
                    self.assertIn('kmipkit_ttlv_value_boolean', result['invoked_symbols'])
                    self.assertIn('kmipkit_ttlv_item_create', result['invoked_symbols'])
                    self.assertIn('kmipkit_ttlv_decode', result['invoked_symbols'])
                    self.assertIn('kmipkit_request_message_from_ttlv', result['invoked_symbols'])
                    self.assertEqual(result['constructed_wire'], result['decoded_wire'])

    def test_datetime_constructor_executes_normative_message_timestamp_cells(self):
        """OASIS KMIP 2.0 §9.20 Table 387 and KMIP 2.1 §9.20 Table 425 define Time Stamp as Date-Time."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP')
        tags = {item['name']: int(item['wire_value'], 16)
                for item in catalog['normative_targets'] if item['kind'] == 'tag'}
        trees = self.runner.generic_tree_fixtures(target, catalog, tags, '2.1')

        self.assertIn('kmipkit_ttlv_value_date_time', self.runner.LOCAL_SYMBOLS)
        self.assertEqual(len(trees), 1)
        result = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so').roundtrip(encode(trees[0]))
        self.assertIn('kmipkit_ttlv_value_date_time', result['invoked_symbols'])
        self.assertEqual(result['constructed_wire'], result['decoded_wire'])

    def test_generic_ttlv_value_constructors_are_catalogued_for_route_execution(self):
        expected = {
            'kmipkit_ttlv_value_long_integer', 'kmipkit_ttlv_value_boolean',
            'kmipkit_ttlv_value_interval', 'kmipkit_ttlv_value_date_time_extended',
            'kmipkit_ttlv_value_big_integer', 'kmipkit_ttlv_value_text_string',
            'kmipkit_ttlv_value_byte_string', 'kmipkit_ttlv_value_structure',
        }

        self.assertLessEqual(expected, self.runner.LOCAL_SYMBOLS)

    def test_generic_ttlv_constructors_roundtrip_binary_string_and_structure_members(self):
        """OASIS KMIP 2.1 §7.2, §7.40, §8.1, and §9.10 require their declared TTLV value types."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {item['target_id']: item for item in catalog['normative_targets']}
        tags = {item['name']: int(item['wire_value'], 16)
                for item in catalog['normative_targets'] if item['kind'] == 'tag'}
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        cases = (
            ('KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-ASYNCHRONOUS-CORRELATION-VALUE',
             'kmipkit_ttlv_value_byte_string'),
            ('KMIPKIT-ELEM-MESSAGE-FIELD-9-10-SERVER-CORRELATION-VALUE',
             'kmipkit_ttlv_value_text_string'),
            ('KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-PRIME-FIELD-SIZE',
             'kmipkit_ttlv_value_big_integer'),
            ('KMIPKIT-ELEM-MESSAGE-FIELD-8-1-REQUEST-HEADER',
             'kmipkit_ttlv_value_structure'),
        )

        for target_id, symbol in cases:
            with self.subTest(target_id=target_id):
                target = targets[target_id]
                trees = self.runner.generic_tree_fixtures(target, catalog, tags, '2.1')
                self.assertTrue(trees)
                result = abi.roundtrip(encode(trees[0]))
                self.assertIn(symbol, result['invoked_symbols'])
                self.assertEqual(result['constructed_wire'], result['decoded_wire'])

    def test_generic_structure_recipes_include_normative_child_items(self):
        """OASIS KMIP 2.1 §7.2 and §8.1 define nested Structure members."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {item['target_id']: item for item in catalog['normative_targets']}
        tags = {item['name']: int(item['wire_value'], 16)
                for item in catalog['normative_targets'] if item['kind'] == 'tag'}
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        for target_id in (
            'KMIPKIT-ELEM-OPERATION-STRUCTURE-7-2-ASYNCHRONOUS-REQUEST',
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-1-REQUEST-HEADER',
        ):
            with self.subTest(target_id=target_id):
                trees = self.runner.generic_tree_fixtures(targets[target_id], catalog, tags, '2.1')
                self.assertTrue(trees)
                tree = next(tree for tree in trees if tree[1] == 1 and tree[2])
                required_tags = {
                    tags[child['name']] for child in catalog['normative_targets']
                    if target_id in child.get('parent_element_ids', [])
                    and '2.1' in child.get('protocol_versions', [])
                    and child.get('source_requiredness', '').strip().lower() in {'yes', 'required'}
                }
                self.assertLessEqual(required_tags, {child[0] for child in tree[2]})
                result = abi.roundtrip(encode(tree))
                self.assertIn('kmipkit_ttlv_structure_with_item', result['invoked_symbols'])
                self.assertIn('kmipkit_ttlv_value_structure', result['invoked_symbols'])
                self.assertEqual(result['constructed_wire'], result['decoded_wire'])

    def test_controlled_peer_plan_includes_delegated_login_request_first(self):
        import controlled_kms_test

        operations, cases = controlled_kms_test.include_delegated_login(
            (1, 45), ('test-op-login',))

        self.assertEqual(operations, (47, 1, 45))
        self.assertEqual(cases, ('test-op-delegated-login', 'test-op-login'))

    def test_logout_recipe_requires_a_login_ticket_before_logout(self):
        """OASIS KMIP 2.0 §6.1.30 and KMIP 2.1 §6.1.31 require a Logout Ticket."""
        import route_peer
        from artifact_support import ROOT

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(case for case in catalog['test_cases'] if case['case_id'] == 'test-op-logout')
        self.assertEqual(route_peer.recipe_for_case(case, None), ((45, 46), 'login_logout'))

    def test_logout_matrix_credits_the_logout_exchange_not_its_login_prerequisite(self):
        """OASIS KMIP 2.0 §6.1.30 and KMIP 2.1 §6.1.31 define the target exchange."""
        from route_peer import target_operation_exchange_index

        self.assertEqual(target_operation_exchange_index(((45, 46), 'login_logout')), 1)
        self.assertEqual(target_operation_exchange_index(((45,), 'login')), 0)

    def test_interop_recipe_exercises_begin_end_and_reset(self):
        """OASIS KMIP 2.0 §6.1.25 and KMIP 2.1 §6.1.26 define all three functions."""
        import route_peer
        from artifact_support import ROOT

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        case = next(case for case in catalog['test_cases'] if case['case_id'] == 'test-op-interop')

        self.assertEqual(route_peer.recipe_for_case(case, None), ((52, 52, 52), 'interop'))

    def test_interop_peer_checks_ordered_functions_and_one_identifier(self):
        """OASIS KMIP 2.0 Table 223 and KMIP 2.1 Table 241 require both fields."""
        from route_peer import assert_interop_request, parse, tlv

        def request(functions, *, include_batch_item_ids=True,
                    identifier=b'kmipkit-interop-fixture'):
            items = []
            for index, function in enumerate(functions):
                payload = tlv(0x420160, 5, function.to_bytes(4, 'big'))
                payload += tlv(0x420161, 7, identifier)
                batch_item = tlv(0x42005C, 5, (52).to_bytes(4, 'big'))
                if include_batch_item_ids:
                    batch_item += tlv(0x420093, 8,
                        (b'interop-begin', b'interop-end', b'interop-reset')[index])
                batch_item += tlv(0x420079, 1, payload)
                items.append(tlv(0x42000F, 1, batch_item))
            version = tlv(0x420069, 1,
                tlv(0x42006A, 2, (2).to_bytes(4, 'big')) +
                tlv(0x42006B, 2, (1).to_bytes(4, 'big')))
            header = tlv(0x420077, 1, version +
                tlv(0x42000D, 2, len(functions).to_bytes(4, 'big')))
            return parse(tlv(0x420078, 1, header + b''.join(items)))

        assert_interop_request(request((1, 2, 3)), '2.1')
        with self.assertRaisesRegex(ValueError, 'interop_function_sequence_mismatch'):
            assert_interop_request(request((1, 3, 2)), '2.1')
        with self.assertRaisesRegex(ValueError, 'fixture_interop_batch_item_id_mismatch'):
            assert_interop_request(request((1, 2, 3), include_batch_item_ids=False), '2.1')

    def test_interop_peer_models_begin_end_reset_without_retaining_identifier(self):
        """OASIS KMIP 2.0 §6.1.25 and KMIP 2.1 §6.1.26 define test-state controls."""
        from route_peer import PeerState, parse, tlv

        items = []
        for function in (1, 2, 3):
            payload = tlv(0x420160, 5, function.to_bytes(4, 'big'))
            payload += tlv(0x420161, 7, b'private-fixture-identifier')
            batch_item = tlv(0x42005C, 5, (52).to_bytes(4, 'big'))
            batch_item += tlv(0x420079, 1, payload)
            items.append(tlv(0x42000F, 1, batch_item))
        request = parse(tlv(0x420078, 1, b''.join(items)))
        state = PeerState()

        evidence = state.execute(request)

        self.assertEqual(evidence['effects'], ['interop_begun', 'interop_ended', 'interop_reset'])
        self.assertEqual(evidence['peer_observed_state_before_sha256'],
                         evidence['peer_observed_state_after_sha256'])
        self.assertNotIn(b'private-fixture-identifier', repr(state.__dict__).encode())

    def test_logout_peer_requires_the_ticket_returned_by_login(self):
        """OASIS KMIP 2.0 §6.1.30 and 2.1 §6.1.31 require Logout's Ticket."""
        from route_peer import assert_login_logout_exchange, parse, tlv

        integer = lambda tag, value: tlv(tag, 2, value.to_bytes(4, 'big'))
        enumeration = lambda tag, value: tlv(tag, 5, value.to_bytes(4, 'big'))
        def request(operation, payload=b''):
            if operation == 45 and not payload:
                payload = tlv(0x420079, 1, integer(0x42014C, 2))
            item = enumeration(0x42005C, operation) + payload
            return parse(tlv(0x420078, 1,
                tlv(0x420077, 1,
                    tlv(0x420069, 1, integer(0x42006A, 2) + integer(0x42006B, 1)) +
                    integer(0x42000D, 1)) + tlv(0x42000F, 1, item)))

        assert_login_logout_exchange(request(45), '2.1', 0)
        ticket = tlv(0x420149, 1,
                     enumeration(0x42014A, 1) + tlv(0x42014B, 8, b'controlled-login-ticket'))
        logout_payload = tlv(0x420079, 1, ticket)
        assert_login_logout_exchange(request(46, logout_payload), '2.1', 1)
        bad_ticket = tlv(0x420149, 1,
                         enumeration(0x42014A, 1) + tlv(0x42014B, 8, b'other-ticket'))
        with self.assertRaisesRegex(ValueError, 'fixture_logout_ticket_mismatch'):
            assert_login_logout_exchange(
                request(46, tlv(0x420079, 1, bad_ticket)), '2.1', 1)

    def test_logout_peer_checks_success_ticket_and_empty_logout_payload(self):
        """OASIS KMIP 2.0 §6.1.29-§6.1.30 and 2.1 §6.1.30-§6.1.31 define response payloads."""
        from controlled_kms_test import response
        from route_peer import parse, verify_login_logout_response

        login = parse(response((45,), 1))
        logout = parse(response((46,), 1))
        verify_login_logout_response(login, 0)
        verify_login_logout_response(logout, 1)
        logout[2][1][2][2][2].append((0x420094, 7, b'bad'))
        with self.assertRaisesRegex(ValueError, 'fixture_logout_response_payload_not_empty'):
            verify_login_logout_response(logout, 1)

    def test_query_operation_response_cases_have_controlled_peer_recipes(self):
        """OASIS KMIP 2.0 §6.1.37 Table 260 and KMIP 2.1 §6.1.40 Table 283 define Query responses."""
        import route_peer
        from artifact_support import ROOT

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        cases = {case['case_id']: case for case in catalog['test_cases']}
        for case_id in (
            'test-norm-payload-client-query-response-20-001',
            'test-norm-payload-client-query-response-21-001',
        ):
            with self.subTest(case_id=case_id):
                self.assertEqual(route_peer.recipe_for_case(cases[case_id], None), ((24,), 'query'))

    def test_query_operation_responses_execute_and_read_typed_and_ttlv_fields(self):
        """OASIS KMIP 2.0 §6.1.37 Table 260 and KMIP 2.1 §6.1.40 Table 283 define Query responses."""
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {
            'test-norm-payload-client-query-response-20-001',
            'test-norm-payload-client-query-response-21-001',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] in case_ids]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12)
        self.assertEqual(len([event for event in events if event['action'] == 'send']), 12)
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertTrue(all(check['consumer_library_correctness'] == 'passed' for check in checks))
        required_symbols = {
            'kmipkit_client_batch_item_response_query_field_count',
            'kmipkit_client_batch_item_response_query_field_at',
            'kmipkit_client_batch_item_response_query_ttlv',
        }
        self.assertTrue(all(required_symbols <= set(check['public_api_trace_symbols'])
                            for check in checks))
        for case_id in case_ids:
            case = next(item for item in catalog['test_cases'] if item['case_id'] == case_id)
            target_id = case['requires']['normative_target_id']
            decoded_response_symbols = {
                event['public_api_symbol'] for event in events
                if event['case_id'] == case_id and event['action'] == 'decode'
                and event['target_id'] == target_id
                and event['public_api_symbol'] in required_symbols
            }
            self.assertEqual(decoded_response_symbols, required_symbols)

    def test_normative_get_request_payload_routes_execute_in_both_versions(self):
        """OASIS KMIP 2.0 §6.1.19/Table 205 and KMIP 2.1 §6.1.19/Table 220 define Get request fields."""
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {
            'test-norm-payload-client-get-request-20-001',
            'test-norm-payload-client-get-request-21-001',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] in case_ids]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12)
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        for case_id in case_ids:
            case_checks = [check for check in checks if check['case_id'] == case_id]
            self.assertEqual(len(case_checks), 6)
            self.assertTrue(all('kmipkit_client_batch_item_get' in check['public_api_trace_symbols']
                                for check in case_checks))
            self.assertTrue(any(event['case_id'] == case_id and
                                event['public_api_symbol'] == 'kmipkit_client_batch_item_get'
                                for event in events))

    def test_normative_get_attribute_list_request_payload_routes_execute_in_both_versions(self):
        """OASIS KMIP 2.0 §6.1.21/Table 211 and KMIP 2.1 §6.1.21/Table 226 define this payload."""
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {
            'test-norm-payload-client-get-attribute-list-request-20-001',
            'test-norm-payload-client-get-attribute-list-request-21-001',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] in case_ids]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 24)
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        for case_id in case_ids:
            case_checks = [check for check in checks if check['case_id'] == case_id]
            self.assertEqual(len(case_checks), 12)
            for variant, symbol in (
                    ('direct', 'kmipkit_client_batch_item_get_attribute_list'),
                    ('standalone', 'kmipkit_client_batch_item_get_attribute_list_from_request')):
                variant_checks = [check for check in case_checks
                                  if check['constructor_variant'] == variant]
                self.assertEqual(len(variant_checks), 6)
                self.assertTrue(all(symbol in check['public_api_trace_symbols']
                                    for check in variant_checks))
                self.assertTrue(all(check.get('peer_request_unique_identifier_present_by_batch_item')
                                    == [False, True, False] for check in variant_checks))
                self.assertTrue(any(event['case_id'] == case_id and
                                    event['public_api_symbol'] == symbol for event in events))

    def test_normative_add_attribute_request_payload_routes_execute_in_both_versions(self):
        """OASIS KMIP 2.0 §6.1.2/Table 152 and KMIP 2.1 §6.1.2/Table 167 require New Attribute."""
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {
            'test-norm-payload-client-add-attribute-request-20-002',
            'test-norm-payload-client-add-attribute-request-21-002',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] in case_ids]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 24)
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        for case_id in case_ids:
            case_checks = [check for check in checks if check['case_id'] == case_id]
            self.assertEqual(len(case_checks), 12)
            for variant, symbol in (
                    ('direct', 'kmipkit_client_batch_item_add_attribute'),
                    ('standalone', 'kmipkit_client_batch_item_add_attribute_from_request')):
                variant_checks = [check for check in case_checks
                                  if check['constructor_variant'] == variant]
                self.assertEqual(len(variant_checks), 6)
                self.assertTrue(all(symbol in check['public_api_trace_symbols']
                                    for check in variant_checks))
                self.assertTrue(all(check.get('peer_new_attribute_present') is True
                                    for check in variant_checks))
                self.assertTrue(any(event['case_id'] == case_id and
                                    event['public_api_symbol'] == symbol for event in events))
            request_events = [event for event in events if event['case_id'] == case_id and
                              event['action'] == 'send']
            version_digits = case_id.split('-request-')[1][:2]
            version = f'{version_digits[0]}.{version_digits[1]}'
            for encoding in ('TTLV', 'JSON', 'XML'):
                for transport in ('raw_tls', 'https'):
                    cell_events = [event for event in request_events
                                   if event['protocol_version'] == version and
                                   event['encoding'] == encoding and event['transport'] == transport and
                                   event['target_id'].startswith(
                                       'KMIPKIT-PAYLOAD-CLIENT-ADD-ATTRIBUTE-REQUEST-')]
                    direct = [event for event in cell_events if event['public_api_symbol'] ==
                              'kmipkit_client_batch_item_add_attribute']
                    standalone = [event for event in cell_events if event['public_api_symbol'] ==
                                  'kmipkit_client_batch_item_add_attribute_from_request']
                    self.assertEqual(len(direct), 1)
                    self.assertEqual(len(standalone), 1)
                    self.assertEqual(direct[0]['request_tree_sha256'],
                                     standalone[0]['request_tree_sha256'])

    def test_normative_modify_attribute_request_payload_routes_execute_in_both_versions(self):
        """OASIS KMIP 2.0 §6.1.33/Table 247 and KMIP 2.1 §6.1.34/Table 265 define this payload."""
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {
            'test-norm-payload-client-modify-attribute-request-20-002',
            'test-norm-payload-client-modify-attribute-request-20-003',
            'test-norm-payload-client-modify-attribute-request-21-002',
            'test-norm-payload-client-modify-attribute-request-21-003',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] in case_ids]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 48,
                         'Modify Attribute current/new payload fields must run through both constructors')
        self.assertTrue(all(check['status'] == 'passed' for check in checks),
                        repr([check for check in checks if check['status'] != 'passed'][:8]))
        for case_id in case_ids:
            case_checks = [check for check in checks if check['case_id'] == case_id]
            self.assertEqual(len(case_checks), 12)
            for variant, symbol in (
                    ('direct', 'kmipkit_client_batch_item_modify_attribute'),
                    ('standalone', 'kmipkit_client_batch_item_modify_attribute_from_request')):
                variant_checks = [check for check in case_checks
                                  if check['constructor_variant'] == variant]
                self.assertEqual(len(variant_checks), 6)
                self.assertTrue(all(symbol in check['public_api_trace_symbols']
                                    for check in variant_checks))
                self.assertTrue(any(event['case_id'] == case_id and
                                    event['public_api_symbol'] == symbol for event in events))
            expected_current = case_id.endswith('-002')
            self.assertTrue(all(check['peer_current_attribute_present'] == expected_current
                                for check in case_checks))
            self.assertTrue(all(check['peer_new_attribute_present'] is True
                                for check in case_checks))
            self.assertTrue(all(check['peer_attribute_state_changed'] is True
                                for check in case_checks))
            self.assertTrue(all(check['peer_seed_operations'] == [13] and
                                check['peer_request_operations'] == [14] and
                                check['peer_cleanup_operations'] == [19, 20]
                                for check in case_checks))
            request_events = [event for event in events if event['case_id'] == case_id and
                              event['action'] == 'send']
            version_digits = case_id.split('-request-')[1][:2]
            version = f'{version_digits[0]}.{version_digits[1]}'
            for encoding in ('TTLV', 'JSON', 'XML'):
                for transport in ('raw_tls', 'https'):
                    cell_events = [event for event in request_events
                                   if event['protocol_version'] == version and
                                   event['encoding'] == encoding and event['transport'] == transport and
                                   event['target_id'].startswith(
                                       'KMIPKIT-PAYLOAD-CLIENT-MODIFY-ATTRIBUTE-REQUEST-')]
                    direct = [event for event in cell_events if event['public_api_symbol'] ==
                              'kmipkit_client_batch_item_modify_attribute']
                    standalone = [event for event in cell_events if event['public_api_symbol'] ==
                                  'kmipkit_client_batch_item_modify_attribute_from_request']
                    self.assertEqual(len(direct), 1)
                    self.assertEqual(len(standalone), 1)
                    self.assertEqual(direct[0]['request_tree_sha256'],
                                     standalone[0]['request_tree_sha256'])

    def test_normative_modify_attribute_response_payload_routes_execute_in_both_versions(self):
        """OASIS KMIP 2.0 §6.1.33/Table 248 and KMIP 2.1 §6.1.34/Table 266 require Unique Identifier."""
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {
            'test-norm-payload-client-modify-attribute-response-20-001',
            'test-norm-payload-client-modify-attribute-response-21-001',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] in case_ids]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 24)
        self.assertTrue(all(check['status'] == 'passed' for check in checks),
                        repr([check for check in checks if check['status'] != 'passed'][:8]))
        for case_id in case_ids:
            case_checks = [check for check in checks if check['case_id'] == case_id]
            self.assertEqual(len(case_checks), 12)
            self.assertTrue(all('kmipkit_client_batch_item_response_modify_attribute' in
                                check['public_api_trace_symbols'] for check in case_checks))
            self.assertTrue(any(event['case_id'] == case_id and
                                event['public_api_symbol'] ==
                                'kmipkit_client_batch_item_response_modify_attribute' and
                                event['action'] == 'decode' for event in events))

    def test_reserved_tag_is_not_constructed_as_an_assigned_ttlv_item(self):
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-TAG-420009')
        tags = {item['name']: int(item['wire_value'], 16) for item in catalog['normative_targets']
                if item['kind'] == 'tag'}
        self.assertIsNone(self.runner.generic_tree_fixtures(target, catalog, tags, '2.1'))

    def test_enum_extension_and_reserved_ranges_roundtrip_both_boundaries(self):
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        tags = {target['name']: int(target['wire_value'], 16) for target in catalog['normative_targets']
                if target['kind'] == 'tag'}
        enum_extension = targets['KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-EXTENSIONS-8XXXXXXX']
        mask_extension = targets['KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-EXTENSIONS-XXX00000']
        reserved_range = targets['KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-00000001-00000103']
        self.assertEqual([tree[2] for tree in self.runner.generic_tree_fixtures(
            enum_extension, catalog, tags, '2.1')], [bytes.fromhex('80000000'), bytes.fromhex('8fffffff')])
        self.assertEqual([tree[2] for tree in self.runner.generic_tree_fixtures(
            mask_extension, catalog, tags, '2.1')], [bytes.fromhex('00000000'), bytes.fromhex('fff00000')])
        self.assertEqual([tree[2] for tree in self.runner.generic_tree_fixtures(
            reserved_range, catalog, tags, '2.1')], [bytes.fromhex('00000001'), bytes.fromhex('00000103')])

    def test_tag_allocation_ranges_check_each_boundary_through_the_decoder(self):
        from artifact_support import ROOT, INSTALL
        from route_ttlv import TreeAbi, encode
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        targets = [target for target in catalog['normative_targets'] if target['kind'] == 'tag_range']
        self.assertEqual(len(targets), 5)
        expected_rejections = {
            'KMIPKIT-RANGE-001': [12, 12],
            'KMIPKIT-RANGE-002': [12, 11],
            'KMIPKIT-RANGE-003': [12, 12],
            'KMIPKIT-RANGE-005': [12, 12],
        }
        for target in targets:
            with self.subTest(target=target['target_id']):
                result = self.runner.execute_tag_range_case(abi, target, catalog)
                self.assertIn('kmipkit_ttlv_decode', result['invoked_symbols'])
                if target['allocation'] == 'extension':
                    self.assertIn('kmipkit_ttlv_item_create', result['invoked_symbols'])
                else:
                    wires = [encode((tag, 5, (7).to_bytes(4, 'big')))
                             for tag in self.runner.range_boundary_tags(target)]
                    self.assertEqual(result['rejection_codes'], expected_rejections[target['target_id']])
                    self.assertEqual(result['decode_input_sha256'], hashlib.sha256(b''.join(wires)).hexdigest())
                self.assertEqual(result['sample_count'], 2)

    def test_ttlv_encoding_parser_keeps_compound_types_distinct(self):
        self.assertEqual(self.runner._type_codes('Long Integer'), [3])
        self.assertEqual(self.runner._type_codes('Big Integer'), [4])
        self.assertEqual(self.runner._type_codes('Date Time Extended'), [11])
        self.assertEqual(self.runner._type_codes('Text String, Enumeration or Integer'), [7, 5, 2])

    def test_unmapped_structure_branch_is_pending_not_arbitrary_scalar(self):
        target = {'kind': 'operation_payload_field', 'name': 'Unique Identifier'}
        self.assertIsNone(self.runner.scalar_fixture(target, {}))

    def test_pending_cells_cannot_pass_completeness_gate(self):
        with self.assertRaisesRegex(ValueError, 'pending'):
            self.runner.require_complete([{'status': 'passed'}], [{'reason': 'validator_recipe_pending'}])

    def test_duplicate_events_rejected(self):
        event = {'case_id': 'case', 'route_id': 'route', 'target_id': 'target',
                 'protocol_version': '2.1', 'encoding': 'TTLV', 'transport': None,
                 'action': 'construct', 'execution_route_id': None, 'status': 'passed'}
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            self.runner.require_complete([event, event], [])

    def test_ci_executes_matrix_and_shared_evidence_gate(self):
        root = Path(__file__).resolve().parents[3]
        workflow = (root / '.github/workflows/interop-c.yml').read_text()
        self.assertIn('route_matrix.py', workflow)
        self.assertIn('validate_route_evidence.py', workflow)


    def test_installed_scalar_roundtrip_checks_exact_value(self):
        from artifact_support import INSTALL
        abi = self.runner.InstalledAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        fixture = (0x420028, 5, bytes.fromhex('00000003'))
        wire = abi.execute('kmipkit_ttlv_decode', fixture)
        self.assertEqual(wire[8:12], fixture[2])
        self.assertIn('kmipkit_ttlv_decode', abi.called)

    def test_named_route_is_not_credited_when_another_symbol_was_called(self):
        from artifact_support import INSTALL
        abi = self.runner.InstalledAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        with self.assertRaisesRegex(ValueError, 'named_route_not_invoked'):
            abi.execute('kmipkit_ttlv_value_boolean', (0x420028, 5, bytes.fromhex('00000003')))

    def test_codec_roundtrips_controlled_response_in_all_encodings(self):
        from route_peer import Codec, parse, response
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        codec = Codec(catalog)
        wire = response((1, 10, 20), 1)
        for encoding in ('TTLV', 'JSON', 'XML'):
            self.assertEqual(codec.decode(codec.encode(wire, encoding), encoding), parse(wire))

    def test_json_codec_decodes_concatenated_named_bitmask_values(self):
        from route_peer import Codec
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        codec = Codec(catalog)
        wire = json.dumps({'tag': '0x42002C', 'type': 'Integer', 'value': 'SignVerify'}).encode()
        self.assertEqual(codec.decode(wire, 'JSON'), (0x42002C, 2, (3).to_bytes(4, 'big')))

    def test_positive_case_cannot_pass_by_operation_name_alone(self):
        from route_peer import assert_recipe, parse, tlv
        # This contains the correct Query operation but omits Query Function.
        integer = lambda tag, value: tlv(tag, 2, value.to_bytes(4, 'big'))
        request = tlv(0x420078, 1, integer(0x42006A, 2) + integer(0x42006B, 1)
                      + integer(0x42000D, 1) + tlv(0x42005C, 5, (24).to_bytes(4, 'big')))
        with self.assertRaisesRegex(ValueError, 'fixture_query_function_mismatch'):
            assert_recipe(parse(request), 'test-op-query', '2.1')

    def test_request_payload_target_requires_its_field_inside_the_exact_operation(self):
        from artifact_support import ROOT
        from route_peer import Codec, parse, request_payload_field_present, tlv
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        codec = Codec(catalog)

        def enum(tag, value):
            return tlv(tag, 5, value.to_bytes(4, 'big'))

        query_request = parse(tlv(0x42000F, 1,
            enum(0x42005C, 24) + tlv(0x420079, 1, enum(0x420074, 1))))
        query_function = targets['KMIPKIT-PAYLOAD-CLIENT-QUERY-REQUEST-20-001']
        object_groups = targets['KMIPKIT-PAYLOAD-CLIENT-QUERY-REQUEST-21-002']
        self.assertTrue(request_payload_field_present(query_function, query_request, targets, codec))
        self.assertFalse(request_payload_field_present(object_groups, query_request, targets, codec))

        unrelated_operation = parse(tlv(0x42000F, 1,
            enum(0x42005C, 10) + tlv(0x420079, 1, tlv(0x420094, 7, b'fixture-id'))))
        unique_identifier = targets['KMIPKIT-PAYLOAD-CLIENT-GET-ATTRIBUTES-REQUEST-20-001']
        self.assertFalse(request_payload_field_present(unique_identifier, unrelated_operation, targets, codec))

        get_attributes = parse(tlv(0x42000F, 1,
            enum(0x42005C, 11) + tlv(0x420079, 1, tlv(0x420094, 7, b'fixture-id'))))
        self.assertTrue(request_payload_field_present(unique_identifier, get_attributes, targets, codec))

    def test_query_payload_normative_cases_invoke_c_and_match_controlled_peer(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {
            'test-norm-payload-client-query-request-20-001',
            'test-norm-payload-client-query-request-21-001',
            'test-norm-payload-client-query-request-21-002',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                                if case['case_id'] in case_ids]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            network_combinations=(('TTLV', 'raw_tls'),))
        self.assertEqual(len(checks), 3)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertTrue(all('kmipkit_client_batch_item_query' in check['public_api_trace_symbols']
                            for check in checks), checks)
        fields = [event for event in events if event['target_id'].startswith(
            'KMIPKIT-PAYLOAD-CLIENT-QUERY-REQUEST-')]
        self.assertEqual(len(fields), 6)
        self.assertEqual({event['action'] for event in fields}, {'construct', 'send'})
        self.assertTrue(all(event['public_api_symbol'] == 'kmipkit_client_batch_item_query'
                            for event in fields))

    def test_create_payload_normative_cases_invoke_c_and_match_controlled_peer(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {
            'test-norm-payload-client-create-request-20-001',
            'test-norm-payload-client-create-request-20-002',
            'test-norm-payload-client-create-request-21-001',
            'test-norm-payload-client-create-request-21-002',
            'test-norm-payload-client-create-request-21-003',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                                if case['case_id'] in case_ids]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            network_combinations=(('TTLV', 'raw_tls'),))
        self.assertEqual(len(checks), 5)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        fields = [event for event in events if event['target_id'].startswith(
            'KMIPKIT-PAYLOAD-CLIENT-CREATE-REQUEST-')]
        self.assertEqual(len(fields), 10)
        self.assertEqual({event['action'] for event in fields}, {'construct', 'send'})
        self.assertTrue(all(event['public_api_symbol'] == 'kmipkit_client_batch_item_create'
                            for event in fields))

    def test_lifecycle_peer_requires_cleanup_and_checks_illegal_destroy(self):
        import route_peer
        self.assertTrue(hasattr(route_peer, 'verify_lifecycle'), 'stateful lifecycle verification is missing')
        from route_peer import parse, response
        effects = route_peer.verify_lifecycle((1, 18, 20, 19, 20), parse(response((1, 18, 20, 19, 20), 1)))
        self.assertIn('illegal_destroy_rejected', effects)
        self.assertEqual(effects[-1], 'destroyed')
        with self.assertRaisesRegex(ValueError, 'fixture_cleanup_missing'):
            route_peer.verify_lifecycle((1,), parse(response((1,), 1)))

    def test_reserved_tag_decoder_is_checked_as_rejection(self):
        from artifact_support import INSTALL
        abi = self.runner.InstalledAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        self.assertTrue(hasattr(abi, 'execute_reserved_tag'), 'reserved tag negative recipe is missing')
        wire = abi.execute_reserved_tag('kmipkit_ttlv_decode', 0x420009)
        self.assertEqual(wire[:3], bytes.fromhex('420009'))
        self.assertIn('kmipkit_ttlv_decode', abi.called)

    def test_reserved_tag_decode_event_reports_rejection_without_tree_hash(self):
        cell = {'case_id': 'test-norm-tag-tag-420009', 'route_id': 'c.generic-ttlv.kmipkit_ttlv_decode',
                'target_id': 'KMIPKIT-ELEM-TAG-420009', 'public_api_symbol': 'kmipkit_ttlv_decode',
                'execution_route_id': None, 'execution_api_symbol': None, 'action': 'decode',
                'protocol_version': '2.0', 'encoding': 'TTLV', 'transport': None,
                'workflow_step_index': None, 'workflow_operation': None, 'workflow_item_index': None}
        wire = bytes.fromhex('42000905000000040000000700000000')
        event = self.runner.rejected_tag_event(cell, wire, 4, allocation='reserved')
        self.assertEqual(event['decode_outcome'], 'rejected')
        self.assertEqual(event['decode_rejection_code'], 'tag_reserved')
        self.assertEqual(event['decode_input_sha256'], hashlib.sha256(wire).hexdigest())
        self.assertNotIn('response_tree_sha256', event)
        unused_event = self.runner.rejected_tag_event(cell, wire, 5, allocation='unused')
        self.assertEqual(unused_event['decode_rejection_code'], 'tag_not_allocated')

    def test_certificate_model_recipe_invokes_typed_and_generic_field_routes(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['sample_count'], 1)
        self.assertEqual(result['normalized_tree'], [0x420013, 1, [
            [0x42001d, 5, '00000001'], [0x42001e, 8, '010203']]])
        self.assertTrue({
            'kmipkit_certificate_from_fields', 'kmipkit_managed_object_from_fields',
            'kmipkit_managed_object_from_item',
        }.issubset(result['invoked_symbols']))

    def test_certificate_object_type_cells_execute_their_typed_constructor(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        models = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)
        routes = (
            ('KMIPKIT-ELEM-OBJECT-TYPE-CERTIFICATE', 'Certificate',
             'kmipkit_certificate_from_fields'),
            ('KMIPKIT-ELEM-OBJECT-TYPE-CERTIFICATE-REQUEST', 'Certificate Request',
             'kmipkit_certificate_request_from_fields'),
        )
        for target_id, name, symbol in routes:
            with self.subTest(name=name):
                target = next(item for item in catalog['normative_targets']
                              if item['target_id'] == target_id)
                result = models.execute(target, '2.0', symbol)

                self.assertIsNotNone(result)
                self.assertIn(symbol, result['invoked_symbols'])

    def test_named_object_type_cells_execute_their_typed_constructors(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        models = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)
        routes = (
            ('KMIPKIT-ELEM-OBJECT-TYPE-OPAQUE-OBJECT', 'kmipkit_opaque_object_from_fields'),
            ('KMIPKIT-ELEM-OBJECT-TYPE-PGP-KEY', 'kmipkit_pgp_key_from_fields'),
            ('KMIPKIT-ELEM-OBJECT-TYPE-SECRET-DATA', 'kmipkit_secret_data_from_fields'),
            ('KMIPKIT-ELEM-OBJECT-TYPE-SPLIT-KEY', 'kmipkit_split_key_from_fields'),
            ('KMIPKIT-ELEM-OBJECT-TYPE-PRIVATE-KEY', 'kmipkit_private_key_from_fields'),
            ('KMIPKIT-ELEM-OBJECT-TYPE-PUBLIC-KEY', 'kmipkit_public_key_from_fields'),
        )
        for target_id, symbol in routes:
            with self.subTest(target_id=target_id):
                target = next(item for item in catalog['normative_targets']
                              if item['target_id'] == target_id)
                result = models.execute(target, '2.0', symbol)

                self.assertIsNotNone(result)
                self.assertIn(symbol, result['invoked_symbols'])

    def test_symmetric_key_model_recipe_invokes_nested_key_block_routes(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-SYMMETRIC-KEY')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [0x42008f, 1, [
            [0x420040, 1, [[0x420042, 5, '00000001']]]]])
        self.assertTrue({
            'kmipkit_key_block_from_fields', 'kmipkit_key_block_from_item',
            'kmipkit_symmetric_key_from_fields', 'kmipkit_managed_object_from_fields',
            'kmipkit_managed_object_from_item',
        }.issubset(result['invoked_symbols']))

    def test_symmetric_key_object_type_cell_executes_its_typed_constructor(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.0', 'kmipkit_symmetric_key_from_fields')

        self.assertIsNotNone(result)
        self.assertIn('kmipkit_symmetric_key_from_fields', result['invoked_symbols'])

    def test_revocation_reason_attribute_cell_executes_its_constructor(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-ATTRIBUTE-REVOCATION-REASON')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.0', 'kmipkit_revocation_reason_create')

        self.assertIsNotNone(result)
        self.assertIn('kmipkit_revocation_reason_create', result['invoked_symbols'])
        self.assertEqual(result['sample_count'], 16)

    def test_named_managed_object_recipes_cover_catalogued_object_structures(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        cases = {
            'KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE-REQUEST': (
                'kmipkit_certificate_request_from_fields',
                [0x420018, 1, [[0x420019, 5, '00000002'], [0x420140, 8, '72657175657374']]],
            ),
            'KMIPKIT-ELEM-OBJECT-STRUCTURE-PGP-KEY': (
                'kmipkit_pgp_key_from_fields',
                [0x4200bd, 1, [[0x4200be, 2, '00000007'],
                               [0x420040, 1, [[0x420042, 5, '00000001']]]]],
            ),
            'KMIPKIT-ELEM-OBJECT-STRUCTURE-PRIVATE-KEY': (
                'kmipkit_private_key_from_fields',
                [0x420064, 1, [[0x420040, 1, [[0x420042, 5, '00000001']]]]],
            ),
            'KMIPKIT-ELEM-OBJECT-STRUCTURE-PUBLIC-KEY': (
                'kmipkit_public_key_from_fields',
                [0x42006d, 1, [[0x420040, 1, [[0x420042, 5, '00000001']]]]],
            ),
            'KMIPKIT-ELEM-OBJECT-STRUCTURE-OPAQUE-OBJECT': (
                'kmipkit_opaque_object_from_fields',
                [0x42005b, 1, [[0x420059, 5, '80000000'], [0x42005a, 8, '6f7061717565']]],
            ),
        }
        recipes = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)
        for target_id, (constructor, expected_tree) in cases.items():
            with self.subTest(target=target_id):
                target = next(item for item in catalog['normative_targets'] if item['target_id'] == target_id)
                result = recipes.execute(target, '2.1')
                self.assertEqual(result['normalized_tree'], expected_tree)
                self.assertIn(constructor, result['invoked_symbols'])

    def test_key_block_model_recipe_invokes_field_and_item_converters(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-BLOCK')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [0x420040, 1, [[0x420042, 5, '00000001']]])
        self.assertTrue({'kmipkit_key_block_from_fields', 'kmipkit_key_block_from_item'}
                        .issubset(result['invoked_symbols']))

    def test_key_value_model_recipe_invokes_format_scoped_converters(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-VALUE')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [0x420045, 1, [[0x420043, 8, 'a1']]])
        self.assertTrue({'kmipkit_key_value_from_fields', 'kmipkit_key_value_from_item'}
                        .issubset(result['invoked_symbols']))

    def test_secret_data_model_recipe_covers_required_key_material(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-SECRET-DATA')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [0x420085, 1, [
            [0x420086, 5, '00000001'],
            [0x420040, 1, [[0x420042, 5, '00000001'],
                           [0x420045, 1, [[0x420043, 8, 'a1']]]]],
        ]])
        self.assertTrue({'kmipkit_secret_data_from_fields', 'kmipkit_key_block_from_fields',
                         'kmipkit_key_block_from_item'}.issubset(result['invoked_symbols']))

    def test_split_key_model_recipe_preserves_source_order_and_method(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-SPLIT-KEY')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [0x420089, 1, [
            [0x42008b, 2, '00000002'], [0x420044, 2, '00000002'],
            [0x42008c, 2, '00000001'], [0x42008a, 5, '00000001'],
            [0x420040, 1, [[0x420042, 5, '00000001']]],
        ]])
        self.assertTrue({'kmipkit_split_key_from_fields', 'kmipkit_key_block_from_fields',
                         'kmipkit_key_block_from_item'}.issubset(result['invoked_symbols']))

    def test_cryptographic_parameters_recipe_preserves_the_exact_catalog_enum(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-GCM-00000009')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['sample_count'], 1)
        self.assertEqual(result['normalized_tree'], [
            [0x420011, 5, '00000009'], [0x4200cd, 2, '0000000c'],
            [0x4200ce, 2, '00000010'],
        ])
        self.assertTrue({
            'kmipkit_cryptographic_parameters_from_fields_v1',
            'kmipkit_cryptographic_parameters_read_fields_v1',
            'kmipkit_cryptographic_parameters_into_ttlv',
            'kmipkit_cryptographic_parameters_from_ttlv',
            'kmipkit_ttlv_structure_view',
        }.issubset(result['invoked_symbols']))

    def test_cryptographic_parameters_recipe_checks_boolean_structure_members(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-RANDOM-IV')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [[0x4200c5, 6, '0000000000000000']])
        self.assertIn('kmipkit_cryptographic_parameters_from_fields_v1', result['invoked_symbols'])

    def test_cryptographic_parameters_recipe_roundtrips_enum_extension_boundaries(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-EXTENSIONS-8XXXXXXX')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['sample_count'], 2)
        self.assertEqual(result['normalized_tree'], [
            [[0x420011, 5, '80000000']], [[0x420011, 5, '8fffffff']],
        ])
        self.assertIn('kmipkit_cryptographic_parameters_from_fields_v1', result['invoked_symbols'])

    def test_cancel_request_constructor_and_encoder_cover_both_payload_versions(self):
        """OASIS KMIP 2.0 §6.1.5 Table 161 and 2.1 §6.1.5 Table 176 define Cancel request payloads."""
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        tags = self.runner.catalog_tag_values(catalog)
        models = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)
        expected_symbols = {
            'kmipkit_cancel_request_create',
            'kmipkit_cancel_request_to_ttlv_payload',
        }
        for target_id in (
            'KMIPKIT-ELEM-OP-C2S-CANCEL',
            'KMIPKIT-PAYLOAD-CLIENT-CANCEL-REQUEST-20-001',
            'KMIPKIT-PAYLOAD-CLIENT-CANCEL-REQUEST-21-001',
        ):
            for version in ('2.0', '2.1'):
                for symbol in expected_symbols:
                    with self.subTest(target_id=target_id, protocol_version=version, symbol=symbol):
                        result = models.execute(targets[target_id], version, symbol)
                        self.assertIsNotNone(result)
                        self.assertIn(symbol, result['invoked_symbols'])
                        self.assertEqual(result['normalized_tree'], [[
                            tags['Asynchronous Correlation Value'], 8,
                            b'fixture-cancel-request'.hex()]])

    def test_async_cancel_response_payload_roundtrips_each_catalogued_result(self):
        """OASIS KMIP 2.0 §6.1.5 Table 162 and 2.1 §6.1.5 Table 177 define Cancellation Result."""
        from artifact_support import INSTALL, ROOT
        from route_ttlv import TreeAbi, encode

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        cases = {case['case_id']: case for case in catalog['test_cases']
                 if case['case_id'].startswith('test-async-cancel-result-')}
        tags = self.runner.catalog_tag_values(catalog)
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        response_targets = (
            'KMIPKIT-PAYLOAD-CLIENT-CANCEL-RESPONSE-20-002',
            'KMIPKIT-PAYLOAD-CLIENT-CANCEL-RESPONSE-21-002',
        )
        for case in cases.values():
            value_name = case['requires']['protocol_scenario']['parameters']['cancel_result']
            for target_id in response_targets:
                for version in ('2.0', '2.1'):
                    with self.subTest(case_id=case['case_id'], target_id=target_id,
                                      protocol_version=version):
                        value = self.runner.catalog_enum_value(
                            catalog, 'Cancellation Result', value_name, version)
                        fixture = self.runner.generic_tree_fixtures(
                            targets[target_id], catalog, tags, version, case=case)
                        self.assertEqual(fixture, [(tags['Cancellation Result'], 5,
                                                    value.to_bytes(4, 'big'))])
                        result = abi.roundtrip(encode(fixture[0]))
                        self.assertEqual(result['normalized_tree'], [
                            tags['Cancellation Result'], 5, value.to_bytes(4, 'big').hex()])
                        self.assertIn('kmipkit_ttlv_decode', result['invoked_symbols'])

    def test_query_peer_driver_checks_typed_response_ttlv_accessor(self):
        root = Path(__file__).resolve().parents[3]
        source = (root / 'interop/c-kmip-validator/src/live_driver.c').read_text()
        self.assertTrue('kmipkit_client_batch_item_response_query_ttlv' in source,
                        'Query driver does not inspect the typed TTLV response accessor')
        self.assertTrue('kmipkit_ttlv_structure_view_item_count' in source,
                        'Query driver does not compare the typed response field count')

    def test_key_block_route_preserves_exact_cryptographic_algorithm(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-AES-00000003')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_key_block_from_fields')
        self.assertEqual(result['normalized_tree'], [0x420040, 1, [
            [0x420042, 5, '00000001'], [0x420028, 5, '00000003'],
            [0x42002a, 2, '00000080'],
        ]])
        self.assertIn('kmipkit_key_block_from_fields', result['invoked_symbols'])

    def test_key_block_route_roundtrips_transparent_rsa_public_key_material(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-3-8-KEY-MATERIAL')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_key_block_from_fields')
        tags = {item['name']: int(item['wire_value'], 16) for item in catalog['normative_targets']
                if item['kind'] == 'tag'}
        self.assertIsNotNone(result)
        self.assertEqual(result['normalized_tree'], [tags['Key Block'], 1, [
            [tags['Key Format Type'], 5, '0000000b'],
            [tags['Key Value'], 1, [[tags['Key Material'], 1, [
                [tags['Modulus'], 4, '01'], [tags['Public Exponent'], 4, '010001'],
            ]]]],
        ]])
        self.assertTrue({'kmipkit_key_block_from_fields', 'kmipkit_key_block_from_item'}
                        .issubset(result['invoked_symbols']))

    def test_key_block_route_roundtrips_each_transparent_key_material_format(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {item['target_id']: item for item in catalog['normative_targets']}
        tags = {item['name']: int(item['wire_value'], 16) for item in catalog['normative_targets']
                if item['kind'] == 'tag'}
        cases = {
            4: (7, [('Key', 8)]),
            5: (8, [('P', 4), ('Q', 4), ('G', 4), ('X', 4)]),
            6: (9, [('P', 4), ('Q', 4), ('G', 4), ('Y', 4)]),
            7: (10, [('Modulus', 4), ('Public Exponent', 4), ('Private Exponent', 4),
                     ('P', 4), ('Q', 4), ('Prime Exponent P', 4),
                     ('Prime Exponent Q', 4), ('CRT Coefficient', 4)]),
            8: (11, [('Modulus', 4), ('Public Exponent', 4)]),
            9: (12, [('P', 4), ('Q', 4), ('G', 4), ('J', 4), ('X', 4)]),
            10: (13, [('P', 4), ('Q', 4), ('G', 4), ('J', 4), ('Y', 4)]),
            11: (20, [('Recommended Curve', 5), ('D', 4)]),
            12: (21, [('Recommended Curve', 5), ('Q String', 8)]),
        }
        models = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)
        for section, (format_value, members) in cases.items():
            target = targets[f'KMIPKIT-ELEM-OBJECT-STRUCTURE-3-{section}-KEY-MATERIAL']
            for version in ('2.0', '2.1'):
                with self.subTest(section=section, version=version):
                    result = models.execute(target, version, 'kmipkit_key_block_from_fields')
                    self.assertIsNotNone(result)
                    key_block_fields = result['normalized_tree'][2]
                    self.assertEqual(key_block_fields[0], [tags['Key Format Type'], 5,
                                                           f'{format_value:08x}'])
                    material_fields = key_block_fields[1][2][0][2]
                    self.assertEqual([(field[0], field[1]) for field in material_fields],
                                     [(tags[name], item_type) for name, item_type in members])
                    self.assertTrue({'kmipkit_key_block_from_fields', 'kmipkit_key_block_from_item'}
                                    .issubset(result['invoked_symbols']))

    def test_key_block_route_roundtrips_optional_key_value_attributes(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {item['target_id']: item for item in catalog['normative_targets']}
        tags = {item['name']: int(item['wire_value'], 16) for item in catalog['normative_targets']
                if item['kind'] == 'tag'}
        target = targets['KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-ATTRIBUTES']
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_key_block_from_fields')
        self.assertIsNotNone(result)
        self.assertEqual(result['normalized_tree'], [tags['Key Block'], 1, [
            [tags['Key Format Type'], 5, '00000001'],
            [tags['Key Value'], 1, [[tags['Key Material'], 8, 'a1'],
                                    [tags['Attributes'], 1, []]]],
        ]])
        self.assertIn('kmipkit_key_block_from_fields', result['invoked_symbols'])

    def test_secret_ttlv_value_routes_preserve_only_digest_evidence(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {item['target_id']: item for item in catalog['normative_targets']}
        cases = [
            ('KMIPKIT-ELEM-DATA-TYPE-BYTE-STRING', 'kmipkit_secret_bytes_into_ttlv_value', 6),
            ('KMIPKIT-ELEM-DATA-TYPE-TEXT-STRING', 'kmipkit_secret_text_into_ttlv_value', 6),
        ]
        models = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)
        for target_id, symbol, samples in cases:
            with self.subTest(symbol=symbol):
                result = models.execute(targets[target_id], '2.1', symbol)
                self.assertIsNotNone(result)
                self.assertEqual(result['sample_count'], samples)
                self.assertRegex(result['tree_sha256'], r'^[0-9a-f]{64}$')
                self.assertIn(symbol, result['invoked_symbols'])
                self.assertNotIn('normalized_tree', result)
                if 'text' in symbol:
                    self.assertEqual(result.get('negative_checks'), 1)
                else:
                    self.assertNotIn('negative_checks', result)

    def test_key_block_route_constructs_wrapping_information_and_key_value(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-WRAPPING-DATA')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_key_block_from_fields')
        self.assertTrue(any(child[0] == 0x420046 for child in result['normalized_tree'][2]))
        self.assertIn('kmipkit_key_block_from_fields', result['invoked_symbols'])

    def test_key_block_route_roundtrips_cryptographic_algorithm_extension_boundaries(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EXTENSIONS-8XXXXXXX')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_key_block_from_fields')
        self.assertEqual(result['sample_count'], 2)
        self.assertEqual([sample[2][1][2] for sample in result['normalized_tree']],
                         ['80000000', '8fffffff'])


    def test_compiled_consumer_provenance_cannot_be_replaced_by_coordinator(self):
        import route_peer
        self.assertTrue(hasattr(route_peer, 'verify_consumer_provenance'), 'consumer provenance check is missing')
        from artifact_support import PIN
        good = {'execution': {'library_commit': PIN, 'artifact_sha256': 'a' * 64},
                'catalog_sha256': 'b' * 64}
        route_peer.verify_consumer_provenance(good, 'a' * 64, 'b' * 64)
        with self.assertRaisesRegex(ValueError, 'consumer_artifact_provenance_mismatch'):
            route_peer.verify_consumer_provenance(good, 'c' * 64, 'b' * 64)


    def tree_abi(self):
        self.assertIsNotNone(importlib.util.find_spec('route_ttlv'), 'recursive public C TTLV executor is missing')
        from route_ttlv import TreeAbi
        from artifact_support import INSTALL
        return TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')

    def test_recursive_ttlv_preserves_nested_order_and_values(self):
        abi = self.tree_abi()
        wire = bytes.fromhex('42000101000000204200010200000004000000010000000042000205000000040000000200000000')
        result = abi.roundtrip(wire)
        self.assertEqual(result['decoded_wire'], wire)
        self.assertEqual(result['constructed_wire'], wire)
        self.assertIn('kmipkit_ttlv_value_structure', result['invoked_symbols'])

    def test_every_normative_ttlv_boundary_vector_is_executed(self):
        from artifact_support import ROOT
        vectors = json.loads((ROOT / 'interop/shared/vectors/ttlv-boundaries.json').read_text())
        for vector in vectors['vectors']:
            with self.subTest(data_type=vector['data_type']):
                result = self.tree_abi().boundary_case(vector)
                self.assertEqual(result['sample_count'], len(vector['samples']))
                self.assertEqual(result['failed_samples'], 0)
                self.assertIn('kmipkit_ttlv_decode', result['invoked_symbols'])

    def test_boolean_public_constructor_rejects_non_boolean_value(self):
        abi = self.tree_abi()
        self.assertTrue(abi.reject_invalid_boolean())


    def value_executor(self):
        self.assertIsNotNone(importlib.util.find_spec('route_values'), 'named-value C case executor is missing')
        from route_values import ValueCases
        from artifact_support import INSTALL, ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        return ValueCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)

    def test_enum_case_uses_its_named_type_and_preserves_unknown_value(self):
        result = self.value_executor().execute({'requires': {'enum_type': 'Cryptographic Algorithm'}}, '2.1')
        self.assertEqual(result['tag'], 0x420028)
        self.assertGreater(result['known_values_checked'], 20)
        self.assertTrue(result['unknown_value_preserved'])
        self.assertIn('kmipkit_ttlv_value_enumeration', result['invoked_symbols'])

    def test_bitmask_case_executes_every_flag_combination(self):
        result = self.value_executor().execute({'requires': {'bitmask_type': 'Cryptographic Usage Mask',
            'exhaustive_bitmask_combinations': {'flag_wire_values': ['00000001', '00000002', '00000004'],
                                              'combination_count': 8}}}, '2.1')
        self.assertEqual(result['combination_count'], 8)
        self.assertTrue(result['unknown_value_preserved'])
        self.assertEqual(result['tag'], 0x42002C)

    def test_item_type_enum_route_uses_named_enumeration_constructor(self):
        executor = self.value_executor()
        result = executor.execute({'requires': {'enum_type': 'Item Type'}}, '2.1')
        self.assertEqual(result['known_values_checked'], 11)
        expected = sorted(int(target['wire_value'], 16) for target in executor.catalog['normative_targets']
                          if target['kind'] == 'enumeration_value'
                          and target['parent_element_ids'] == ['KMIPKIT-ELEM-ENUMERATION-ITEM-TYPE'])
        self.assertEqual(result['values_checked'], expected)
        self.assertIn('kmipkit_ttlv_value_enumeration', result['invoked_symbols'])

    def test_normative_item_type_value_case_uses_item_type_route_recipe(self):
        """OASIS KMIP 2.0/2.1 §11.23 defines the Item Type values under test."""
        executor = self.value_executor()
        case = next(item for item in executor.catalog['test_cases']
                    if item['case_id'] ==
                    'test-norm-enumeration-value-enum-value-item-type-big-integer-00000004')
        target = next(item for item in executor.catalog['normative_targets']
                      if item['target_id'] == case['requires']['normative_target_id'])

        result = executor.execute(case, '2.1', target)

        self.assertIsNotNone(result)
        self.assertEqual(result['known_values_checked'], 11)
        self.assertIn('kmipkit_ttlv_item_create', result['invoked_symbols'])
        self.assertIn('kmipkit_ttlv_decode', result['invoked_symbols'])


    def test_attribute_request_routes_are_independently_executed(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {'test-op-get-attributes', 'test-op-get-attribute-list'}
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases'] if case['case_id'] in case_ids]}
        events, _, checks = execute_network_cells(ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)
        self.assertTrue(checks and all(check['status'] == 'passed' for check in checks))
        for symbol in ('kmipkit_client_batch_item_get_attributes_from_request',
                       'kmipkit_client_batch_item_get_attribute_list_from_request'):
            self.assertTrue(any(event['public_api_symbol'] == symbol and event['action'] == 'send' for event in events), symbol)

    def test_operation_response_ttlv_accessors_are_executed(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        expected = {
            'test-op-check': 'kmipkit_client_batch_item_response_check_ttlv',
            'test-op-get-attribute-list': 'kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv',
            'test-op-log': 'kmipkit_client_batch_item_response_log_ttlv',
            'test-op-revoke': 'kmipkit_client_batch_item_response_revoke_ttlv',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] in expected]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            protocol_versions=('2.0', '2.1'), network_combinations=(('TTLV', 'raw_tls'),))
        self.assertTrue(checks and all(check['status'] == 'passed' for check in checks))
        for case_id, symbol in expected.items():
            with self.subTest(case_id=case_id):
                self.assertTrue(any(event['case_id'] == case_id and
                                    event['public_api_symbol'] == symbol and
                                    event['action'] == 'decode' and
                                    event['transport'] is None and
                                    event['status'] == 'passed' for event in events), symbol)
                self.assertEqual({event['protocol_version'] for event in events
                                  if event['case_id'] == case_id and
                                  event['public_api_symbol'] == symbol}, {'2.0', '2.1'})

    @staticmethod
    def _delegated_login_route_inputs():
        from artifact_support import ROOT

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == 'test-op-delegated-login']}
        return ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest

    def test_delegated_login_response_accessors_read_the_ticket_for_both_versions(self):
        """OASIS KMIP v2.0 §6.1.12 Table 185 and v2.1 §6.1.12 Table 200."""
        from route_peer import execute_network_cells

        binary, selected, manifest = self._delegated_login_route_inputs()
        events, _, checks = execute_network_cells(
            binary, selected, manifest,
            network_combinations=(('TTLV', 'raw_tls'),))

        self.assertEqual(len(checks), 2)
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        symbols = {'kmipkit_client_batch_item_response_delegated_login',
                   'kmipkit_client_batch_item_response_delegated_login_ttlv'}
        self.assertEqual({(event['protocol_version'], event['public_api_symbol'])
                          for event in events if event['public_api_symbol'] in symbols and
                          event['transport'] is None and event['action'] == 'decode' and
                          event['status'] == 'passed'},
                         {(version, symbol) for version in ('2.0', '2.1') for symbol in symbols})
        for check in checks:
            self.assertTrue(symbols <= set(check['public_api_trace_symbols']))
        self.assertNotIn('controlled-delegated-login-ticket', json.dumps([events, checks]))

    def test_delegated_login_response_accessors_reject_changed_ticket_bytes(self):
        """OASIS KMIP v2.0/v2.1 §6.1.12: observe the returned Ticket value."""
        from unittest.mock import patch
        import route_peer

        binary, selected, manifest = self._delegated_login_route_inputs()
        fixture_response = route_peer.response

        def changed_ticket(*args, **kwargs):
            value = b'controlled-delegated-login-ticket'
            return fixture_response(*args, **kwargs).replace(value, b'x' * len(value))

        with patch.object(route_peer, 'response', side_effect=changed_ticket):
            events, _, checks = route_peer.execute_network_cells(
                binary, selected, manifest,
                network_combinations=(('TTLV', 'raw_tls'),))
        self.assertEqual(len(checks), 2)
        self.assertTrue(all(check['status'] == 'failed' for check in checks), checks)
        self.assertFalse(any(event['status'] == 'passed' for event in events))

    def test_add_attribute_operation_runs_from_the_installed_c_consumer(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-op-add-attribute'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 24,
                         'direct and standalone Add Attribute must run for both versions and six wire paths')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertEqual({check['constructor_variant'] for check in checks}, {'direct', 'standalone'})
        self.assertTrue(any(event['public_api_symbol'] == 'kmipkit_client_batch_item_add_attribute'
                            and event['action'] == 'send' for event in events))
        self.assertTrue(any(event['public_api_symbol'] ==
                            'kmipkit_client_batch_item_response_add_attribute'
                            and event['action'] == 'decode' for event in events))
        self.assertTrue(any(event['public_api_symbol'] ==
                            'kmipkit_add_attribute_request_to_ttlv_payload'
                            and event['action'] == 'encode' for event in events))
        for check in checks:
            self.assertEqual(check['peer_setup_operations'], [1])
            self.assertEqual(check['peer_request_operations'], [13])
            self.assertEqual(check['peer_observed_effects'], ['attribute_added'])
            self.assertEqual(check['peer_cleanup_operations'], [19, 20])
            self.assertTrue(check['peer_attribute_state_changed'])
            self.assertTrue(check['peer_final_state_matches_setup'])
            self.assertIn('kmipkit_client_batch_item_destroy', check['public_api_trace_symbols'])
        standalone_checks = [check for check in checks if check['constructor_variant'] == 'standalone']
        for check in standalone_checks:
            self.assertIn('kmipkit_client_batch_item_add_attribute_from_request',
                          check['public_api_trace_symbols'])
        send_events = [event for event in events
                       if event['case_id'] == case_id and event['action'] == 'send']
        for version in ('2.0', '2.1'):
            for encoding in ('TTLV', 'JSON', 'XML'):
                for transport in ('raw_tls', 'https'):
                    cell_events = [event for event in send_events
                                   if event['protocol_version'] == version
                                   and event['encoding'] == encoding
                                   and event['transport'] == transport]
                    direct = [event for event in cell_events
                              if event['public_api_symbol'] ==
                              'kmipkit_client_batch_item_add_attribute']
                    standalone = [event for event in cell_events
                                  if event['public_api_symbol'] ==
                                  'kmipkit_client_batch_item_add_attribute_from_request']
                    self.assertEqual(len(direct), 1)
                    self.assertEqual(len(standalone), 1)
                    self.assertEqual(direct[0]['request_tree_sha256'],
                                     standalone[0]['request_tree_sha256'])
        decoded_responses = [event for event in events
                             if event['case_id'] == case_id and event['action'] == 'decode'
                             and event['public_api_symbol'] ==
                             'kmipkit_client_batch_item_response_add_attribute']
        for version in ('2.0', '2.1'):
            version_responses = [event['response_tree_sha256'] for event in decoded_responses
                                 if event['protocol_version'] == version]
            self.assertGreaterEqual(len(version_responses), 1)
            self.assertEqual(len(set(version_responses)), 1)

    def test_set_attribute_operation_runs_from_the_installed_c_consumer(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-op-set-attribute'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 24,
                         'direct and standalone Set Attribute must run for both versions and six wire paths')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertEqual({check['constructor_variant'] for check in checks}, {'direct', 'standalone'})
        self.assertTrue(all(check['peer_setup_operations'] == [1] for check in checks))
        self.assertTrue(all(check['peer_request_operations'] == [49] for check in checks))
        self.assertTrue(all(check['peer_observed_effects'] == ['attribute_set'] for check in checks))
        self.assertTrue(all(check['peer_cleanup_operations'] == [19, 20] for check in checks))
        self.assertTrue(all(check['peer_attribute_state_changed'] for check in checks))
        self.assertTrue(all(check['peer_final_state_matches_setup'] for check in checks))
        self.assertTrue(any(event['public_api_symbol'] == 'kmipkit_client_batch_item_set_attribute'
                            and event['action'] == 'send' for event in events))
        self.assertTrue(any(event['public_api_symbol'] ==
                            'kmipkit_client_batch_item_response_set_attribute'
                            and event['action'] == 'decode' for event in events))
        self.assertTrue(any(event['public_api_symbol'] ==
                            'kmipkit_set_attribute_request_to_ttlv_payload'
                            and event['action'] == 'encode' for event in events))
        send_events = [event for event in events
                       if event['case_id'] == case_id and event['action'] == 'send']
        for version in ('2.0', '2.1'):
            for encoding in ('TTLV', 'JSON', 'XML'):
                for transport in ('raw_tls', 'https'):
                    cell_events = [event for event in send_events
                                   if event['protocol_version'] == version
                                   and event['encoding'] == encoding
                                   and event['transport'] == transport]
                    direct = [event for event in cell_events
                              if event['public_api_symbol'] == 'kmipkit_client_batch_item_set_attribute']
                    standalone = [event for event in cell_events
                                  if event['public_api_symbol'] ==
                                  'kmipkit_client_batch_item_set_attribute_from_request']
                    self.assertEqual(len(direct), 1)
                    self.assertEqual(len(standalone), 1)
                    self.assertEqual(direct[0]['request_tree_sha256'], standalone[0]['request_tree_sha256'])

    def test_set_constraints_runs_over_every_supported_wire_path(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-op-set-constraints'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            protocol_versions=('2.1',))

        self.assertEqual(len(checks), 6,
                         'Set Constraints must run for all six encoding and transport combinations')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertTrue(any(event['target_id'] == 'KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS'
                            and event['action'] == 'send' for event in events))
        self.assertTrue(any(event['public_api_symbol'] ==
                            'kmipkit_client_batch_item_set_constraints'
                            and event['action'] == 'send' for event in events))
        self.assertTrue(any(event['public_api_symbol'] ==
                            'kmipkit_client_batch_item_response_set_constraints_ttlv'
                            and event['action'] == 'decode' for event in events))

    def test_set_defaults_runs_over_every_supported_wire_path(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-op-set-defaults'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            protocol_versions=('2.1',))

        self.assertEqual(len(checks), 6,
                         'Set Defaults must run for all six encoding and transport combinations')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertTrue(any(event['target_id'] == 'KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS'
                            and event['action'] == 'send' for event in events))
        self.assertTrue(any(event['public_api_symbol'] ==
                            'kmipkit_client_batch_item_set_defaults'
                            and event['action'] == 'send' for event in events))
        self.assertTrue(any(event['public_api_symbol'] ==
                            'kmipkit_client_batch_item_response_set_defaults_ttlv'
                            and event['action'] == 'decode' for event in events))

    def test_delete_attribute_operation_runs_from_the_installed_c_consumer(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-op-delete-attribute'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 24,
                         'direct and standalone Delete Attribute must run for both versions and six wire paths')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertEqual({check['constructor_variant'] for check in checks}, {'direct', 'standalone'})
        self.assertTrue(all(check['peer_setup_operations'] == [1] for check in checks))
        self.assertTrue(all(check['peer_seed_operations'] == [13] for check in checks))
        self.assertTrue(all(check['peer_request_operations'] == [15] for check in checks))
        self.assertTrue(all(check['peer_observed_effects'] == ['attribute_deleted'] for check in checks))
        self.assertTrue(all(check['peer_cleanup_operations'] == [19, 20] for check in checks))
        self.assertTrue(all(check['peer_attribute_state_changed'] for check in checks))
        self.assertTrue(all(check['peer_final_state_matches_setup'] for check in checks))
        self.assertTrue(any(event['public_api_symbol'] == 'kmipkit_client_batch_item_delete_attribute'
                            and event['action'] == 'send' for event in events))
        self.assertTrue(any(event['public_api_symbol'] ==
                            'kmipkit_client_batch_item_response_delete_attribute'
                            and event['action'] == 'decode' for event in events))
        self.assertTrue(any(event['public_api_symbol'] ==
                            'kmipkit_delete_attribute_request_to_ttlv_payload'
                            and event['action'] == 'encode' for event in events))
        send_events = [event for event in events
                       if event['case_id'] == case_id and event['action'] == 'send']
        for version in ('2.0', '2.1'):
            for encoding in ('TTLV', 'JSON', 'XML'):
                for transport in ('raw_tls', 'https'):
                    cell_events = [event for event in send_events
                                   if event['protocol_version'] == version
                                   and event['encoding'] == encoding
                                   and event['transport'] == transport]
                    direct = [event for event in cell_events
                              if event['public_api_symbol'] ==
                              'kmipkit_client_batch_item_delete_attribute']
                    standalone = [event for event in cell_events
                                  if event['public_api_symbol'] ==
                                  'kmipkit_client_batch_item_delete_attribute_from_request']
                    self.assertEqual(len(direct), 1)
                    self.assertEqual(len(standalone), 1)
                    self.assertEqual(direct[0]['request_tree_sha256'], standalone[0]['request_tree_sha256'])

    def test_delete_attribute_setup_rejections_are_classified_by_result_reason(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-op-delete-attribute'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}

        for reason, expected_compatibility in ((5, 'unsupported'), (12, 'policy_restricted')):
            with self.subTest(reason=reason):
                _, _, checks = execute_network_cells(
                    ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
                    protocol_versions=('2.1',), network_combinations=(('TTLV', 'raw_tls'),),
                    peer_rejection={'case_id': case_id, 'exchange_index': 1,
                                    'operation': 13, 'result_status': 1,
                                    'result_reason': reason})

                self.assertEqual(len(checks), 2)
                self.assertTrue(all(check['status'] == 'passed' for check in checks))
                self.assertTrue(all(check['consumer_server_compatibility'] ==
                                    expected_compatibility for check in checks))
                self.assertTrue(all(check['consumer_library_correctness'] == 'passed'
                                    for check in checks))
                self.assertTrue(all(check['consumer_result_status_code'] == 1
                                    for check in checks))
                self.assertTrue(all(check['consumer_result_reason_code'] == reason
                                    for check in checks))
                self.assertTrue(all('kmipkit_client_batch_item_delete_attribute' not in
                                    check['public_api_trace_symbols'] for check in checks))

    def test_locate_response_reads_identifiers_for_both_protocol_versions(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-op-locate'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            protocol_versions=('2.0', '2.1'), network_combinations=(('TTLV', 'raw_tls'),))

        self.assertEqual(len(checks), 2)
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        identifier_reads = [event for event in events
                            if event['case_id'] == case_id and event['action'] == 'decode'
                            and event['public_api_symbol'] ==
                            'kmipkit_client_batch_item_response_locate_unique_identifier_at']
        self.assertEqual({event['protocol_version'] for event in identifier_reads}, {'2.0', '2.1'})


    def test_read_back_attribute_recipe_calls_get_attributes_with_exact_tag(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-attr-activation-date'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases'] if case['case_id'] == case_id]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            protocol_versions=('2.1',), network_combinations=(('TTLV', 'raw_tls'),))
        self.assertEqual(len(checks), 2, 'both Get Attributes constructors must run against the controlled peer')
        self.assertEqual({check['constructor_variant'] for check in checks}, {'direct', 'standalone'})
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertTrue(all(check['request_attribute_reference_tags'] == ['420001'] for check in checks))
        self.assertTrue(all(check['response_attribute_tags'] == ['420001'] for check in checks))
        for symbol in ('kmipkit_client_batch_item_get_attributes',
                       'kmipkit_client_batch_item_get_attributes_from_request'):
            self.assertTrue(any(event['public_api_symbol'] == symbol and event['action'] == 'send'
                                for event in events), symbol)


    def test_vendor_attribute_read_back_uses_vendor_name_reference(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-attr-vendor-attribute'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases'] if case['case_id'] == case_id]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            protocol_versions=('2.1',), network_combinations=(('TTLV', 'raw_tls'),))
        self.assertEqual(len(checks), 2, 'vendor attribute must use both Get Attributes constructors')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertTrue(all(check['request_attribute_reference_tags'] == [] for check in checks))
        self.assertTrue(all(check['request_attribute_reference_names'] == ['fixture_attribute']
                            for check in checks))
        self.assertTrue(all(check['request_vendor_identifications'] == ['fixture_vendor']
                            for check in checks))
        self.assertTrue(all(check['response_attribute_tags'] == ['420008'] for check in checks))
        for symbol in ('kmipkit_client_batch_item_get_attributes',
                       'kmipkit_client_batch_item_get_attributes_from_request'):
            self.assertTrue(any(event['public_api_symbol'] == symbol and event['action'] == 'send'
                                for event in events), symbol)


    def test_certificate_attributes_read_back_all_section_46_fields_without_reference(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-attr-certificate-attributes'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases'] if case['case_id'] == case_id]}
        field_names = [
            'Certificate Subject CN', 'Certificate Subject O', 'Certificate Subject OU',
            'Certificate Subject Email', 'Certificate Subject C', 'Certificate Subject ST',
            'Certificate Subject L', 'Certificate Subject UID', 'Certificate Subject Serial Number',
            'Certificate Subject Title', 'Certificate Subject DC', 'Certificate Subject DN Qualifier',
            'Certificate Issuer CN', 'Certificate Issuer O', 'Certificate Issuer OU',
            'Certificate Issuer Email', 'Certificate Issuer C', 'Certificate Issuer ST',
            'Certificate Issuer L', 'Certificate Issuer UID', 'Certificate Issuer Serial Number',
            'Certificate Issuer Title', 'Certificate Issuer DC', 'Certificate Issuer DN Qualifier',
        ]
        tags = {target['name']: target['wire_value'].upper() for target in catalog['normative_targets']
                if target['kind'] == 'tag' and target['name'] in field_names}
        expected_tags = [tags[name] for name in field_names]
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            protocol_versions=('2.1',), network_combinations=(('TTLV', 'raw_tls'),))
        self.assertEqual(len(checks), 2, 'certificate attribute aggregate must use both Get Attributes constructors')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertTrue(all(check['request_attribute_reference_tags'] == [] and
                            check['request_attribute_reference_names'] == [] for check in checks))
        required_register_attributes = {'42001D', '420028', '42002A', '42002C', '4200AD'}
        self.assertTrue(all(required_register_attributes <=
                            set(check['request_certificate_attribute_tags']) for check in checks))
        self.assertTrue(all(check['response_attribute_tags'] == expected_tags for check in checks))
        for symbol in ('kmipkit_client_batch_item_get_attributes',
                       'kmipkit_client_batch_item_get_attributes_from_request'):
            self.assertTrue(any(event['public_api_symbol'] == symbol and event['action'] == 'send'
                                for event in events), symbol)

    @staticmethod
    def _batch_register_crypto_route_inputs():
        from artifact_support import ROOT

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] ==
                                               'test-batch-register-encrypt-decrypt-destroy']}
        return ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest

    def test_batch_register_crypto_checks_every_typed_route_and_cleans_the_object(self):
        """OASIS KMIP 2.0 §§6.1.39–6.1.40; KMIP 2.1 §§6.1.43–6.1.44.

        Both versions §§6.1.1, 6.1.11, 6.1.15, 6.1.17, 8.2–8.4, and 8.6.
        """
        from coverage_matrix import iter_cells
        from route_peer import execute_network_cells
        from validate_route_evidence import _key

        binary, selected, manifest = self._batch_register_crypto_route_inputs()
        events, keys, checks = execute_network_cells(binary, selected, manifest)
        expected = {_key(cell) for cell in iter_cells(selected, manifest, languages=('c',))
                    if cell['route_id'].startswith('c.typed-protocol.')}

        self.assertEqual(len(checks), 12, 'one six-item batch must run on every version/wire combination')
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        self.assertEqual(len(expected), 304)
        self.assertEqual(keys, expected)
        self.assertEqual(len(events), len(expected))
        for check in checks:
            self.assertTrue(check['peer_final_state_matches_setup'])
        for event in events:
            if event['action'] != 'send':
                continue
            self.assertEqual(event['batch_operation_sequence'],
                             ['Register', 'Activate', 'Encrypt', 'Decrypt', 'Revoke', 'Destroy'])
            self.assertEqual(event['peer_observed_result_status_codes'], [0] * 6)
            self.assertEqual(event['peer_observed_effects'],
                             ['created', 'activated', 'encrypted', 'decrypted', 'revoked', 'destroyed'])
            self.assertEqual(event['peer_observed_state_before_sha256'],
                             event['peer_observed_state_after_sha256'])
        result_text = json.dumps([events, checks])
        for fixture in ('000102030405060708090a0b0c0d0e0f',
                        '00112233445566778899aabbccddeeff',
                        '69c4e0d86a7b0430d8cdb78070b4c55a'):
            self.assertNotIn(fixture, result_text)

    def test_batch_register_crypto_rejects_changed_ciphertext_after_cleanup(self):
        """OASIS KMIP 2.0/2.1 §6.1.17: compare returned Data, not only its presence."""
        from unittest.mock import patch
        import route_peer

        binary, selected, manifest = self._batch_register_crypto_route_inputs()
        fixture_response = route_peer.response

        def changed_ciphertext(*args, **kwargs):
            ciphertext = bytes.fromhex('69c4e0d86a7b0430d8cdb78070b4c55a')
            return fixture_response(*args, **kwargs).replace(ciphertext, bytes(len(ciphertext)))

        with patch.object(route_peer, 'response', side_effect=changed_ciphertext):
            events, _, checks = route_peer.execute_network_cells(
                binary, selected, manifest, protocol_versions=('2.1',),
                network_combinations=(('TTLV', 'raw_tls'),))
        self.assertEqual(len(checks), 1)
        self.assertEqual(checks[0]['status'], 'failed')
        self.assertTrue(checks[0]['peer_final_state_matches_setup'])
        self.assertFalse(events)

    def test_batch_create_activate_get_attribute_list_destroy_uses_one_checked_request(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-batch-create-activate-destroy'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)
        self.assertEqual(len(checks), 12, 'batch workflow must pass both versions over all six wire transports')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        expected_operations = {
            'KMIPKIT-ELEM-OP-C2S-CREATE', 'KMIPKIT-ELEM-OP-C2S-ACTIVATE',
            'KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST', 'KMIPKIT-ELEM-OP-C2S-REVOKE',
            'KMIPKIT-ELEM-OP-C2S-DESTROY',
        }
        self.assertTrue(expected_operations <= {event['target_id'] for event in events})
        for symbol in ('kmipkit_client_batch_item_create', 'kmipkit_client_batch_item_activate',
                       'kmipkit_client_batch_item_get_attribute_list',
                       'kmipkit_client_batch_item_revoke', 'kmipkit_client_batch_item_destroy'):
            self.assertTrue(any(event['public_api_symbol'] == symbol and event['action'] == 'send'
                                for event in events), symbol)
        self.assertTrue(all('kmipkit_client_execute' in check['public_api_trace_symbols']
                            for check in checks))

    def test_id_placeholder_accepts_omission_or_reserved_enumeration_only(self):
        from route_peer import assert_placeholder_usage, parse, tlv

        def enumeration(tag, value):
            return tlv(tag, 5, value.to_bytes(4, 'big'))

        def batch_item(operation, unique_identifier=None):
            payload = b'' if unique_identifier is None else unique_identifier
            return tlv(0x42000F, 1, enumeration(0x42005C, operation)
                       + tlv(0x420079, 1, payload))

        items = [batch_item(1)] + [batch_item(operation) for operation in (18, 12, 19, 20)]
        request = parse(tlv(0x420078, 1, b''.join(items)))
        assert_placeholder_usage(request, 'test-batch-create-activate-destroy')

        reserved_placeholder = tlv(0x420094, 5, (1).to_bytes(4, 'big'))
        explicit_placeholder_items = [batch_item(1)] + [
            batch_item(operation, reserved_placeholder) for operation in (18, 12, 19, 20)
        ]
        explicit_placeholder_request = parse(tlv(0x420078, 1, b''.join(explicit_placeholder_items)))
        assert_placeholder_usage(explicit_placeholder_request, 'test-batch-create-activate-destroy')

        request_with_explicit_target = parse(tlv(
            0x420078, 1, b''.join([items[0], items[1], batch_item(
                12, tlv(0x420094, 7, b'fixture-key')), *items[3:]])))
        with self.assertRaisesRegex(ValueError, 'fixture_id_placeholder_identifier_mismatch'):
            assert_placeholder_usage(request_with_explicit_target, 'test-batch-create-activate-destroy')

        request_with_invalid_enumeration = parse(tlv(
            0x420078, 1, b''.join([items[0], batch_item(
                18, tlv(0x420094, 5, (2).to_bytes(4, 'big'))), *items[2:]])))
        with self.assertRaisesRegex(ValueError, 'fixture_id_placeholder_identifier_mismatch'):
            assert_placeholder_usage(request_with_invalid_enumeration, 'test-batch-create-activate-destroy')

    def test_placeholder_chain_case_maps_create_activate_get_recipe(self):
        from route_peer import recipe_for_case

        recipe = recipe_for_case({'case_id': 'test-batch-placeholder-chain-resolves'}, None)

        self.assertEqual(recipe, ((1, 18, 10), 'batch_placeholder_chain'))

    def test_register_operation_maps_to_an_isolated_certificate_recipe(self):
        from route_peer import recipe_for_case

        recipe = recipe_for_case({'case_id': 'test-op-register'}, None)

        self.assertEqual(recipe, ((3, 20), 'register'))

    def test_register_operation_executes_the_installed_request_and_response_routes(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-op-register'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12, 'Register must pass both versions and six encoding/transport cells')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertTrue(any(event['target_id'] == 'KMIPKIT-ELEM-OP-C2S-REGISTER'
                            and event['action'] == 'send' for event in events))
        required_symbols = {
            'kmipkit_register_request_create',
            'kmipkit_client_batch_item_register',
            'kmipkit_client_batch_item_response_register',
            'kmipkit_client_batch_item_response_register_ttlv',
            'kmipkit_client_batch_item_destroy',
            'kmipkit_client_execute',
        }
        self.assertTrue(all(required_symbols <= set(check['public_api_trace_symbols'])
                            for check in checks))
        generic_response_events = [event for event in events
                                   if event['public_api_symbol'] ==
                                   'kmipkit_client_batch_item_response_register_ttlv']
        self.assertEqual({event['protocol_version'] for event in generic_response_events},
                         {'2.0', '2.1'})

    def test_certificate_register_object_routes_execute_from_the_installed_consumer(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        required_symbols = {
            'kmipkit_register_request_create',
            'kmipkit_client_batch_item_register',
            'kmipkit_client_batch_item_response_register',
            'kmipkit_client_batch_item_response_register_ttlv',
            'kmipkit_client_batch_item_destroy',
            'kmipkit_client_execute',
        }
        for case_id, object_type_symbol in (
            ('test-obj-certificate', None),
            ('test-obj-certificate-request', 'kmipkit_certificate_request_from_fields'),
        ):
            with self.subTest(case_id=case_id):
                selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                                       if case['case_id'] == case_id]}
                events, _, checks = execute_network_cells(
                    ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

                self.assertEqual(len(checks), 12)
                self.assertTrue(all(check['status'] == 'passed' for check in checks))
                self.assertTrue(all(required_symbols <= set(check['public_api_trace_symbols'])
                                    for check in checks))
                if object_type_symbol:
                    self.assertTrue(all(object_type_symbol in check['public_api_trace_symbols']
                                        for check in checks))
                register_events = [event for event in events
                                   if event['case_id'] == case_id
                                   and event['target_id'] == 'KMIPKIT-ELEM-OP-C2S-REGISTER']
                self.assertTrue(register_events)

    def test_placeholder_chain_executes_typed_create_activate_get_batch(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-batch-placeholder-chain-resolves'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}

        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12, 'placeholder chain must run over both versions and six wire transports')
        failed_checks = [
            {key: check.get(key) for key in
             ('protocol_version', 'encoding', 'transport', 'status', 'reason', 'result_reason')}
            for check in checks if check['status'] != 'passed'
        ]
        self.assertFalse(failed_checks, f'controlled placeholder-chain failures: {failed_checks[:3]}')
        expected_operations = {
            'KMIPKIT-ELEM-OP-C2S-CREATE',
            'KMIPKIT-ELEM-OP-C2S-ACTIVATE',
            'KMIPKIT-ELEM-OP-C2S-GET',
        }
        self.assertTrue(expected_operations <= {event['target_id'] for event in events})
        create_send = next(event for event in events
                           if event['target_id'] == 'KMIPKIT-ELEM-OP-C2S-CREATE'
                           and event['action'] == 'send')
        self.assertEqual(create_send['peer_observed_operation_sequence'],
                         ['Create', 'Activate', 'Get'])
        for symbol in ('kmipkit_client_batch_item_create', 'kmipkit_client_batch_item_activate',
                       'kmipkit_client_batch_item_get'):
            self.assertTrue(any(event['public_api_symbol'] == symbol and event['action'] == 'send'
                                for event in events), symbol)
        self.assertTrue(all('kmipkit_client_execute' in check['public_api_trace_symbols']
                            for check in checks))


    def test_two_item_batch_continuation_and_id_correlation(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {
            'test-batch-count-matches-items',
            'test-batch-unique-item-id-response-correlation',
            'test-batch-order-false-correlates-response-items',
            'test-batch-continue-after-failure',
        }
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] in case_ids]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)
        self.assertEqual(len(checks), 60,
                         'Get Attribute List must run through direct and request-based constructors')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        expected_operations = {
            'KMIPKIT-ELEM-OP-C2S-GET', 'KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST',
        }
        self.assertTrue(expected_operations <= {event['target_id'] for event in events})
        for symbol in ('kmipkit_client_batch_item_get', 'kmipkit_client_batch_item_get_attribute_list'):
            self.assertTrue(any(event['public_api_symbol'] == symbol and event['action'] == 'send'
                                for event in events), symbol)
        standalone_get_attribute_list = [event for event in events
                                         if event['case_id'] == 'test-batch-continue-after-failure'
                                         and event['public_api_symbol'] ==
                                         'kmipkit_client_batch_item_get_attribute_list_from_request'
                                         and event['target_id'] == 'KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST'
                                         and event.get('workflow_item_index') == 1]
        self.assertEqual(len(standalone_get_attribute_list), 36)
        self.assertEqual({event['action'] for event in standalone_get_attribute_list},
                         {'construct', 'send', 'decode'})
        self.assertEqual({(event['protocol_version'], event['encoding'], event['transport'])
                          for event in standalone_get_attribute_list},
                         {(version, encoding, transport)
                          for version in ('2.0', '2.1')
                          for encoding in ('TTLV', 'JSON', 'XML')
                          for transport in ('raw_tls', 'https')})
        for check in checks:
            symbols = check['public_api_trace_symbols']
            self.assertTrue({'kmipkit_client_execute', 'kmipkit_client_batch_response_item_count',
                             'kmipkit_client_batch_response_item_at',
                             'kmipkit_client_batch_item_response_unique_batch_item_id'} <= set(symbols))
            expected_response_order = [12, 10] if check['case_id'] == \
                'test-batch-order-false-correlates-response-items' else [10, 12]
            self.assertEqual(check['peer_response_operation_values'], expected_response_order)
            expected_statuses = [1, 0] if check['case_id'] == \
                'test-batch-continue-after-failure' else [0, 0]
            self.assertEqual(check['peer_response_result_statuses'], expected_statuses)
            if check['case_id'] == 'test-batch-continue-after-failure':
                self.assertIn('kmipkit_client_batch_add_error_continuation_value', symbols)


    def test_stop_batch_returns_only_the_failed_item_and_stops_processing(self):
        from artifact_support import ROOT
        from route_peer import RECIPES, execute_network_cells
        case_id = 'test-batch-stop-on-first-failure'
        self.assertEqual(RECIPES[case_id], ((10, 10), 'batch_stop_on_first_failure'))
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)

        self.assertEqual(len(checks), 12, 'STOP must pass both versions over all six wire transports')
        self.assertTrue(all(check['status'] == 'passed' for check in checks), checks)
        for check in checks:
            self.assertEqual(check['peer_response_operation_values'], [10])
            self.assertEqual(check['peer_response_result_statuses'], [1])
            self.assertEqual(check['peer_response_unique_batch_item_ids'], ['item-a'])
            self.assertEqual(check['peer_observed_effects'], ['object_not_found'])
            self.assertEqual(check['peer_observed_state_before_sha256'],
                             check['peer_observed_state_after_sha256'])
            self.assertTrue({'kmipkit_client_execute',
                             'kmipkit_client_batch_add_error_continuation_value',
                             'kmipkit_client_batch_item_response_get',
                             'kmipkit_client_batch_item_response_metadata_v1',
                             'kmipkit_client_batch_item_response_pending_correlation_value',
                             'kmipkit_client_batch_item_response_header_ttlv',
                             'kmipkit_client_batch_item_response_raw_ttlv',
                             'kmipkit_client_batch_response_item_count',
                             'kmipkit_client_batch_response_item_at'} <=
                            set(check['public_api_trace_symbols']))
            self.assertTrue({'kmipkit_client_batch_add_error_continuation_value',
                             'kmipkit_client_batch_item_response_get',
                             'kmipkit_client_batch_item_response_metadata_v1',
                             'kmipkit_client_batch_item_response_pending_correlation_value',
                             'kmipkit_client_batch_item_response_header_ttlv',
                             'kmipkit_client_batch_item_response_raw_ttlv'} <=
                            set(check['public_api_success_symbols']))
        send_events = [event for event in events if event['action'] == 'send']
        self.assertEqual(len(send_events), 24, 'each request item route is counted separately')
        self.assertTrue(all(event['batch_operation_sequence'] == ['Get', 'Get']
                            and event['batch_item_count'] == 2
                            and event['peer_observed_response_count'] == 1
                            and event['peer_observed_result_status_codes'] == [1]
                            for event in send_events))
        accessor_symbols = {
            'kmipkit_client_batch_add_error_continuation_value',
            'kmipkit_client_batch_item_response_get',
            'kmipkit_client_batch_item_response_metadata_v1',
            'kmipkit_client_batch_item_response_pending_correlation_value',
            'kmipkit_client_batch_item_response_header_ttlv',
            'kmipkit_client_batch_item_response_raw_ttlv',
        }
        exercised_routes = {
            (event['public_api_symbol'], event['action'], event['target_id'],
             event.get('workflow_item_index'))
            for event in events if event['case_id'] == case_id and
            event['public_api_symbol'] in accessor_symbols
        }
        expected_routes = {
            ('kmipkit_client_batch_add_error_continuation_value', 'construct',
             'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ERROR-CONTINUATION-OPTION', None),
            ('kmipkit_client_batch_add_error_continuation_value', 'construct',
             'KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-STOP-00000002', None),
            ('kmipkit_client_batch_item_response_get', 'decode', 'KMIPKIT-ELEM-OP-C2S-GET', 0),
            ('kmipkit_client_batch_item_response_metadata_v1', 'decode',
             'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM', None),
            ('kmipkit_client_batch_item_response_pending_correlation_value', 'decode',
             'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM', None),
            ('kmipkit_client_batch_item_response_header_ttlv', 'decode',
             'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM', None),
            ('kmipkit_client_batch_item_response_raw_ttlv', 'decode',
             'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM', None),
        }
        self.assertEqual(exercised_routes, expected_routes,
                         'only the failed first response item is available for decoding')


    def test_batch_placeholder_chain_has_an_exact_peer_recipe(self):
        from route_peer import RECIPES
        case_id = 'test-batch-placeholder-chain-resolves'
        self.assertIn(case_id, RECIPES, 'placeholder-chain peer recipe is missing')
        operations, workflow = RECIPES[case_id]
        self.assertEqual((1, 18, 10), operations[:3])
        self.assertEqual('batch_placeholder_chain', workflow)


    def test_batch_placeholder_chain_resolves_the_same_object_and_cleans_up(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-batch-placeholder-chain-resolves'
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)
        self.assertEqual(len(checks), 12, 'placeholder chain must pass both versions over all wire transports')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        self.assertTrue(all(event['peer_observed_operation_sequence'] ==
                            ['Create', 'Activate', 'Get']
                            for event in events if event['action'] == 'send'))
        self.assertTrue(all(check['peer_cleanup_operations'] == [19, 20] for check in checks))
        empty_state = hashlib.sha256(b'{"objects":{}}').hexdigest()
        self.assertTrue(all(check['peer_cleanup_state_after_sha256'] == empty_state
                            for check in checks))
        for symbol in ('kmipkit_client_batch_item_create', 'kmipkit_client_batch_item_activate',
                       'kmipkit_client_batch_item_get'):
            self.assertTrue(any(event['public_api_symbol'] == symbol and event['action'] == 'send'
                                for event in events), symbol)
        self.assertTrue(any(event['execution_api_symbol'] == 'kmipkit_client_execute' and
                            event['action'] == 'send' for event in events))
        for symbol in ('kmipkit_client_batch_item_revoke', 'kmipkit_client_batch_item_destroy'):
            self.assertTrue(all(symbol in check['public_api_trace_symbols'] for check in checks), symbol)


    def test_placeholder_batch_evidence_matches_the_catalog_before_cleanup(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        from validate_route_evidence import _key, _validate_batch_event

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case = next(case for case in catalog['test_cases']
                    if case['case_id'] == 'test-batch-placeholder-chain-resolves')
        selected = {**catalog, 'test_cases': [case]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest,
            protocol_versions=('2.1',), network_combinations=(('TTLV', 'raw_tls'),))
        self.assertEqual(len(checks), 1)
        self.assertEqual(checks[0]['status'], 'passed')
        send_events = [event for event in events if event['action'] == 'send']
        self.assertTrue(send_events)
        for event in send_events:
            try:
                _validate_batch_event(event, case, catalog, _key(event))
            except ValueError as error:
                self.fail(str(error))

    def test_continued_batch_evidence_has_observed_unchanged_fixture_state(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        from validate_route_evidence import _key, _validate_batch_event

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case = next(case for case in catalog['test_cases']
                    if case['case_id'] == 'test-batch-continue-after-failure')
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c',
            {**catalog, 'test_cases': [case]}, manifest,
            protocol_versions=('2.1',), network_combinations=(('TTLV', 'raw_tls'),))
        self.assertEqual(len(checks), 1)
        self.assertEqual(checks[0]['status'], 'passed')
        expected_state = hashlib.sha256(json.dumps(
            {'objects': {'fixture-key-1': 'pre_active'}},
            sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        send_events = [event for event in events if event['action'] == 'send']
        self.assertTrue(send_events)
        for event in send_events:
            self.assertEqual(event.get('peer_observed_effects'),
                             ['first_item_failed', 'second_item_executed'])
            self.assertEqual(event.get('peer_observed_state_before_sha256'), expected_state)
            self.assertEqual(event.get('peer_observed_state_after_sha256'), expected_state)
            try:
                _validate_batch_event(event, case, catalog, _key(event))
            except ValueError as error:
                self.fail(str(error))

    def test_placeholder_state_records_activation_and_independent_deletion(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case = next(case for case in catalog['test_cases']
                    if case['case_id'] == 'test-batch-placeholder-chain-resolves')
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c',
            {**catalog, 'test_cases': [case]}, manifest,
            protocol_versions=('2.1',), network_combinations=(('TTLV', 'raw_tls'),))
        self.assertEqual(len(checks), 1)
        self.assertEqual(checks[0]['status'], 'passed')
        empty_state = hashlib.sha256(b'{"objects":{}}').hexdigest()
        active_state = hashlib.sha256(json.dumps(
            {'objects': {'fixture-key-1': 'active'}},
            sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        send_events = [event for event in events if event['action'] == 'send']
        self.assertTrue(send_events)
        for event in send_events:
            self.assertEqual(event.get('peer_observed_state_before_sha256'), empty_state)
            self.assertEqual(event.get('peer_observed_state_after_sha256'), active_state)
        self.assertEqual(checks[0].get('peer_cleanup_state_after_sha256'), empty_state)

    def test_tag_family_matrix_decodes_each_target_instead_of_a_surrogate_enum(self):
        from artifact_support import ROOT
        from contextlib import redirect_stderr, redirect_stdout
        import io
        from unittest.mock import patch

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        cells = [cell for cell in self.runner.iter_cells(catalog, manifest, languages=('c',))
                 if cell['case_id'] == 'test-enum-tag' and cell['protocol_version'] == '2.1'
                 and cell['public_api_symbol'] == 'kmipkit_ttlv_decode'
                 and cell['target_id'] in {'KMIPKIT-ELEM-TAG-420001', 'KMIPKIT-ELEM-TAG-420009'}]
        self.assertEqual(len(cells), 2)
        with tempfile.TemporaryDirectory(dir=self.runner.OUTPUT_ROOT) as directory:
            output = Path(directory) / 'evidence.json'
            # Select only input cells; installed ABI calls and evidence validation are real.
            with patch.object(self.runner, 'iter_cells', return_value=cells), \
                    redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
                self.assertEqual(self.runner.run(output), 1, 'a subset cannot satisfy the full gate')
            events = {event['target_id']: event for event in json.loads(output.read_text())['events']}
        self.assertEqual(events['KMIPKIT-ELEM-TAG-420009'].get('decode_outcome'), 'rejected')
        self.assertEqual(events['KMIPKIT-ELEM-TAG-420009']['decode_rejection_code'], 'tag_reserved')
        expected_tree = {'tag': 0x420001, 'type': 2, 'value': '00000007'}
        expected_hash = hashlib.sha256(json.dumps(
            expected_tree, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        self.assertEqual(events['KMIPKIT-ELEM-TAG-420001']['response_tree_sha256'], expected_hash)

    def test_single_item_batch_omits_optional_batch_options(self):
        from artifact_support import ROOT
        from route_peer import RECIPES, execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_id = 'test-batch-single-item-options-omitted'
        self.assertIn(case_id, RECIPES, 'single-item batch peer recipe is missing')
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases']
                                               if case['case_id'] == case_id]}
        events, _, checks = execute_network_cells(
            ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)
        self.assertEqual(len(checks), 12, 'single-item batch must pass both versions over all six wire transports')
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        expected_targets = {
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-3-BATCH-ITEM',
            'KMIPKIT-ELEM-OP-C2S-GET',
        }
        self.assertTrue(expected_targets <= {event['target_id'] for event in events})
        self.assertTrue(any(event['public_api_symbol'] == 'kmipkit_client_batch_item_get' and
                            event['action'] == 'send' for event in events))
        self.assertTrue(all('kmipkit_client_execute' in check['public_api_trace_symbols']
                            for check in checks))

    def test_hash_and_rng_cases_call_their_public_routes_and_check_responses(self):
        from artifact_support import ROOT
        from route_peer import RECIPES, execute_network_cells
        case_ids = {
            'test-op-hash', 'test-op-rng-retrieve', 'test-op-rng-seed',
            'test-crypto-hash-sha_256-0', 'test-crypto-rng-retrieve-rng-32',
            'test-crypto-rng-seed-rng-32',
        }
        self.assertTrue(case_ids <= RECIPES.keys(), 'Hash/RNG case-specific peer recipes are missing')
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases'] if case['case_id'] in case_ids]}
        events, _, checks = execute_network_cells(ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)
        self.assertEqual(len(checks), 72)
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        for name in ('hash', 'rng_retrieve', 'rng_seed'):
            self.assertTrue(any(event['public_api_symbol'] == 'kmipkit_client_batch_item_' + name and event['action'] == 'decode' for event in events))


    def attribute_executor(self):
        self.assertIsNotNone(importlib.util.find_spec('route_attributes'), 'typed attribute route executor is missing')
        from route_attributes import AttributeCases
        from artifact_support import INSTALL, ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        return AttributeCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)

    def test_all_typed_attribute_paths_preserve_value_and_reject_wrong_type(self):
        executor = self.attribute_executor()
        target = next(target for target in executor.catalog['normative_targets']
                      if target['target_id'] == 'KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-ALGORITHM')
        result = executor.execute(target, '2.1')
        expected = {'kmipkit_typed_attribute_from_item', 'kmipkit_typed_attribute_from_value',
                    'kmipkit_attribute_set_create', 'kmipkit_attribute_set_try_push',
                    'kmipkit_attribute_set_try_push_typed'}
        self.assertTrue(expected <= result['invoked_symbols'])
        self.assertEqual(result['negative_checks'], 5)
        self.assertGreater(result['sample_count'], 1)

    def test_attribute_local_routes_also_run_for_server_exchange_cases(self):
        self.assertTrue(self.runner.should_execute_attribute_case(
            {'transport': None}, {'category': 'attributes', 'requires': {}}, {'kind': 'attribute'}))
        self.assertTrue(self.runner.should_execute_attribute_case(
            {'transport': None}, {'category': 'attributes', 'requires': {'protocol_scenario': 'lifecycle'}},
            {'kind': 'attribute'}))
        self.assertFalse(self.runner.should_execute_attribute_case(
            {'transport': 'raw_tls'}, {'category': 'attributes', 'requires': {}}, {'kind': 'attribute'}))
        self.assertFalse(self.runner.should_execute_attribute_case(
            {'transport': None}, {'category': 'object_lifecycle', 'requires': {}}, {'kind': 'attribute'}))

    def test_modify_attribute_typed_payload_route_is_inspected_from_installed_c_abi(self):
        """Exercise OASIS KMIP 2.0 §6.1.33 Table 247 and KMIP 2.1 §6.1.34 Table 265."""
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases

        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-PAYLOAD-CLIENT-MODIFY-ATTRIBUTE-REQUEST-21-003')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_modify_attribute_request_to_ttlv_payload')

        self.assertIsNotNone(result, 'installed Modify Attribute request payload route is missing')
        self.assertEqual(result['sample_count'], 2)
        self.assertTrue({'kmipkit_modify_attribute_request_create',
                         'kmipkit_modify_attribute_request_to_ttlv_payload'} <= result['invoked_symbols'])
        self.assertEqual(result['normalized_trees'], [
            [[0x420094, 7, b'fixture-object-1'.hex()],
             [0x42013C, 1, [[0x4200FD, 7, b'old'.hex()]]],
             [0x42013D, 1, [[0x4200FD, 7, b'new'.hex()]]]],
            [[0x42013D, 1, [[0x4200FD, 7, b'new'.hex()]]]],
        ])

    def test_structured_name_attribute_has_both_required_fields(self):
        executor = self.attribute_executor()
        target = next(target for target in executor.catalog['normative_targets']
                      if target['target_id'] == 'KMIPKIT-ELEM-ATTRIBUTE-NAME')
        trees = executor.fixtures(target, '2.1')
        self.assertEqual([child[0] for child in trees[0][2]], [0x420055, 0x420054])
        result = executor.execute(target, '2.1')
        self.assertTrue(result['invoked_symbols'])

    def test_vendor_attribute_uses_attribute_container_and_required_members(self):
        executor = self.attribute_executor()
        target = next(target for target in executor.catalog['normative_targets']
                      if target['target_id'] == 'KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE')
        trees = executor.fixtures(target, '2.1')
        self.assertIsNotNone(trees)
        self.assertEqual(len(trees), 1)
        self.assertEqual(trees[0][0], 0x420008)
        self.assertEqual([child[0] for child in trees[0][2]], [0x42009D, 0x42000A, 0x42000B])
        self.assertEqual([child[2] for child in trees[0][2]], [b'fixture_vendor', b'fixture_attribute', b'fixture-value'])
        result = executor.execute(target, '2.1')
        self.assertTrue({'kmipkit_typed_attribute_from_item', 'kmipkit_attribute_set_create',
                         'kmipkit_ttlv_decode'} <= result['invoked_symbols'])
        self.assertEqual(result['negative_checks'], 5)


if __name__ == '__main__':
    unittest.main()
