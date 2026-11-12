"""KMIPKIT-0092-FR-001/002/009: source-backed public API routes."""
from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path
import unittest
import sys
import subprocess

SHARED = Path(__file__).resolve().parents[1]
ROOT = SHARED.parents[1]


class RouteCapabilitiesTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        sys.path.insert(0, str(SHARED / 'tools'))
        spec = importlib.util.spec_from_file_location('route_capabilities', SHARED / 'tools/route_capabilities.py')
        cls.module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.module)

    def setUp(self):
        self.data = json.loads((SHARED / 'coverage/route-capabilities.json').read_text())

    def test_normative_inventory_requires_local_construct_and_decode_in_every_language(self):
        for language in ('rust', 'c', 'java', 'python'):
            routes = [r for r in self.data['routes'] if r['language'] == language
                      and 'normative_inventory' in r['case_categories'] and r['api_level'] == 'generic-ttlv']
            self.assertEqual({'construct', 'decode'}, {a for r in routes for a in r['actions']})
            self.assertTrue(all(r['mode'] == 'local' for r in routes))
            self.assertTrue(all(r['combinations'] == [{'encoding': 'TTLV', 'transport': None}]
                                for r in routes))

    def test_local_typed_model_routes_cover_objects_attributes_and_enum_wrappers(self):
        symbols = {r['public_api_symbol']: r for r in self.data['routes']}
        required = {
            'rust': ['kmipkit_protocol::Certificate::try_new', 'kmipkit_protocol::ObjectType::from_raw'],
            'c': ['kmipkit_managed_object_from_fields', 'kmipkit_typed_attribute_from_value'],
            'java': ['org.kmipkit.ttlv.ManagedObject.fromFields', 'org.kmipkit.ttlv.TypedAttribute.create'],
            'python': ['kmipkit.managed_objects.ManagedObject.from_fields', 'kmipkit.typed_attributes.TypedAttribute.create']}
        for language, names in required.items():
            for name in names:
                self.assertTrue(name in symbols, name)
                route = symbols[name]
                self.assertEqual(language, route['language'])
                self.assertEqual('local', route['mode'])
                self.assertEqual('typed-protocol', route['api_level'])
                self.assertTrue(route['normative_target_ids'])

    def test_typed_attribute_specializations_have_exact_normative_targets(self):
        routes = [r for r in self.data['routes'] if r['language'] == 'rust'
                  and 'Attribute::<' in r['public_api_symbol']]
        self.assertTrue(routes)
        self.assertTrue(any('KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE' in r['normative_target_ids'] for r in routes))

    def test_typed_attribute_marker_does_not_inherit_a_sibling_attribute_name(self):
        routes = [route for route in self.data['routes']
                  if route['language'] == 'rust' and 'typed::Comment>' in route['public_api_symbol']]
        self.assertTrue(routes)
        for route in routes:
            targets = set(route['normative_target_ids'])
            self.assertIn('KMIPKIT-ELEM-ATTRIBUTE-COMMENT', targets)
            self.assertIn('KMIPKIT-ELEM-TAG-4200FD', targets)
            self.assertNotIn('KMIPKIT-ELEM-ATTRIBUTE-DESCRIPTION', targets)
            self.assertNotIn('KMIPKIT-ELEM-TAG-4200FC', targets)

    def test_raw_response_route_names_only_the_java_fixture_backed_enum_target(self):
        key_format_raw = 'KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RAW-00000001'
        endpoint_role_client = 'KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-CLIENT-00000001'
        routes = {(route['language'], route['route_id']): route for route in self.data['routes']}
        java_route = routes['java', 'java.typed-protocol.function.client_batch_item_response_raw_ttlv']
        self.assertIn(key_format_raw, java_route['normative_target_ids'])
        self.assertNotIn(endpoint_role_client, java_route['normative_target_ids'])
        for language in ('c', 'python'):
            route = routes[language, f'{language}.typed-protocol.function.client_batch_item_response_raw_ttlv']
            self.assertNotIn(key_format_raw, route['normative_target_ids'])
            self.assertNotIn(endpoint_role_client, route['normative_target_ids'])

    def test_java_batch_response_accessors_are_limited_to_response_batch_items(self):
        expected = ['KMIPKIT-ELEM-MESSAGE-FIELD-8-4-BATCH-ITEM']
        for suffix in ('item_at', 'item_count'):
            route_id = f'java.typed-protocol.function.client_batch_response_{suffix}'
            route = next(route for route in self.data['routes'] if route['route_id'] == route_id)
            self.assertEqual(expected, route['normative_target_ids'])

    def test_attribute_container_routes_do_not_claim_managed_object_structures(self):
        # OASIS KMIP 2.0/2.1 §11.34 Object Type Enumeration is distinct from the §4.36 Object Type Attribute.
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        managed_object_type_values = {target_id for target_id, target in targets.items()
            if target['kind'] == 'object_type' or
            'KMIPKIT-ELEM-ENUMERATION-OBJECT-TYPE' in target.get('parent_element_ids', [])}
        routes = [route for route in self.data['routes']
                  if route['api_level'] == 'typed-protocol'
                  and any(token in route['public_api_symbol'].casefold()
                          for token in ('typedattribute', 'typed_attribute', 'attributeset', 'attribute_set'))]
        self.assertTrue(routes)
        for route in routes:
            route_targets = [targets[target_id] for target_id in route['normative_target_ids']
                             if target_id in targets]
            self.assertFalse(any(target['kind'] in {'object_structure', 'operation_structure'}
                                 for target in route_targets), route['public_api_symbol'])
            self.assertFalse(managed_object_type_values.intersection(route['normative_target_ids']),
                             route['public_api_symbol'])

    def test_object_item_constructors_do_not_claim_object_type_values_or_sibling_structures(self):
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        routes = {route['public_api_symbol']: route for route in self.data['routes']}
        certificate = routes['kmipkit_protocol::Certificate::try_new']
        certificate_targets = [targets[target_id] for target_id in certificate['normative_target_ids']]
        self.assertIn('KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE', certificate['normative_target_ids'])
        self.assertFalse(any(target['kind'] == 'object_type' or
                             target.get('parent_element_ids') == ['KMIPKIT-ELEM-ENUMERATION-OBJECT-TYPE']
                             for target in certificate_targets))

        unknown = routes['kmipkit_protocol::ManagedObject::unknown']
        unknown_targets = [targets[target_id] for target_id in unknown['normative_target_ids']]
        self.assertTrue(all(target['kind'] == 'tag_range' and target['target_id'] == 'KMIPKIT-RANGE-004'
                            for target in unknown_targets), unknown['normative_target_ids'])

    def test_managed_object_parser_route_only_claims_root_object_structures(self):
        # OASIS KMIP 2.0/2.1 §§2.1–2.9 define managed-object roots; §3 structures are embedded models.
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        route = next(route for route in self.data['routes']
                     if route['language'] == 'rust'
                     and route['public_api_symbol'] == 'kmipkit_protocol::ManagedObject::try_from_item')
        root_names = {
            'Certificate', 'Certificate Request', 'Opaque Object', 'PGP Key', 'Private Key',
            'Public Key', 'Secret Data', 'Split Key', 'Symmetric Key'}
        expected = {target['target_id'] for target in catalog['normative_targets']
                    if target['kind'] == 'object_structure' and target['name'] in root_names}
        actual = set(route['normative_target_ids'])
        self.assertEqual(expected, actual)
        self.assertTrue(all(targets[target_id]['kind'] == 'object_structure' for target_id in actual))

    def test_secret_data_parser_does_not_claim_inapplicable_key_block_crypto_fields(self):
        # OASIS KMIP 2.1 §3.1 Table 11 excludes Cryptographic Algorithm and Length from Secret Data.
        route = next(route for route in self.data['routes']
                     if route['language'] == 'rust'
                     and route['public_api_symbol'] == 'kmipkit_protocol::SecretData::try_from_item')
        self.assertIn('KMIPKIT-ELEM-OBJECT-STRUCTURE-SECRET-DATA', route['normative_target_ids'])
        self.assertNotIn('KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-CRYPTOGRAPHIC-ALGORITHM',
                         route['normative_target_ids'])
        self.assertNotIn('KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-CRYPTOGRAPHIC-LENGTH',
                         route['normative_target_ids'])

    def test_object_type_raw_value_route_only_claims_its_own_enum_family(self):
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        route = next(route for route in self.data['routes']
                     if route['language'] == 'rust'
                     and route['public_api_symbol'] == 'kmipkit_protocol::ObjectType::from_raw')
        expected = {target['target_id'] for target in catalog['normative_targets']
                    if target['kind'] == 'enumeration_value'
                    and 'KMIPKIT-ELEM-ENUMERATION-OBJECT-TYPE' in target.get('parent_element_ids', [])}
        self.assertEqual(expected, set(route['normative_target_ids']))
        self.assertTrue(all(target_id in targets for target_id in route['normative_target_ids']))

    def test_raw_value_routes_keep_nonstandard_normative_family_aliases(self):
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        routes = {route['public_api_symbol']: route for route in self.data['routes']}
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        revocation = routes['kmipkit_protocol::RevocationReason::from_raw']
        expected_revocation = {target['target_id'] for target in catalog['normative_targets']
                               if target['kind'] == 'enumeration_value'
                               and 'KMIPKIT-ELEM-ENUMERATION-REVOCATION-REASON-CODE'
                               in target.get('parent_element_ids', [])}
        self.assertEqual(expected_revocation, set(revocation['normative_target_ids']))

        protocol_version = routes['kmipkit_protocol::ProtocolVersion::from_raw']
        protocol_fields = {target_id for target_id, target in targets.items()
                           if target['kind'] == 'message_field'
                           and target['name'].startswith('Protocol Version')}
        self.assertTrue(protocol_fields <= set(protocol_version['normative_target_ids']))
        self.assertFalse(any(targets[target_id]['kind'] in {'object_structure', 'structure_member'}
                             for target_id in protocol_version['normative_target_ids']))

    def test_credential_value_serializers_only_claim_their_own_ttlv_shapes(self):
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        routes = {route['public_api_symbol']: route for route in self.data['routes']}
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        hashed = set(routes['kmipkit_protocol::HashedPasswordCredential::into_ttlv']['normative_target_ids'])
        self.assertIn('KMIPKIT-ELEM-CREDENTIAL-HASHED-PASSWORD', hashed)
        self.assertIn('KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-HASHED-PASSWORD', hashed)
        self.assertNotIn('KMIPKIT-ELEM-CREDENTIAL-CREDENTIAL', hashed)
        self.assertFalse(any(target_id.startswith('KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-')
                             for target_id in hashed))
        self.assertNotIn('KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-PASSWORD', hashed)
        self.assertNotIn('KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-PASSWORD-00000001', hashed)

        credential_value = set(routes['kmipkit_protocol::CredentialValue::into_ttlv']['normative_target_ids'])
        self.assertNotIn('KMIPKIT-ELEM-CREDENTIAL-CREDENTIAL', credential_value)
        variant_roots = {target['target_id'] for target in catalog['normative_targets']
                         if target['kind'] == 'credential'
                         and target['target_id'] != 'KMIPKIT-ELEM-CREDENTIAL-CREDENTIAL'}
        self.assertTrue(variant_roots <= credential_value)
        self.assertTrue(all(target_id in targets for target_id in credential_value))
        self.assertFalse(any(target_id.startswith('KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-')
                             for target_id in credential_value))

        credential = set(routes['kmipkit_protocol::Credential::into_ttlv']['normative_target_ids'])
        self.assertIn('KMIPKIT-ELEM-CREDENTIAL-CREDENTIAL', credential)
        credential_type_values = {target_id for target_id, target in targets.items()
                                  if target['kind'] == 'enumeration_value'
                                  and 'KMIPKIT-ELEM-ENUMERATION-CREDENTIAL-TYPE'
                                  in target.get('parent_element_ids', [])
                                  and target.get('allocation') == 'assigned'}
        self.assertTrue(credential_type_values <= credential)

    def test_typed_sensitive_attribute_does_not_claim_same_named_reason_enum(self):
        route = next(route for route in self.data['routes']
                     if route['language'] == 'rust'
                     and route['public_api_symbol'] ==
                     'kmipkit_protocol::attribute::Attribute::<kmipkit_protocol::attribute::typed::Sensitive>::from_item')
        self.assertEqual({'KMIPKIT-ELEM-ATTRIBUTE-SENSITIVE', 'KMIPKIT-ELEM-TAG-420120'},
                         set(route['normative_target_ids']))

    def test_java_typed_attribute_routes_do_not_claim_same_named_reason_enum(self):
        targets = {target['target_id'] for target in json.loads(
            (SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())['normative_targets']}
        sensitive_attribute = 'KMIPKIT-ELEM-ATTRIBUTE-SENSITIVE'
        unrelated_reason = 'KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-SENSITIVE-00000016'
        symbols = {
            'org.kmipkit.extensions.AttributeSet.tryNew',
            'org.kmipkit.ttlv.TypedAttribute.create',
            'org.kmipkit.ttlv.TypedAttribute.fromItem',
        }
        routes = {route['public_api_symbol']: route for route in self.data['routes']
                  if route['language'] == 'java'}
        self.assertTrue(unrelated_reason in targets)
        for symbol in symbols:
            route_targets = set(routes[symbol]['normative_target_ids'])
            self.assertIn(sensitive_attribute, route_targets, symbol)
            self.assertNotIn(unrelated_reason, route_targets, symbol)

    def test_cryptographic_parameters_encoder_does_not_claim_shredding_enum_values(self):
        unrelated_value = 'KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-CRYPTOGRAPHIC-00000002'
        routes = [route for route in self.data['routes']
                  if route['public_api_symbol'].endswith('CryptographicParameters::into_ttlv')]
        self.assertTrue(routes)
        for route in routes:
            self.assertNotIn(unrelated_value, route['normative_target_ids'], route['public_api_symbol'])

    def test_scalar_ttlv_converters_only_claim_targets_they_can_emit(self):
        routes = {route['public_api_symbol']: route for route in self.data['routes']}
        expected = {
            'kmipkit_protocol::SecretBytes::into_ttlv_value': {
                'KMIPKIT-ELEM-DATA-TYPE-BYTE-STRING',
            },
            'kmipkit_protocol::SecretText::into_ttlv_value': {
                'KMIPKIT-ELEM-DATA-TYPE-TEXT-STRING',
            },
        }
        for symbol, emitted_targets in expected.items():
            self.assertTrue(emitted_targets <= set(routes[symbol]['normative_target_ids']), symbol)
            self.assertEqual(emitted_targets, set(routes[symbol]['normative_target_ids']), symbol)

    def test_operation_route_matching_uses_complete_name_tokens(self):
        operations = {operation['operationId']: operation
                      for operation in self.module._api(ROOT, self.data['library_commit'])['operations']}
        hash_id = next(operation_id for operation_id, operation in operations.items()
                       if operation['name'] == 'Hash')
        recertify_id = next(operation_id for operation_id, operation in operations.items()
                            if operation['name'] == 'Re-certify')
        rekey_id = next(operation_id for operation_id, operation in operations.items()
                        if operation['name'] == 'Re-key')
        query_async_id = next(operation_id for operation_id, operation in operations.items()
                              if operation['name'] == 'Query Asynchronous Requests')
        self.assertIsNone(self.module._function_operation_id('hashed_password_credential_into_ttlv', operations))
        self.assertEqual(hash_id, self.module._function_operation_id('client_batch_item_hash', operations))
        self.assertEqual(recertify_id,
                         self.module._function_operation_id('client_batch_item_response_recertify', operations))
        self.assertEqual(rekey_id,
                         self.module._function_operation_id('client_batch_item_response_rekey', operations))
        self.assertEqual(query_async_id,
                         self.module._function_operation_id('client_execute_query_async_requests', operations))
        hashed_password = next(route for route in self.data['routes']
                               if route['language'] == 'rust'
                               and route['public_api_symbol'] ==
                               'kmipkit_protocol::HashedPasswordCredential::into_ttlv')
        self.assertEqual([], hashed_password['operation_ids'])
        self.assertIn('KMIPKIT-ELEM-CREDENTIAL-HASHED-PASSWORD', hashed_password['normative_target_ids'])
        self.assertNotIn('KMIPKIT-ELEM-OP-C2S-HASH', hashed_password['normative_target_ids'])

    def test_network_query_constructor_only_claims_request_payload_targets(self):
        # OASIS KMIP 2.1 §6.1.40 Table 282 defines Query inputs; Table 283 is its response.
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        operation_id = 'KMIPKIT-ELEM-OP-C2S-QUERY'
        route = next(route for route in self.data['routes']
                     if route['language'] == 'c'
                     and route['public_api_symbol'] == 'kmipkit_client_batch_item_query')
        expected = {operation_id} | {
            target_id for target_id, target in targets.items()
            if target.get('operation_id') == operation_id and target.get('role') == 'request'
        }
        self.assertEqual(expected, set(route['normative_target_ids']))

    def test_register_and_revoke_batch_item_constructors_are_network_routes(self):
        expected = {
            ('rust', 'kmipkit_client::ClientBatchItem::register'),
            ('c', 'kmipkit_client_batch_item_register'),
            ('java', 'org.kmipkit.extensions.ClientBatchItem.register'),
            ('python', 'kmipkit.extensions.client_batch_item_register'),
            ('rust', 'kmipkit_client::ClientBatchItem::revoke'),
            ('c', 'kmipkit_client_batch_item_revoke'),
            ('java', 'org.kmipkit.extensions.ClientBatchItem.revoke'),
            ('python', 'kmipkit.extensions.client_batch_item_revoke'),
        }
        routes = {(route['language'], route['public_api_symbol']): route for route in self.data['routes']}
        self.assertTrue(expected <= set(routes), sorted(expected - set(routes)))
        operation_ids = {
            ('rust', 'kmipkit_client::ClientBatchItem::register'): 'KMIPKIT-ELEM-OP-C2S-REGISTER',
            ('c', 'kmipkit_client_batch_item_register'): 'KMIPKIT-ELEM-OP-C2S-REGISTER',
            ('java', 'org.kmipkit.extensions.ClientBatchItem.register'): 'KMIPKIT-ELEM-OP-C2S-REGISTER',
            ('python', 'kmipkit.extensions.client_batch_item_register'): 'KMIPKIT-ELEM-OP-C2S-REGISTER',
            ('rust', 'kmipkit_client::ClientBatchItem::revoke'): 'KMIPKIT-ELEM-OP-C2S-REVOKE',
            ('c', 'kmipkit_client_batch_item_revoke'): 'KMIPKIT-ELEM-OP-C2S-REVOKE',
            ('java', 'org.kmipkit.extensions.ClientBatchItem.revoke'): 'KMIPKIT-ELEM-OP-C2S-REVOKE',
            ('python', 'kmipkit.extensions.client_batch_item_revoke'): 'KMIPKIT-ELEM-OP-C2S-REVOKE',
        }
        for key, operation_id in operation_ids.items():
            route = routes[key]
            self.assertEqual('network', route['mode'], key)
            self.assertEqual([operation_id], route['operation_ids'], key)
            self.assertEqual(6, len(route['combinations']), key)
            self.assertEqual('construct', route['actions'][0], key)

    def test_batch_response_readers_only_claim_response_batch_fields(self):
        # OASIS KMIP 2.0/2.1 §§8.5-8.6 and §§9.5, 9.7 define response Batch Count and Batch Item.
        expected = {
            'count': {
                'KMIPKIT-ELEM-MESSAGE-FIELD-8-5-BATCH-COUNT',
                'KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT',
            },
            'at': {
                'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM',
                'KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM',
            },
        }
        symbols = {
            'rust': {
                'count': 'kmipkit_client::ClientBatchResponse::len',
                'at': 'kmipkit_client::ClientBatchResponse::get',
            },
            'c': {
                'count': 'kmipkit_client_batch_response_item_count',
                'at': 'kmipkit_client_batch_response_item_at',
            },
            'java': {
                'count': 'org.kmipkit.extensions.ClientBatchResponse.itemCount',
                'at': 'org.kmipkit.extensions.ClientBatchResponse.itemAt',
            },
            'python': {
                'count': 'kmipkit.client.BatchResult.item_count',
                'at': 'kmipkit.client.BatchResult.item_at',
            },
        }
        routes = {(route['language'], route['public_api_symbol']): route for route in self.data['routes']}
        for language, readers in symbols.items():
            for reader, target_ids in expected.items():
                route = routes[(language, readers[reader])]
                self.assertEqual(sorted(target_ids), sorted(route['normative_target_ids']), readers[reader])
                self.assertEqual(['decode'], route['actions'], readers[reader])

    def test_batch_error_continuation_builder_is_a_local_route_in_every_language(self):
        # OASIS KMIP 2.0/2.1 §§8.2, 9.6, and 11.5 define this request option and its values.
        symbols = {
            'rust': 'kmipkit_client::ClientBatch::with_batch_error_continuation_option',
            'c': 'kmipkit_client_batch_add_error_continuation_value',
            'java': 'org.kmipkit.extensions.ClientBatch.addErrorContinuationValue',
            'python': 'kmipkit.client.Batch.add_error_continuation_value',
        }
        expected = {
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ERROR-CONTINUATION-OPTION',
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-6-BATCH-ERROR-CONTINUATION-OPTION',
            'KMIPKIT-ELEM-TAG-42000E',
            'KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-CONTINUE-00000001',
            'KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-STOP-00000002',
            'KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-UNDO-00000003',
            'KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-EXTENSIONS-8XXXXXXX',
        }
        routes = {(route['language'], route['public_api_symbol']): route for route in self.data['routes']}
        for language, symbol in symbols.items():
            route = routes[(language, symbol)]
            self.assertEqual('local', route['mode'], symbol)
            self.assertEqual(expected, set(route['normative_target_ids']), symbol)
            self.assertIn('construct', route['actions'], symbol)

    def test_response_message_extension_readers_only_claim_the_extension_field(self):
        # OASIS KMIP 2.0/2.1 §§8.6, 9.13, and 11.56 define response Message Extension.
        expected = {
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-MESSAGE-EXTENSION',
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION',
            'KMIPKIT-ELEM-TAG-420051',
        }
        symbols = {
            'rust': {
                'count': 'kmipkit_client::ClientBatchItemResponse::extensions',
                'at': 'kmipkit_client::ClientMessageExtension::with_ttlv',
            },
            'c': {
                'count': 'kmipkit_client_batch_item_response_extension_count',
                'at': 'kmipkit_client_batch_item_response_extension_ttlv_at',
            },
            'java': {
                'count': 'org.kmipkit.extensions.ClientBatchItemResult.extensionCount',
                'at': 'org.kmipkit.extensions.ClientBatchItemResult.extensionTtlvAt',
            },
            'python': {
                'count': 'kmipkit.client.BatchItemResult.extension_count',
                'at': 'kmipkit.client.BatchItemResult.extension_ttlv_at',
            },
        }
        routes = {(route['language'], route['public_api_symbol']): route for route in self.data['routes']}
        for language, readers in symbols.items():
            for reader, symbol in readers.items():
                route = routes[(language, symbol)]
                self.assertEqual(expected, set(route['normative_target_ids']), symbol)
                self.assertEqual(['decode'], route['actions'], symbol)

    def test_raw_response_ttlv_routes_claim_the_observed_get_response_value(self):
        # OASIS KMIP 2.0/2.1 §§3.1, 9.7, and 11.25 define this Get response tree and Raw value.
        expected = {
            'KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM',
            'KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM',
            'KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-FORMAT-TYPE',
            'KMIPKIT-ELEM-TAG-42000F',
            'KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RAW-00000001',
        }
        symbols = {
            'c': 'kmipkit_client_batch_item_response_raw_ttlv',
            'java': 'org.kmipkit.extensions.ClientBatchItemResult.raw',
            'python': 'kmipkit.client.BatchItemResult.raw',
        }
        routes = {(route['language'], route['public_api_symbol']): route for route in self.data['routes']}
        for language, symbol in symbols.items():
            route = routes[(language, symbol)]
            self.assertEqual(expected, set(route['normative_target_ids']), symbol)
            self.assertEqual(['decode'], route['actions'], symbol)

    def test_checked_tag_constructor_exclusions_are_source_backed_and_narrow(self):
        expected = {
            'rust': {'rust.generic-ttlv.item.new', 'rust.generic-ttlv.model.kmipkit.ttlv.structure.try.push'},
            'c': {'c.generic-ttlv.kmipkit_ttlv_item_create', 'c.generic-ttlv.model.kmipkit.ttlv.structure.with.item'},
            'java': {'java.generic-ttlv.ttlv_item.create', 'java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.withitem'},
            'python': {'python.generic-ttlv.ttlv.ttlv_item_create', 'python.generic-ttlv.model.kmipkit.ttlv.ttlv.structure.with.item'},
        }
        for language, route_ids in expected.items():
            routes = {route['route_id']: route for route in self.data['routes']
                      if route['language'] == language}
            for route_id in route_ids:
                route = routes[route_id]
                self.assertEqual([{
                    'target_kinds': ['tag', 'tag_range'],
                    'allocations': ['reserved', 'unused'],
                    'reason': 'This constructor takes an allocation-checked Tag; RawTag::try_checked rejects reserved and unused allocations before an Item can be constructed.',
                    'source_paths': [route['source_paths'][0], 'crates/kmipkit-ttlv/src/tag.rs'],
                }], route['case_exclusions'])

    def test_reserved_tag_item_view_exclusions_require_decoder_rejection_sources(self):
        expected = {
            'rust': ('rust.generic-ttlv.function.ttlv_item_view_from_item',
                     'crates/kmipkit-ttlv/src/item.rs'),
            'java': ('java.generic-ttlv.function.ttlv_item_view_from_item',
                     'bindings/java/src/main/java/org/kmipkit/ttlv/TtlvItem.java'),
            'python': ('python.generic-ttlv.function.ttlv_item_view_from_item',
                        'bindings/python/src/kmipkit/ttlv.py'),
        }
        for language, (route_id, route_source) in expected.items():
            route = next(route for route in self.data['routes'] if route['route_id'] == route_id)
            self.assertEqual([{
                'target_kinds': ['tag', 'tag_range'],
                'allocations': ['reserved', 'unused'],
                'reason': 'The TTLV decoder rejects reserved and unused allocations before it can return an Item; this accessor therefore has no valid Item input for the case.',
                'source_paths': [route_source, 'crates/kmipkit-ttlv/src/codec/decoder.rs',
                                 'crates/kmipkit-ttlv/src/tag.rs'],
            }], route['case_exclusions'])

    def test_async_request_routes_use_the_exact_public_operation_executor(self):
        expected = {
            'rust': 'kmipkit_client::Client::execute_{operation}',
            'c': 'kmipkit_client_execute_{operation}',
            'java': 'org.kmipkit.extensions.Client.execute{operation_title}',
            'python': 'kmipkit.client.Client.execute_{operation}',
        }
        operations = {
            'cancel': 'cancel',
            'poll': 'poll',
            'process': 'process',
            'queryasyncrequests': 'query_async_requests',
        }
        routes = {(route['language'], route['execution_route_id']): route
                  for route in self.data['execution_routes']}
        for language, template in expected.items():
            for route_id, method_name in operations.items():
                title = ''.join(part.title() for part in method_name.split('_'))
                symbol = template.format(operation=method_name, operation_title=title)
                route = routes[(language, f'{language}.execute.{route_id}')]
                self.assertEqual(symbol, route['public_api_symbol'])

    def test_every_named_enum_and_mask_target_has_an_exact_public_value_symbol(self):
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        target_ids = {t['target_id'] for t in catalog['normative_targets']
                      if t['kind'] in {'enumeration_value', 'bitmask_value'} and t.get('allocation') == 'assigned'}
        for language in ('rust', 'c', 'java', 'python'):
            values = {v['target_id'] for v in self.data.get('value_symbols', []) if v['language'] == language and v['availability'] == 'named-constant'}
            self.assertEqual(target_ids, values)

    def test_installed_named_values_cannot_be_missing_or_have_the_wrong_wire_value(self):
        self.assertTrue(hasattr(self.module, 'validate_installed_values'), 'installed named-value validation is missing')
        with self.assertRaisesRegex(ValueError, 'named value'):
            self.module.validate_installed_values(self.data, 'python', {})
        values = {row['public_api_symbol']: row['wire_value'] for row in self.data['value_symbols']
                  if row['language'] == 'python' and row['availability'] == 'named-constant'}
        first = next(iter(values))
        values[first] ^= 1
        with self.assertRaisesRegex(ValueError, 'named value'):
            self.module.validate_installed_values(self.data, 'python', values)

    def test_generic_value_constructors_have_distinct_data_type_routes(self):
        symbols = {r['public_api_symbol'] for r in self.data['routes']}
        for symbol in ('kmipkit_ttlv::Value::enumeration', 'kmipkit_ttlv_value_enumeration',
                       'org.kmipkit.ttlv.TtlvValue.Enumeration', 'kmipkit.ttlv.ttlv_value_enumeration'):
            self.assertTrue(symbol in symbols, symbol)

    def test_public_request_serializers_and_response_readers_are_routes(self):
        api = json.loads((ROOT / 'specification/api/public-api.json').read_text())
        routes = {(route['language'], route['public_api_symbol']): route for route in self.data['routes']}
        package_languages = {package['id']: package['language'] for package in api['publicPackages']}
        declared = {(package_languages[declaration['packageId']], declaration['exportName'])
                    for declaration in api['publicDeclarations']}
        for function in api['functions']:
            function_id = function['id'].lower()
            if not any(token in function_id for token in (
                    'to_ttlv', 'into_ttlv', 'from_ttlv', 'from_item',
                    'to_batch_item', 'response_')):
                continue
            for language in ('rust', 'c', 'java', 'python'):
                symbol = self.module.function_entrypoint(function, language)
                if symbol:
                    if (language, symbol) not in declared:
                        continue
                    with self.subTest(function=function['id'], language=language, symbol=symbol):
                        self.assertTrue((language, symbol) in routes,
                                        f"{function['id']} is missing {language} route {symbol}")
                        expected_action = ('construct' if '_to_batch_item' in function_id else
                                           'decode' if any(token in function_id for token in ('from_ttlv', 'from_item', 'response_')) else
                                           'encode')
                        if (language, symbol) in routes:
                            self.assertIn(expected_action, routes[(language, symbol)]['actions'],
                                          f"{function['id']} must be a {expected_action} route")

    def test_known_but_false_api_level_cannot_relabel_a_constructor(self):
        self.data['routes'][0]['api_level'] = 'high-level'
        self.reject(self.data, 'api_level')

    def test_case_category_and_protocol_coverage_cannot_be_silently_removed(self):
        self.data['routes'][0]['case_categories'] = []
        self.reject(self.data, 'case_categories')
        self.data = json.loads((SHARED / 'coverage/route-capabilities.json').read_text())
        self.data['routes'][0]['protocol_versions'] = ['2.1']
        self.reject(self.data, 'protocol_versions')

    def test_transparent_key_constructor_is_bound_to_its_owned_key_material_table(self):
        route = next(r for r in self.data['routes'] if r['public_api_symbol'] ==
                     'kmipkit_protocol::TransparentRsaPrivateKey::try_from_components')
        ids = route['normative_target_ids']
        self.assertTrue(any(target.startswith('KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-') for target in ids))
        self.assertFalse(any(target.startswith('KMIPKIT-ELEM-STRUCTURE-MEMBER-3-8-KEY-MATERIAL-') for target in ids))

    def test_referenced_pkcs11_values_remain_covered_when_named_constants_are_absent(self):
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        expected = {target['target_id'] for target in catalog['normative_targets']
                    if target['kind'] == 'referenced_enum_value'}
        for language in ('rust', 'c', 'java', 'python'):
            rows = [row for row in self.data['value_symbols'] if row['language'] == language
                    and row['target_id'] in expected]
            self.assertEqual(expected, {row['target_id'] for row in rows})
            constructors = {route['public_api_symbol'] for route in self.data['routes'] if route['language'] == language}
            for row in rows:
                if row['availability'] == 'numeric-only':
                    self.assertIsNone(row['public_api_symbol'])
                    self.assertIn(row['numeric_constructor'], constructors)

    def test_check_cli_is_read_only_and_accepts_current_generated_metadata(self):
        manifest = SHARED / 'coverage/route-capabilities.json'
        before = manifest.read_bytes()
        result = subprocess.run([sys.executable, str(SHARED / 'tools/route_capabilities.py'), '--check'],
                                capture_output=True, text=True, check=False)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(before, manifest.read_bytes())

    def test_checked_in_manifest_validates_against_public_api(self):
        self.module.validate(self.data, ROOT)
        self.assertEqual({'rust', 'c', 'java', 'python'}, {r['language'] for r in self.data['routes']})

    def reject(self, data, message):
        with self.assertRaisesRegex(ValueError, message):
            self.module.validate(data, ROOT)

    def test_unknown_language_is_rejected(self):
        self.data['routes'][0]['language'] = 'ruby'
        self.reject(self.data, 'language')

    def test_unknown_route_family_is_rejected(self):
        self.data['routes'][0]['api_level'] = 'imaginary'
        self.reject(self.data, 'api_level')

    def test_duplicate_ids_are_rejected(self):
        self.data['routes'].append(copy.deepcopy(self.data['routes'][0]))
        self.reject(self.data, 'duplicate route_id')

    def test_missing_public_symbol_is_rejected(self):
        self.data['routes'][0]['public_api_symbol'] = 'not_a_public_symbol'
        self.reject(self.data, 'public API symbol')

    def test_impossible_encoding_transport_is_rejected(self):
        self.data['routes'][0]['combinations'].append({'encoding': 'DER', 'transport': 'udp'})
        self.reject(self.data, 'encoding|transport')

    def test_local_route_cannot_claim_network_transport(self):
        route = next(r for r in self.data['routes'] if r['mode'] == 'local')
        route['combinations'][0]['transport'] = 'raw_tls'
        self.reject(self.data, 'local')

    def test_public_constructor_cannot_disappear(self):
        route = next(r for r in self.data['routes'] if r['mode'] == 'network')
        self.data['routes'].remove(route)
        self.reject(self.data, 'missing public API route')

    def test_false_unavailable_route_is_rejected(self):
        route = next(r for r in self.data['routes'] if r['mode'] == 'network')
        self.data['unavailable'].append({'language': route['language'], 'capability': 'constructor',
            'public_api_symbol': route['public_api_symbol'], 'reason': 'validator recipe pending',
            'source_paths': ['AGENTS.md']})
        self.reject(self.data, 'public constructor.*unavailable')

    def test_same_constructor_cannot_be_counted_twice(self):
        route = copy.deepcopy(self.data['routes'][0])
        route['route_id'] += '.pretend-independent'
        self.data['routes'].append(route)
        self.reject(self.data, 'same constructor')

    def test_unknown_route_id_is_rejected(self):
        self.data['routes'][0]['route_id'] += '.invented'
        self.reject(self.data, 'unknown route_id')

    def test_constructor_cannot_claim_another_operation(self):
        route = next(r for r in self.data['routes'] if r['mode'] == 'network')
        route['operation_ids'] = ['KMIPKIT-ELEM-OP-C2S-NONEXISTENT']
        self.reject(self.data, 'operation mapping')

    def test_missing_supported_pair_is_rejected(self):
        route = next(r for r in self.data['routes'] if r['mode'] == 'network')
        route['combinations'].pop()
        self.reject(self.data, 'supported encoding/transport')

    def test_missing_execution_route_is_rejected(self):
        self.data['execution_routes'].pop()
        self.reject(self.data, 'execution route')

    def test_regeneration_matches_checked_in_capabilities(self):
        self.assertEqual(self.data, self.module.build_manifest(ROOT, self.data['library_commit']))

    def test_artifact_probe_requires_the_public_sender(self):
        symbols = {r['public_api_symbol'] for r in self.data['routes'] if r['language'] == 'python'}
        with self.assertRaisesRegex(ValueError, 'installed artifact.*missing'):
            self.module.validate_installed_symbols(self.data, 'python', symbols)

    def test_artifact_probe_rejects_wrong_commit(self):
        with self.assertRaisesRegex(ValueError, 'artifact commit'):
            self.module.validate_artifact_identity(self.data, {'library_commit': 'a' * 40})

    def test_artifact_probe_requires_every_declared_symbol(self):
        with self.assertRaisesRegex(ValueError, 'installed artifact.*missing'):
            self.module.validate_installed_symbols(self.data, 'python', set())


class ModelApplicabilityTests(unittest.TestCase):
    def test_operation_scope_also_filters_descendant_payload_type_variants(self):
        sys.path.insert(0, str(SHARED / 'tools'))
        from route_models import target_ids
        targets = []
        for operation in ('activate', 'destroy'):
            field_id = operation + '-identifier'
            targets.append({'target_id': field_id, 'kind': 'operation_payload_field',
                'name': 'Unique Identifier', 'operation_id': operation, 'role': 'request'})
            targets.append({'target_id': field_id + '-text', 'kind': 'operation_payload_type_variant',
                'name': 'Unique Identifier as Text String', 'operation_id': operation,
                'role': 'request', 'parent_element_ids': [field_id]})
        self.assertEqual(['activate-identifier', 'activate-identifier-text'],
            target_ids(targets, ['Unique Identifier'], operation_id='activate'))
        self.assertEqual([], target_ids(targets, ['Unique Identifier']))


class ManifestSerializationTests(unittest.TestCase):
    def test_generated_records_are_complete_json_before_atomic_replacement(self):
        sys.path.insert(0, str(SHARED / 'tools'))
        import route_capabilities
        data = {'schema_version': '1.0', 'routes': [{'route_id': 'a'}], 'value_symbols': []}
        serialized = route_capabilities.serialize_manifest(data)
        self.assertTrue(serialized.endswith('}\n'), 'serialized manifest must close its root object')
        self.assertEqual(data, json.loads(serialized))


if __name__ == '__main__':
    unittest.main()
