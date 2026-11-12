"""KMIPKIT-0092: executable route evidence must fail closed on missing recipes."""
import copy
import importlib
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells
from validate_route_evidence import validate_report


class RouteMatrixTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(
            importlib.util.find_spec('kmipkit_validator.route_matrix'),
            'the installed-package consumer needs an executable route-matrix runner',
        )
        self.runner = importlib.import_module('kmipkit_validator.route_matrix')
        self.catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        self.manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        self.artifact = {'library_commit': self.manifest['library_commit'], 'artifact_sha256': 'a' * 64}

    def integer_fixture(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [next(case for case in catalog['test_cases']
                                     if case['requires'].get('target_kind') == 'data_type'
                                     and case['requires'].get('normative_target_id', '').endswith('INTEGER'))]
        manifest = copy.deepcopy(self.manifest)
        wanted = {'kmipkit.ttlv.ttlv_value_integer', 'kmipkit.ttlv.ttlv_item_create',
                  'kmipkit.ttlv.ttlv_decode', 'kmipkit.ttlv.ttlv_item_view_from_item'}
        manifest['routes'] = [route for route in manifest['routes']
                              if route['language'] == 'python' and route['public_api_symbol'] in wanted]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping'))
        return catalog, manifest

    def test_generic_routes_call_named_apis_and_compare_constructed_and_decoded_values(self):
        catalog, manifest = self.integer_fixture()
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertEqual(len(list(iter_cells(catalog, manifest, languages=('python',)))), len(report['events']))
        summary = validate_report(report, catalog, manifest,
                                  catalog_sha256=report['catalog_sha256'],
                                  route_manifest_sha256=report['route_manifest_sha256'],
                                  expected_artifact_sha256=self.artifact['artifact_sha256'])
        self.assertTrue(summary['all_invocations_passed'])
        self.assertTrue(all(event['invocation_count'] == 1 for event in report['events']))
        jsonschema = importlib.import_module('jsonschema')
        schema = json.loads((ROOT / 'interop/shared/schemas/route-execution-evidence.schema.json').read_text())
        jsonschema.validate(report, schema)

    def test_matrix_preloads_python_package_before_parallel_invocations(self):
        from unittest.mock import patch

        catalog, manifest = self.integer_fixture()
        imported = []
        import_module = self.runner.importlib.import_module
        executor_type = self.runner.ThreadPoolExecutor

        def track_import(name, *args, **kwargs):
            imported.append(name)
            return import_module(name, *args, **kwargs)

        def checked_executor(*args, **kwargs):
            self.assertIn('kmipkit', imported,
                          'the package import must finish before route worker threads start')
            return executor_type(*args, **kwargs)

        with patch.object(self.runner.importlib, 'import_module', side_effect=track_import), \
                patch.object(self.runner, 'ThreadPoolExecutor', side_effect=checked_executor):
            report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        self.assertFalse(gaps)
        self.assertTrue(report['events'])

    def test_missing_recipe_is_pending_coverage_and_never_becomes_api_unavailability(self):
        catalog, manifest = self.integer_fixture()
        route = copy.deepcopy(manifest['routes'][0])
        route['route_id'] = 'python.test.unimplemented.public.route'
        route['public_api_symbol'] = 'kmipkit.ttlv.codec_limits_defaults'
        manifest['routes'] = [route, manifest['routes'][-1]]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], report['events'])
        self.assertTrue(gaps)
        self.assertEqual({'recipe_pending'}, {gap['failure_code'] for gap in gaps})
        with self.assertRaisesRegex(ValueError, 'missing route execution cells'):
            validate_report(report, catalog, manifest, catalog_sha256=report['catalog_sha256'],
                            route_manifest_sha256=report['route_manifest_sha256'],
                            expected_artifact_sha256=self.artifact['artifact_sha256'])

    def test_duplicate_or_forged_positive_cells_are_rejected(self):
        catalog, manifest = self.integer_fixture()
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertFalse(gaps)
        for mutation, message in (
            (lambda value: value['events'].append(copy.deepcopy(value['events'][0])), 'duplicate'),
            (lambda value: value['events'][0].update(public_api_symbol='kmipkit.fake'), 'public API symbol'),
            (lambda value: value['events'][0].update(invocation_count=0), 'not actually invoked'),
        ):
            altered = copy.deepcopy(report)
            mutation(altered)
            with self.assertRaisesRegex(ValueError, message):
                validate_report(altered, catalog, manifest, catalog_sha256=report['catalog_sha256'],
                                route_manifest_sha256=report['route_manifest_sha256'],
                                expected_artifact_sha256=self.artifact['artifact_sha256'])

    def test_failure_after_an_observed_api_call_is_distinct_from_missing_coverage(self):
        from unittest.mock import patch
        catalog, manifest = self.integer_fixture()
        with patch.object(self.runner, 'item_tree', side_effect=ValueError('secret-value-sentinel')):
            report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertTrue(report['events'], 'an invoked public API failure must remain an execution event')
        self.assertTrue(gaps, 'unreached public APIs must remain pending coverage')
        self.assertEqual({'failed'}, {event['status'] for event in report['events']})
        self.assertEqual({'route_execution_failed'}, {event['failure_code'] for event in report['events']})
        self.assertNotIn('secret-value-sentinel', json.dumps([report, gaps]))

    def test_installed_distribution_records_the_exact_verified_wheel_digest(self):
        from kmipkit_validator.contract import validate_artifact
        artifact = validate_artifact()
        artifact['artifact_sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'installed wheel identity'):
            self.runner.verify_installed_artifact(artifact)

    def test_exception_text_and_values_never_escape_into_report_or_pending_coverage(self):
        catalog, manifest = self.integer_fixture()
        from unittest.mock import patch
        with patch.object(self.runner, 'execute_cell', side_effect=RuntimeError('private-key-sentinel')):
            report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertNotIn('private-key-sentinel', json.dumps([report, gaps]))
        self.assertEqual({'recipe_execution_failed'}, {gap['failure_code'] for gap in gaps})

    def test_generic_enum_recipe_uses_its_own_numeric_tag_and_assigned_value(self):
        catalog, manifest = self.integer_fixture()
        catalog['test_cases'] = [next(case for case in self.catalog['test_cases']
                                     if case['requires'].get('normative_target_id') ==
                                     'KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-AES-00000003')]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertFalse(gaps)
        expected = self.runner.digest([[0x420028, 5, 3]])
        self.assertTrue(report['events'])
        self.assertEqual({expected}, {event['request_tree_sha256'] for event in report['events']})

    def test_generic_enum_routes_read_each_value_from_its_normative_target(self):
        catalog, manifest = self.integer_fixture()
        catalog['test_cases'] = [next(case for case in self.catalog['test_cases']
                                     if case['case_id'] == 'test-enum-adjustment-type')]
        symbols = {'kmipkit.ttlv.ttlv_item_create', 'kmipkit.ttlv.ttlv_value_enumeration'}
        manifest['routes'] = [route for route in manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] in symbols]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['language'] == 'python'
                                       and route['public_api_symbol'] == 'kmipkit.ttlv.ttlv_value_enumeration'))

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        self.assertFalse(gaps)
        self.assertEqual(16, len(report['events']))
        self.assertEqual(symbols, {event['public_api_symbol'] for event in report['events']})
        adjustment_targets = [target for target in self.catalog['normative_targets']
                              if target['target_id'].startswith(
                                  'KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-')]
        adjustment_tag = next(target for target in self.catalog['normative_targets']
                              if target['kind'] == 'tag' and target['name'] == 'Adjustment Type')
        expected_target_ids = {target['target_id'] for target in adjustment_targets}
        expected_trees = {self.runner.digest([[int(adjustment_tag['wire_value'], 16), 5,
                                                int(target['wire_value'].replace('X', '0'), 16)]])
                          for target in adjustment_targets}
        self.assertEqual(expected_target_ids, {event['target_id'] for event in report['events']})
        self.assertEqual(expected_trees, {event['request_tree_sha256'] for event in report['events']})

    def test_cryptographic_parameter_enum_setters_preserve_exact_values_and_types(self):
        names = ('BLOCK-CIPHER-MODE', 'PADDING-METHOD', 'HASHING-ALGORITHM', 'KEY-ROLE-TYPE',
                 'DIGITAL-SIGNATURE-ALGORITHM', 'CRYPTOGRAPHIC-ALGORITHM', 'MASK-GENERATOR')
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in catalog['test_cases']
                                 if case['requires'].get('target_kind') == 'enumeration_value'
                                 and any(case['requires'].get('normative_target_id', '').startswith(
                                     'KMIPKIT-ELEM-ENUM-VALUE-' + name + '-') for name in names)]
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes'] if route['language'] == 'python'
                              and route['public_api_symbol'].startswith('kmipkit.cryptographic_parameters.')]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping'))
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual(0, len(gaps), f'{len(gaps)} parameter enum cells lack recipes')
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        self.assertGreater(len(report['events']), 200)

    def test_scalar_payload_variants_roundtrip_the_exact_field_type_and_tag(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in catalog['test_cases']
                                 if case['requires'].get('target_kind') == 'operation_payload_value_variant'
                                 and case['requires'].get('value_type') != 'Structure']
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes'] if route['language'] == 'python'
                              and route['public_api_symbol'].startswith('kmipkit.ttlv.')]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping'))
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        field_gaps = [gap for gap in gaps if targets[gap['target_id']]['kind'] == 'operation_payload_value_variant']
        self.assertEqual(0, len(field_gaps), f'{len(field_gaps)} exact scalar-field cells lack recipes')
        self.assertGreater(len(report['events']), 100)
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_typed_attribute_and_set_routes_preserve_the_named_enum_and_unknown_values(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in catalog['test_cases'] if case['requires'].get('normative_target_id', '')
                                 .startswith('KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-')]
        symbols = {'kmipkit.typed_attributes.TypedAttribute.create',
                   'kmipkit.typed_attributes.TypedAttribute.from_item',
                   'kmipkit.attribute_operations.AttributeSet.try_new',
                   'kmipkit.attribute_operations.AttributeSet.try_push',
                   'kmipkit.attribute_operations.AttributeSet.try_push_typed'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes'] if route['public_api_symbol'] in symbols]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping'))
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual(0, len(gaps), f'{len(gaps)} typed attribute cells lack a recipe')
        self.assertTrue(report['events'])
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        self.assertEqual(symbols, {event['public_api_symbol'] for event in report['events']})

    def test_typed_attribute_routes_construct_a_valid_alternative_name_structure(self):
        # OASIS KMIP 2.0/2.1 §4.2, Table 31 requires Alternative Name Value and Type.
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-attr-alternative-name']
        symbols = {'kmipkit.typed_attributes.TypedAttribute.create',
                   'kmipkit.typed_attributes.TypedAttribute.from_item',
                   'kmipkit.attribute_operations.AttributeSet.try_new',
                   'kmipkit.attribute_operations.AttributeSet.try_push',
                   'kmipkit.attribute_operations.AttributeSet.try_push_typed'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] in symbols]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['language'] == 'python'
                                       and route['public_api_symbol'] ==
                                       'kmipkit.extensions.client_batch_item_get_attributes'))

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact, process_workers=2)

        typed_events = [event for event in report['events'] if event['public_api_symbol'] in symbols]
        typed_gaps = [gap for gap in gaps if gap['public_api_symbol'] in symbols]
        self.assertFalse(typed_gaps)
        self.assertEqual(10, len(typed_events))
        self.assertEqual(symbols, {event['public_api_symbol'] for event in typed_events})
        def tag_value(name):
            target = next(target for target in self.catalog['normative_targets']
                          if target['kind'] == 'tag' and target['name'] == name)
            return int(target['wire_value'], 16)

        alternative_type = next(target for target in self.catalog['normative_targets']
                                if target['kind'] == 'enumeration_value'
                                and target['name'] == 'Uninterpreted Text String'
                                and 'KMIPKIT-ELEM-ENUMERATION-ALTERNATIVE-NAME-TYPE'
                                in target['parent_element_ids'])
        expected = self.runner.digest([[
            tag_value('Alternative Name'), 1,
            [[tag_value('Alternative Name Value'), 7, 'KMIPKit.Interop'.encode().hex()],
             [tag_value('Alternative Name Type'), 5, int(alternative_type['wire_value'], 16)]],
        ]])
        self.assertEqual({expected}, {event['request_tree_sha256'] for event in typed_events})

    def test_typed_attribute_routes_construct_catalogued_attribute_structures(self):
        # OASIS KMIP 2.0/2.1 §§4.4, 4.14, 4.16, 4.21, 4.27, 4.31, 4.32, 4.46,
        # 4.47, 4.52/4.59, 4.53/4.60–4.63. The catalog binds each case to its
        # versioned table and normative reference.
        structure_cases = {
            'test-attr-application-specific-information',
            'test-attr-cryptographic-domain-parameters',
            'test-attr-cryptographic-parameters', 'test-attr-digest',
            'test-attr-key-value-location', 'test-attr-link', 'test-attr-name',
            'test-attr-random-number-generator', 'test-attr-revocation-reason',
            'test-attr-rotate-name', 'test-attr-usage-limits',
            'test-attr-vendor-attribute', 'test-attr-x.509-certificate-identifier',
            'test-attr-x.509-certificate-issuer', 'test-attr-x.509-certificate-subject',
        }
        catalog = copy.deepcopy(self.catalog)
        target_kinds = {target['target_id']: target['kind']
                        for target in self.catalog['normative_targets']}
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] in structure_cases
                                 and target_kinds.get(case['requires'].get('normative_target_id')) == 'attribute']
        for case in catalog['test_cases']:
            case.pop('operation', None)
            for field in ('operation_id', 'operation_sequence', 'protocol_scenario'):
                case['requires'].pop(field, None)
        symbols = {'kmipkit.typed_attributes.TypedAttribute.create',
                   'kmipkit.typed_attributes.TypedAttribute.from_item',
                   'kmipkit.attribute_operations.AttributeSet.try_new',
                   'kmipkit.attribute_operations.AttributeSet.try_push',
                   'kmipkit.attribute_operations.AttributeSet.try_push_typed'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] in symbols]
        attribute_ids = {target['target_id'] for target in self.catalog['normative_targets']
                         if target['kind'] == 'attribute'}
        for route in manifest['routes']:
            route['normative_target_ids'] = [target_id for target_id in route['normative_target_ids']
                                             if target_id in attribute_ids]

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        self.assertEqual(0, len(gaps), f'{len(gaps)} catalogued typed-attribute structures lack recipes')
        self.assertTrue(report['events'])
        self.assertEqual(symbols, {event['public_api_symbol'] for event in report['events']})
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_attribute_set_routes_use_the_typed_attribute_recipe(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-attr-activation-date']
        symbols = {'kmipkit.attribute_operations.AttributeSet.try_new',
                   'kmipkit.attribute_operations.AttributeSet.try_push',
                   'kmipkit.attribute_operations.AttributeSet.try_push_typed'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] in symbols]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['language'] == 'python'
                                       and route['public_api_symbol'] ==
                                       'kmipkit.extensions.client_batch_item_get_attributes'))

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        typed_events = [event for event in report['events'] if event['public_api_symbol'] in symbols]
        typed_gaps = [gap for gap in gaps if gap['public_api_symbol'] in symbols]
        self.assertEqual(0, len(typed_gaps), f'{len(typed_gaps)} AttributeSet cells have no recipe')
        self.assertEqual(6, len(typed_events))
        self.assertEqual(symbols, {event['public_api_symbol'] for event in typed_events})
        self.assertTrue(all(event['status'] == 'passed' for event in typed_events))

    def test_typed_attribute_routes_cover_nested_attribute_fields_and_direct_tags(self):
        # OASIS KMIP §§4.2, 4.4, 4.14, 4.16, 4.21, 4.27, 4.31, 4.32,
        # 4.46–4.63 define the containing attribute structures and their fields.
        catalog = copy.deepcopy(self.catalog)
        targets = {target['target_id']: target for target in self.catalog['normative_targets']}
        route_symbols = {'kmipkit.typed_attributes.TypedAttribute.create',
                         'kmipkit.typed_attributes.TypedAttribute.from_item',
                         'kmipkit.attribute_operations.AttributeSet.try_new',
                         'kmipkit.attribute_operations.AttributeSet.try_push',
                         'kmipkit.attribute_operations.AttributeSet.try_push_typed'}
        routes = [route for route in self.manifest['routes']
                  if route['language'] == 'python' and route['public_api_symbol'] in route_symbols]
        target_ids = {target_id for route in routes for target_id in route['normative_target_ids']
                      if targets[target_id]['kind'] in ('structure_member', 'tag')}
        catalog['test_cases'] = [case for case in catalog['test_cases']
                                 if case['requires'].get('normative_target_id') in target_ids]
        for case in catalog['test_cases']:
            case.pop('operation', None)
            for field in ('operation_id', 'operation_sequence', 'protocol_scenario'):
                case['requires'].pop(field, None)
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = copy.deepcopy(routes)
        for route in manifest['routes']:
            route['normative_target_ids'] = [target_id for target_id in route['normative_target_ids']
                                             if target_id in target_ids]

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        self.assertEqual(0, len(gaps), f'{len(gaps)} direct-attribute field/tag cells lack recipes')
        self.assertTrue(report['events'])
        self.assertEqual(route_symbols, {event['public_api_symbol'] for event in report['events']})
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_credential_type_enum_routes_preserve_each_normative_value(self):
        # OASIS KMIP 2.0/2.1 §11.11 defines the Credential Type discriminator values.
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-enum-credential-type']
        symbols = {'kmipkit.credentials.CredentialType.from_raw',
                   'kmipkit.credentials.Credential.into_ttlv'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] in symbols]

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        self.assertEqual(0, len(gaps), f'{len(gaps)} Credential Type targets lack typed recipes')
        self.assertEqual(len(list(iter_cells(catalog, manifest, languages=('python',)))),
                         len(report['events']))
        self.assertEqual(symbols, {event['public_api_symbol'] for event in report['events']})
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        credential_type_targets = {target['target_id'] for target in self.catalog['normative_targets']
                                   if target['kind'] == 'enumeration_value'
                                   and 'KMIPKIT-ELEM-ENUMERATION-CREDENTIAL-TYPE'
                                   in target.get('parent_element_ids', [])}
        self.assertEqual(credential_type_targets,
                         {event['target_id'] for event in report['events']})

    def test_all_public_credential_routes_execute_each_normative_target(self):
        # OASIS KMIP 2.0/2.1 §§8.2, 9.4, 9.11 and 9.14 define Authentication,
        # Credential, Credential Value, Credential Type, and Nonce wire shapes.
        from coverage_matrix import _case_obligations, _operation_ids

        symbols = {route['public_api_symbol'] for route in self.manifest['routes']
                   if route['language'] == 'python'
                   and route['public_api_symbol'].startswith('kmipkit.credentials.')}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'].startswith('kmipkit.credentials.')]
        target_ids = {target_id for route in manifest['routes']
                      for target_id in route['normative_target_ids']}
        operation_ids = _operation_ids(self.catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if (obligations := _case_obligations(
                                     case, operation_ids, self.catalog['normative_targets']))
                                 and all(item['target_id'] in target_ids for item in obligations)]

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        self.assertEqual(0, len(gaps), f'{len(gaps)} credential route cells lack executable recipes')
        self.assertEqual(len(list(iter_cells(catalog, manifest, languages=('python',)))),
                         len(report['events']))
        self.assertEqual(symbols, {event['public_api_symbol'] for event in report['events']})
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_all_public_async_request_routes_execute_each_normative_target(self):
        # OASIS KMIP 2.0/2.1 §§4.2, 7.1, 7.25 and 7.34 define the asynchronous
        # request payloads and the Query Asynchronous Requests filter structures.
        from coverage_matrix import _case_obligations, _operation_ids

        symbols = {route['public_api_symbol'] for route in self.manifest['routes']
                   if route['language'] == 'python'
                   and route['public_api_symbol'].startswith('kmipkit.async_operations.')}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'].startswith('kmipkit.async_operations.')]
        target_ids = {target_id for route in manifest['routes']
                      for target_id in route['normative_target_ids']}
        operation_ids = _operation_ids(self.catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if (obligations := _case_obligations(
                                     case, operation_ids, self.catalog['normative_targets']))
                                 and all(item['target_id'] in target_ids for item in obligations)]

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        self.assertEqual(0, len(gaps), f'{len(gaps)} async request route cells lack recipes')
        self.assertEqual(len(list(iter_cells(catalog, manifest, languages=('python',)))),
                         len(report['events']))
        self.assertEqual(symbols, {event['public_api_symbol'] for event in report['events']})
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_attribute_request_constructors_and_encoders_execute_normative_fields(self):
        # OASIS KMIP 2.1 §§6.1.2, 6.1.3, 6.1.13, 6.1.20, 6.1.21, 6.1.34,
        # and 6.1.51 define the selected attribute operation payload tables.
        from coverage_matrix import _case_obligations, _operation_ids

        symbols = {route['public_api_symbol'] for route in self.manifest['routes']
                   if route['language'] == 'python'
                   and route['public_api_symbol'].startswith('kmipkit.attribute_operations.')
                   and route['public_api_symbol'].rsplit('.', 1)[0].endswith('Request')
                   and (route['public_api_symbol'].endswith('.create')
                        or route['public_api_symbol'].endswith('.to_ttlv_payload'))}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] in symbols]
        target_ids = {target_id for route in manifest['routes']
                      for target_id in route['normative_target_ids']}
        operation_ids = _operation_ids(self.catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if (obligations := _case_obligations(
                                     case, operation_ids, self.catalog['normative_targets']))
                                 and all(item['target_id'] in target_ids for item in obligations)]

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        self.assertEqual(0, len(gaps), f'{len(gaps)} attribute request cells lack recipes')
        self.assertTrue(symbols)
        self.assertEqual(len(list(iter_cells(catalog, manifest, languages=('python',)))),
                         len(report['events']))
        self.assertEqual(symbols, {event['public_api_symbol'] for event in report['events']})
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_key_block_routes_execute_each_normative_target(self):
        # OASIS KMIP 2.0/2.1 §§4.12 and 9.1 define Key Block and its fields.
        from coverage_matrix import _case_obligations, _operation_ids

        symbols = {'kmipkit.managed_objects.KeyBlock.from_fields',
                   'kmipkit.managed_objects.KeyBlock.from_item'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] in symbols]
        target_ids = {target_id for route in manifest['routes']
                      for target_id in route['normative_target_ids']}
        operation_ids = _operation_ids(self.catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if (obligations := _case_obligations(
                                     case, operation_ids, self.catalog['normative_targets']))
                                 and all(item['target_id'] in target_ids for item in obligations)]

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        self.assertEqual(0, len(gaps), f'{len(gaps)} Key Block route cells lack recipes')
        self.assertTrue(symbols)
        self.assertEqual(len(list(iter_cells(catalog, manifest, languages=('python',)))),
                         len(report['events']))
        self.assertEqual(symbols, {event['public_api_symbol'] for event in report['events']})
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_reserved_tag_routes_check_rejection_without_a_fabricated_item(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in catalog['test_cases']
                                 if case['requires'].get('target_kind') in ('tag', 'tag_range')
                                 and case['requires'].get('allocation') in ('reserved', 'unused')]
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes'] if route['language'] == 'python'
                              and route['public_api_symbol'].startswith('kmipkit.ttlv.')]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping'))
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual(0, len(gaps), f'{len(gaps)} reserved allocation cells lack rejection checks')
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        decodes = [event for event in report['events'] if event['action'] == 'decode']
        self.assertTrue(decodes)
        for event in decodes:
            self.assertEqual('rejected', event['decode_outcome'])
            self.assertEqual('tag_not_allocated', event['decode_rejection_code'])
            self.assertRegex(event['decode_input_sha256'], '^[0-9a-f]{64}$')
            self.assertNotIn('response_tree_sha256', event)

    def test_every_assigned_tag_is_constructed_and_decoded_with_its_exact_numeric_identity(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in catalog['test_cases']
                                 if case['requires'].get('target_kind') == 'tag'
                                 and case['requires'].get('allocation') == 'assigned']
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes'] if route['language'] == 'python'
                              and route['public_api_symbol'].startswith('kmipkit.ttlv.')]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping'))
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual(0, len(gaps), f'{len(gaps)} assigned-tag cells lack a recipe')
        self.assertGreater(len(report['events']), 3000)
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_tag_enum_expansion_uses_each_normative_target_wire_value(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-enum-tag']
        symbols = {'kmipkit.ttlv.raw_tag_from_raw', 'kmipkit.ttlv.raw_tag_try_checked',
                   'kmipkit.ttlv.ttlv_item_create', 'kmipkit.ttlv.ttlv_decode',
                   'kmipkit.ttlv.ttlv_item_view_from_item'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] in symbols]

        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)

        self.assertEqual(0, len(gaps), f'{len(gaps)} expanded Tag targets lack generic recipes')
        self.assertTrue(report['events'])
        self.assertEqual(symbols, {event['public_api_symbol'] for event in report['events']})
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        tag_targets = {target['target_id'] for target in self.catalog['normative_targets']
                       if target['kind'] in ('tag', 'tag_range')}
        self.assertLessEqual(tag_targets, {event['target_id'] for event in report['events']})

    def test_generic_routes_exercise_every_public_ttlv_data_type(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['requires'].get('target_kind') == 'data_type']
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes'] if route['language'] == 'python'
                              and route['public_api_symbol'].startswith('kmipkit.ttlv.')]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping'))
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertFalse(gaps)
        self.assertEqual(11, len({event['case_id'] for event in report['events']}))

    def test_structure_with_item_route_adds_and_decodes_a_child(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-norm-data-type-data-type-structure']
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] == 'kmipkit.ttlv.ttlv_structure_with_item']
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertEqual(2, len(report['events']))
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        expected = self.runner.digest([[0x4200FF, 1, [[0x4200FF, 2, 7]]]])
        self.assertEqual({expected}, {event['request_tree_sha256'] for event in report['events']})

    def test_query_routes_record_the_peer_rejection_without_calling_it_success(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-op-query']
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_query']
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertEqual(36, len(report['events']))
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        from kmipkit import protocol_values
        for event in report['events']:
            self.assertEqual([1], event['peer_observed_result_status_codes'])
            self.assertEqual([protocol_values.ResultReason.OPERATION_NOT_SUPPORTED],
                             event['peer_observed_result_reason_codes'])
            self.assertEqual('response_received', event['request_delivery_state'])
            self.assertEqual(1, event['transport_invocation_count'])

    def test_async_poll_final_failure_executes_poll_and_preserves_failure_status(self):
        # OASIS KMIP 2.0/2.1 §6.1.38 requires terminal Poll status to mirror the original operation;
        # §6.1.8.1 lists Invalid Object Type as a Create failure reason.
        from unittest.mock import patch
        from kmipkit import protocol_values
        from kmipkit_validator.controlled_peer import ControlledTlsPeer

        case = next(case for case in self.catalog['test_cases']
                    if case['case_id'] == 'test-async-poll-final-operation-failure')
        target = next(target for target in self.catalog['normative_targets']
                      if target['target_id'] == 'KMIPKIT-ELEM-OP-C2S-POLL')
        cell = {
            'case_id': case['case_id'], 'target_id': target['target_id'],
            'protocol_version': '2.1', 'encoding': 'TTLV', 'transport': 'raw_tls',
            'public_api_symbol': 'kmipkit.client.Client.execute_poll',
            'execution_api_symbol': 'kmipkit.client.Client.execute_poll',
            'workflow_operation': 'Poll', 'action': 'decode',
        }
        peers = []

        def record_peer(**options):
            peer = ControlledTlsPeer(**options)
            peers.append(peer)
            return peer

        symbols = [cell['public_api_symbol'], cell['execution_api_symbol']]
        trace = self.runner.CallTrace(symbols)
        with patch('kmipkit_validator.controlled_peer.ControlledTlsPeer', side_effect=record_peer):
            with trace:
                event = self.runner.execute_cell(cell, case, target, trace=trace)

        self.assertEqual({'Poll'}, {peer.request_observation['operation'] for peer in peers})
        self.assertTrue(set(symbols) <= trace.observed)
        self.assertEqual([protocol_values.ResultStatus.OPERATION_FAILED],
                         event['peer_observed_result_status_codes'])
        self.assertEqual([protocol_values.ResultReason.INVALID_OBJECT_TYPE],
                         event['peer_observed_result_reason_codes'])
        self.assertEqual('response_received', event['request_delivery_state'])
        self.assertEqual(1, event['transport_invocation_count'])

    def test_cancel_payload_routes_send_cancel_even_when_case_primary_operation_is_create(self):
        # OASIS KMIP 2.0/2.1 §6.1.5 defines Cancel as a separate request operation.
        from unittest.mock import patch
        from kmipkit import protocol_values
        from kmipkit_validator.controlled_peer import ControlledTlsPeer

        case = next(case for case in self.catalog['test_cases']
                    if case['case_id'] == 'test-async-cancel-result-canceled')
        target = next(target for target in self.catalog['normative_targets']
                      if target['target_id'] == 'KMIPKIT-PAYLOAD-CLIENT-CANCEL-REQUEST-20-001')
        cell = next(cell for cell in iter_cells(self.catalog, self.manifest, languages=('python',))
                    if cell['case_id'] == case['case_id'] and cell['target_id'] == target['target_id']
                    and cell['execution_route_id'] == 'python.execute.cancel'
                    and cell['protocol_version'] == '2.0' and cell['encoding'] == 'TTLV'
                    and cell['transport'] == 'raw_tls' and cell['action'] == 'construct')
        peers = []

        def record_peer(**options):
            peer = ControlledTlsPeer(**options)
            peers.append(peer)
            return peer

        symbols = [cell['public_api_symbol'], cell['execution_api_symbol']]
        trace = self.runner.CallTrace(symbols)
        with patch('kmipkit_validator.controlled_peer.ControlledTlsPeer', side_effect=record_peer):
            with trace:
                event = self.runner.execute_cell(cell, case, target, trace=trace)

        self.assertEqual({'Cancel'}, {peer.request_observation['operation'] for peer in peers})
        self.assertTrue(set(symbols) <= trace.observed)
        self.assertEqual([protocol_values.ResultStatus.OPERATION_FAILED],
                         event['peer_observed_result_status_codes'])
        self.assertEqual([protocol_values.ResultReason.OPERATION_NOT_SUPPORTED],
                         event['peer_observed_result_reason_codes'])
        self.assertEqual('response_received', event['request_delivery_state'])
        self.assertEqual(1, event['transport_invocation_count'])

    def test_unknown_async_correlation_is_stable_sequentially_and_concurrently(self):
        # OASIS KMIP 2.0/2.1 Specification §6.1.38.1, Table 277 lists Invalid Asynchronous Correlation Value.
        from concurrent.futures import ProcessPoolExecutor
        from multiprocessing import get_context
        from kmipkit import protocol_values

        catalog = copy.deepcopy(self.catalog)
        case = next(case for case in catalog['test_cases']
                    if case['case_id'] == 'test-async-unknown-correlation-rejected')
        catalog['test_cases'] = [case]
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        jobs = [(cell, case, targets[cell['target_id']])
                for cell in iter_cells(catalog, self.manifest, languages=('python',))
                if cell['route_id'] == 'python.typed-protocol.client.execute_poll']

        def assert_failed_polls(results):
            self.assertTrue(all(kind == 'event' and value['status'] == 'passed'
                                for kind, value in results))
            self.assertTrue(all(value['peer_observed_result_status_codes'] ==
                                [protocol_values.ResultStatus.OPERATION_FAILED]
                                for _, value in results))
            self.assertTrue(all(value['peer_observed_result_reason_codes'] ==
                                [protocol_values.ResultReason.INVALID_ASYNCHRONOUS_CORRELATION_VALUE]
                                for _, value in results))

        sequential_runs = [
            [self.runner._execute_matrix_cell(*job) for job in jobs]
            for _ in range(2)
        ]
        for results in sequential_runs:
            assert_failed_polls(results)

        with ProcessPoolExecutor(max_workers=8, mp_context=get_context('spawn')) as executor:
            concurrent_runs = [
                [future.result() for future in
                 [executor.submit(self.runner._execute_matrix_cell, *job) for job in jobs]]
                for _ in range(2)
            ]
        for results in concurrent_runs:
            assert_failed_polls(results)

    def test_batch_workflow_builds_the_cell_operation_not_the_case_primary_operation(self):
        # OASIS KMIP 2.0/2.1 §§6.1.8 and 6.1.21 define distinct Create and Get Attribute List requests.
        from unittest.mock import patch
        from kmipkit import protocol_values
        from kmipkit_validator.controlled_peer import ControlledTlsPeer

        case = next(case for case in self.catalog['test_cases']
                    if case['case_id'] == 'test-batch-create-activate-destroy')
        target = next(target for target in self.catalog['normative_targets']
                      if target['target_id'] == 'KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST')
        cell = {
            'case_id': case['case_id'], 'target_id': target['target_id'],
            'protocol_version': '2.1', 'encoding': 'TTLV', 'transport': 'raw_tls',
            'public_api_symbol': 'kmipkit.extensions.client_batch_item_get_attribute_list',
            'execution_api_symbol': 'kmipkit.client.Client.execute',
            'workflow_operation': 'Get Attribute List', 'action': 'construct',
        }
        peers = []

        def record_peer(**options):
            peer = ControlledTlsPeer(**options)
            peers.append(peer)
            return peer

        symbols = [cell['public_api_symbol'], cell['execution_api_symbol']]
        trace = self.runner.CallTrace(symbols)
        with patch('kmipkit_validator.controlled_peer.ControlledTlsPeer', side_effect=record_peer):
            with trace:
                event = self.runner.execute_cell(cell, case, target, trace=trace)

        self.assertEqual({'Get Attribute List'}, {peer.request_observation['operation'] for peer in peers})
        self.assertTrue(set(symbols) <= trace.observed)
        self.assertEqual([protocol_values.ResultStatus.OPERATION_FAILED],
                         event['peer_observed_result_status_codes'])

    def test_create_routes_validate_peer_rejection_and_all_wire_encodings(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-op-create']
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python'
                              and route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_create']
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertTrue(report['events'])
        from kmipkit import protocol_values
        self.assertTrue(all(event['status'] == 'passed'
                            and event['peer_observed_result_status_codes'] == [protocol_values.ResultStatus.OPERATION_FAILED]
                            and event['peer_observed_result_reason_codes'] == [protocol_values.ResultReason.OPERATION_NOT_SUPPORTED]
                            for event in report['events']))
        by_encoding = {}
        for event in report['events']:
            by_encoding.setdefault(event['encoding'], set()).add(
                (event['request_tree_sha256'], event['response_tree_sha256']))
        self.assertEqual({'TTLV', 'JSON', 'XML'}, set(by_encoding))
        self.assertEqual(by_encoding['JSON'], by_encoding['XML'])

    def test_generic_ttlv_routes_encode_and_decode_the_client_operation_value(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-op-ping']
        symbols = {'kmipkit.ttlv.ttlv_item_create', 'kmipkit.ttlv.ttlv_decode',
                   'kmipkit.ttlv.ttlv_item_view_from_item'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes']
                              if route['language'] == 'python' and route['public_api_symbol'] in symbols]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertEqual(3, len(report['events']))
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        expected = self.runner.digest([[0x42005C, 5, 0x3B]])
        self.assertEqual({expected}, {event['request_tree_sha256'] for event in report['events']})

    def test_generic_ttlv_routes_cover_every_catalogued_client_and_server_operation(self):
        catalog = copy.deepcopy(self.catalog)
        operation_ids = {target['target_id'] for target in catalog['normative_targets']
                         if target['kind'] == 'operation'}
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['requires'].get('normative_target_id') in operation_ids]
        symbols = {'kmipkit.ttlv.ttlv_item_create', 'kmipkit.ttlv.ttlv_decode',
                   'kmipkit.ttlv.ttlv_item_view_from_item'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes']
                              if route['language'] == 'python' and route['public_api_symbol'] in symbols]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual(0, len(gaps), f'{len(gaps)} operation IDs lack generic TTLV evidence')
        self.assertTrue(report['events'])
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        self.assertEqual(operation_ids, {event['target_id'] for event in report['events']})

    def test_generic_ttlv_routes_encode_and_decode_operation_payload_fields(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-norm-payload-client-activate-request-20-001']
        symbols = {'kmipkit.ttlv.ttlv_item_create', 'kmipkit.ttlv.ttlv_decode',
                   'kmipkit.ttlv.ttlv_item_view_from_item'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes']
                              if route['language'] == 'python' and route['public_api_symbol'] in symbols]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertEqual(6, len(report['events']))
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        self.assertEqual({'KMIPKIT-ELEM-OP-C2S-ACTIVATE',
                          'KMIPKIT-PAYLOAD-CLIENT-ACTIVATE-REQUEST-20-001'},
                         {event['target_id'] for event in report['events']})
        operation_hash = self.runner.digest([[0x42005C, 5, 0x12]])
        operation_events = [event for event in report['events']
                            if event['target_id'] == 'KMIPKIT-ELEM-OP-C2S-ACTIVATE']
        self.assertEqual({operation_hash}, {event['request_tree_sha256'] for event in operation_events})

    def test_generic_ttlv_routes_construct_the_catalogued_attribute_tag_and_type(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-attr-activation-date']
        symbols = {'kmipkit.ttlv.ttlv_item_create', 'kmipkit.ttlv.ttlv_decode',
                   'kmipkit.ttlv.ttlv_item_view_from_item'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes']
                              if route['language'] == 'python' and route['public_api_symbol'] in symbols]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertEqual(12, len(report['events']))
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        by_target = {}
        for event in report['events']:
            by_target.setdefault(event['target_id'], set()).add(event['request_tree_sha256'])
        self.assertEqual({'KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE', 'KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES'}, set(by_target))
        self.assertEqual({self.runner.digest([[0x420001, 9, -1], [0x420001, 9, 0], [0x420001, 9, 1]])},
                         by_target['KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE'])
        self.assertEqual({self.runner.digest([[0x42005C, 5, 11]])}, by_target['KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES'])

    def test_generic_ttlv_routes_construct_the_catalogued_object_type_enum(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['case_id'] == 'test-obj-certificate']
        symbols = {'kmipkit.ttlv.ttlv_item_create', 'kmipkit.ttlv.ttlv_decode',
                   'kmipkit.ttlv.ttlv_item_view_from_item'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes']
                              if route['language'] == 'python' and route['public_api_symbol'] in symbols]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertEqual(12, len(report['events']))
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        by_target = {}
        for event in report['events']:
            by_target.setdefault(event['target_id'], set()).add(event['request_tree_sha256'])
        self.assertEqual({'KMIPKIT-ELEM-OBJECT-TYPE-CERTIFICATE', 'KMIPKIT-ELEM-OP-C2S-REGISTER'}, set(by_target))
        self.assertEqual({self.runner.digest([[0x420057, 5, 1]])},
                         by_target['KMIPKIT-ELEM-OBJECT-TYPE-CERTIFICATE'])
        self.assertEqual({self.runner.digest([[0x42005C, 5, 3]])}, by_target['KMIPKIT-ELEM-OP-C2S-REGISTER'])

    def test_generic_ttlv_routes_cover_every_catalogued_attribute_object_and_enum_value(self):
        kinds = {'attribute', 'object_type', 'enumeration_value', 'referenced_enum_value', 'bitmask_value'}
        targets = {target['target_id']: target for target in self.catalog['normative_targets']
                   if target['kind'] in kinds and (target['kind'] != 'attribute'
                       or any(parent.startswith('KMIPKIT-ELEM-TAG-') for parent in target.get('parent_element_ids', []))
                       or target['name'] == 'Vendor Attribute')}
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['requires'].get('normative_target_id') in targets]
        symbols = {'kmipkit.ttlv.ttlv_item_create', 'kmipkit.ttlv.ttlv_decode',
                   'kmipkit.ttlv.ttlv_item_view_from_item'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python' and route['public_api_symbol'] in symbols]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual(0, len(gaps), f'{len(gaps)} catalogued attribute/object/enum cells lack generic evidence')
        self.assertTrue(report['events'])
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        self.assertLessEqual(set(targets), {event['target_id'] for event in report['events']})

    def test_generic_ttlv_routes_cover_tagged_fields_and_structures(self):
        kinds = {'structure_member', 'message_field', 'object_structure', 'operation_structure',
                 'attribute_structure', 'credential'}
        targets = {target['target_id']: target for target in self.catalog['normative_targets']
                   if target['kind'] in kinds}
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in self.catalog['test_cases']
                                 if case['requires'].get('normative_target_id') in targets]
        symbols = {'kmipkit.ttlv.ttlv_item_create', 'kmipkit.ttlv.ttlv_decode',
                   'kmipkit.ttlv.ttlv_item_view_from_item'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in self.manifest['routes']
                              if route['language'] == 'python' and route['public_api_symbol'] in symbols]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual(0, len(gaps), f'{len(gaps)} tagged structure cells lack generic evidence')
        self.assertTrue(report['events'])
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        self.assertLessEqual(set(targets), {event['target_id'] for event in report['events']})

    def test_ci_requires_shared_changes_and_matrix_completeness(self):
        workflow = (ROOT / '.github/workflows/interop-python.yml').read_text()
        self.assertIn('python -m kmipkit_validator matrix', workflow)
        self.assertIn('validate_route_evidence.py', workflow)
        self.assertIn('interop/shared/|bindings/python/', workflow)
        self.assertIn('route-execution-evidence.json', workflow)

    def test_cli_matrix_dispatches_without_requiring_a_vendor_connection(self):
        from kmipkit_validator import cli
        from unittest.mock import patch
        self.assertTrue(hasattr(self.runner, 'run_command'), 'matrix command must run the completeness gate')
        with patch.object(self.runner, 'run_command', return_value=1) as execute:
            self.assertEqual(1, cli.main(['matrix', '--output-dir', 'build/matrix']))
        execute.assert_called_once()

    def test_exact_installed_wheel_files_are_checked_before_evidence(self):
        self.assertTrue(hasattr(self.runner, 'verify_installed_artifact'), 'wheel digest alone cannot prove installation identity')
        from kmipkit_validator.contract import validate_artifact
        self.runner.verify_installed_artifact(validate_artifact())

    def test_call_observation_does_not_confuse_shared_synchronization_wrappers(self):
        from kmipkit import ttlv
        with self.runner.CallTrace(['kmipkit.ttlv.ttlv_decode']) as trace:
            with ttlv.codec_limits_defaults():
                pass
        self.assertNotIn('kmipkit.ttlv.ttlv_decode', trace.observed)

    def test_discover_versions_checks_positive_native_response_and_public_field_read(self):
        self.assertTrue(hasattr(self.runner, '_positive_fixture_cell'), 'a positive fixture runner is required')
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in catalog['test_cases'] if case['case_id'] == 'test-op-discover-versions']
        symbols = {'kmipkit.extensions.client_batch_item_discover_versions',
                   'kmipkit.client.BatchItemResult.supported_versions'}
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes'] if route['public_api_symbol'] in symbols]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual(0, len(gaps), f'{len(gaps)} discovery cells lack positive checks')
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        self.assertEqual(38, len(report['events']))
        self.assertEqual(symbols, {event['public_api_symbol'] for event in report['events']})

    def test_decode_wire_digest_is_the_received_response(self):
        import hashlib
        from unittest.mock import patch
        from kmipkit_validator.controlled_peer import ControlledTlsPeer
        peers = []
        def peer_factory(**options):
            peer = ControlledTlsPeer(**options)
            peers.append(peer)
            return peer
        cell = {'protocol_version': '2.1', 'encoding': 'TTLV', 'transport': 'raw_tls',
                'public_api_symbol': 'kmipkit.extensions.client_batch_item_ping', 'action': 'decode'}
        case = next(case for case in self.catalog['test_cases'] if case['case_id'] == 'test-op-ping')
        symbol = cell['public_api_symbol']
        with self.runner.CallTrace([symbol]) as trace:
            with patch('kmipkit_validator.controlled_peer.ControlledTlsPeer', side_effect=peer_factory):
                observed = self.runner.execute_cell(cell, case, {}, trace=trace)
        self.assertIn(symbol, trace.observed)
        self.assertEqual(hashlib.sha256(peers[0].responses[0]).hexdigest(), observed['wire_bytes_sha256'])
        self.assertNotEqual(hashlib.sha256(peers[0].requests[0]).hexdigest(), observed['wire_bytes_sha256'])

    def test_version_rejections_prove_not_sent_and_zero_transport_attempts(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in catalog['test_cases']
                                 if case['requires'].get('expected_status') == 'NotSent']
        symbols = {'kmipkit.extensions.client_batch_item_' + operation for operation in
                   ('ping', 'get_constraints', 'set_constraints', 'set_defaults')}
        symbols.update({'kmipkit.client.Client.execute_process',
                        'kmipkit.client.Client.execute_query_async_requests'})
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes'] if route['public_api_symbol'] in symbols]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertFalse(gaps)
        self.assertEqual(72, len(report['events']))
        self.assertEqual({'construct', 'send'}, {event['action'] for event in report['events']})
        for event in report['events']:
            self.assertNotIn('response_tree_sha256', event)
            self.assertEqual('not_sent', event['request_delivery_state'])
            self.assertEqual(0, event['transport_invocation_count'])
        summary = validate_report(report, catalog, manifest,
                                  catalog_sha256=report['catalog_sha256'],
                                  route_manifest_sha256=report['route_manifest_sha256'],
                                  expected_artifact_sha256=self.artifact['artifact_sha256'])
        self.assertTrue(summary['all_invocations_passed'])

    def test_ping_network_cells_execute_public_client_over_all_supported_combinations(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [next(case for case in catalog['test_cases'] if case['case_id'] == 'test-op-ping')]
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes']
                              if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping']
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertFalse(gaps)
        self.assertEqual(18, len(report['events']))
        for event in report['events']:
            self.assertEqual('passed', event['status'])
            self.assertEqual([0], event['peer_observed_result_status_codes'])
            self.assertNotIn('peer_observed_result_reason_codes', event)
            self.assertEqual('response_received', event['request_delivery_state'])
        by_encoding = {}
        for event in report['events']:
            by_encoding.setdefault(event['encoding'], set()).add(
                (event['request_tree_sha256'], event['response_tree_sha256']))
        self.assertEqual({'TTLV', 'JSON', 'XML'}, set(by_encoding))
        self.assertTrue(all(len(values) == 1 for values in by_encoding.values()))
        self.assertEqual(by_encoding['JSON'], by_encoding['XML'])

    def test_network_matrix_cells_execute_concurrently_and_keep_catalog_order(self):
        from threading import Barrier
        from unittest.mock import patch
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [next(case for case in catalog['test_cases']
                                      if case['case_id'] == 'test-op-ping')]
        manifest = copy.deepcopy(self.manifest)
        symbol = 'kmipkit.extensions.client_batch_item_ping'
        manifest['routes'] = [route for route in manifest['routes']
                              if route['public_api_symbol'] == symbol]
        expected_cells = list(iter_cells(catalog, manifest, languages=('python',)))
        rendezvous = Barrier(2)

        def execute(cell, case, target, *, trace):
            trace.record_invocation(cell['public_api_symbol'])
            if cell.get('execution_api_symbol'):
                trace.record_invocation(cell['execution_api_symbol'])
            rendezvous.wait(timeout=1)
            return {'request_tree_sha256': 'a' * 64, 'response_tree_sha256': 'b' * 64}

        with patch.object(self.runner, 'execute_cell', side_effect=execute):
            report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertEqual(len(expected_cells), len(report['events']))
        self.assertEqual(
            [(cell['protocol_version'], cell['encoding'], cell['transport']) for cell in expected_cells],
            [(event['protocol_version'], event['encoding'], event['transport']) for event in report['events']],
        )
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_local_matrix_cells_execute_concurrently_and_keep_catalog_order(self):
        from threading import Barrier
        from unittest.mock import patch
        catalog = copy.deepcopy(self.catalog)
        case = next(case for case in catalog['test_cases']
                    if case['requires'].get('normative_target_id') == 'KMIPKIT-ELEM-DATA-TYPE-INTEGER')
        target_id = case['requires']['normative_target_id']
        catalog['test_cases'] = [case]
        catalog['normative_targets'] = [target for target in catalog['normative_targets']
                                        if target['target_id'] == target_id]
        symbol = 'kmipkit.ttlv.ttlv_value_integer'
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes']
                              if route['public_api_symbol'] == symbol]
        expected_cells = list(iter_cells(catalog, manifest, languages=('python',)))
        rendezvous = Barrier(2)

        def execute(cell, case, target, *, trace):
            trace.record_invocation(cell['public_api_symbol'])
            rendezvous.wait(timeout=1)
            return {'request_tree_sha256': 'a' * 64, 'response_tree_sha256': 'b' * 64}

        with patch.object(self.runner, 'execute_cell', side_effect=execute):
            report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertEqual(len(expected_cells), len(report['events']))
        self.assertEqual(
            [(cell['protocol_version'], cell['encoding'], cell['transport']) for cell in expected_cells],
            [(event['protocol_version'], event['encoding'], event['transport']) for event in report['events']],
        )
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_process_matrix_backend_executes_local_routes_in_catalog_order(self):
        catalog, manifest = self.integer_fixture()
        manifest['routes'] = [route for route in manifest['routes'] if route['mode'] == 'local']
        expected_cells = list(iter_cells(catalog, manifest, languages=('python',)))
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact, process_workers=2)
        self.assertEqual([], gaps)
        self.assertEqual(len(expected_cells), len(report['events']))
        self.assertEqual(
            [(cell['case_id'], cell['target_id'], cell['public_api_symbol'], cell['protocol_version'])
             for cell in expected_cells],
            [(event['case_id'], event['target_id'], event['public_api_symbol'], event['protocol_version'])
             for event in report['events']],
        )
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_process_matrix_backend_checks_controlled_peer_routes(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [case for case in catalog['test_cases']
                                 if case['case_id'] == 'test-op-ping']
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes']
                              if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping']
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact, process_workers=2)
        self.assertEqual([], gaps)
        self.assertEqual(18, len(report['events']))
        self.assertTrue(all(event['status'] == 'passed'
                            and event['peer_observed_result_status_codes'] == [0]
                            and event['peer_observed_response_count'] == 1
                            and event['request_delivery_state'] == 'response_received'
                            for event in report['events']))

    def test_construct_and_decode_are_distinct_invocations(self):
        catalog, manifest = self.integer_fixture()
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertFalse(gaps)
        self.assertEqual(len(report['events']), len({event['invocation_id'] for event in report['events']}))
        for event in report['events']:
            field = 'response_tree_sha256' if event['action'] == 'decode' else 'request_tree_sha256'
            self.assertRegex(event[field], '^[0-9a-f]{64}$')


if __name__ == '__main__':
    unittest.main()
