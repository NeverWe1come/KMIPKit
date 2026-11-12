"""FFI coverage keeps both libraries and consumers across Cargo output layouts."""
from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HELPER = Path(__file__).resolve().parents[1] / 'ffi_coverage_objects.py'
MODULE = None
if HELPER.is_file():
    spec = importlib.util.spec_from_file_location('ffi_coverage_objects', HELPER)
    MODULE = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(MODULE)


class FfiCoverageObjectTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(MODULE, 'FFI coverage artifact discovery is missing')

    def fixture(self, directory, layout):
        root = Path(directory) / 'coverage target'
        debug = root / 'debug'
        libraries = debug / ('deps' if layout == 'classic' else 'build/kmipkit-ffi/library-hash/out')
        tests = debug / ('deps' if layout == 'classic' else 'build/kmipkit-ffi/test-hash/out')
        objects = [libraries / 'libkmipkit_ffi.so', libraries / 'libkmipkit_ffi.a',
                   tests / 'kmipkit_ffi-012abc', tests / 'c_api_coverage-345def']
        for path in objects:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'fixture object')
            path.chmod(0o755)
        log = root / 'cargo.log'
        log.write_text(f'Running unittests src/lib.rs ({objects[2]})\n'
                       f'Running tests/c_api_coverage.rs ({objects[3]})\n')
        log.with_suffix('.artifacts.jsonl').write_text(json.dumps({
            'reason': 'compiler-artifact', 'target': {'name': 'kmipkit_ffi'},
            'profile': {'test': False}, 'filenames': [str(path) for path in objects[:2]],
        }) + '\n')
        return root, log, objects

    def test_preserves_both_libraries_and_consumers_in_both_cargo_layouts(self):
        for layout in ('classic', 'cargo-1.100'):
            with self.subTest(layout=layout), tempfile.TemporaryDirectory() as directory:
                root, log, objects = self.fixture(directory, layout)
                self.assertEqual(MODULE.collect_ffi_objects(root, log, log.with_suffix('.artifacts.jsonl')), objects)

    def test_missing_static_library_cannot_reduce_the_coverage_denominator(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, objects = self.fixture(directory, 'cargo-1.100')
            objects[1].unlink()
            with self.assertRaisesRegex(ValueError, 'libkmipkit_ffi.a'):
                MODULE.collect_ffi_objects(root, log, log.with_suffix('.artifacts.jsonl'))

    def test_unreferenced_stale_library_cannot_enter_the_coverage_report(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, objects = self.fixture(directory, 'cargo-1.100')
            stale = root / 'debug/build/kmipkit-ffi/stale/out/libkmipkit_ffi.so'
            stale.parent.mkdir(parents=True)
            stale.write_bytes(b'stale object')
            self.assertEqual(MODULE.collect_ffi_objects(
                root, log, log.with_suffix('.artifacts.jsonl')), objects)
            with log.with_suffix('.artifacts.jsonl').open('a') as receipt:
                receipt.write(json.dumps({'reason': 'compiler-artifact',
                    'target': {'name': 'kmipkit_ffi'}, 'profile': {'test': False},
                    'filenames': [str(stale)]}) + '\n')
            with self.assertRaisesRegex(ValueError, 'ambiguous'):
                MODULE.collect_ffi_objects(root, log, log.with_suffix('.artifacts.jsonl'))

    def test_duplicate_hardlink_does_not_make_the_current_library_ambiguous(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, objects = self.fixture(directory, 'classic')
            alias = root / 'debug/libkmipkit_ffi.so'
            os.link(objects[0], alias)
            actual = MODULE.collect_ffi_objects(root, log, log.with_suffix('.artifacts.jsonl'))
            self.assertTrue(actual[0].samefile(objects[0]))
            self.assertEqual(actual[1:], objects[1:])

    def test_reported_consumer_must_belong_to_the_instrumented_target(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, objects = self.fixture(directory, 'classic')
            outside = Path(directory) / objects[3].name
            outside.write_bytes(b'foreign consumer')
            outside.chmod(0o755)
            log.write_text(log.read_text().replace(str(objects[3]), str(outside)))
            with self.assertRaisesRegex(ValueError, 'outside'):
                MODULE.collect_ffi_objects(root, log, log.with_suffix('.artifacts.jsonl'))

    def test_arbitrary_receipt_filename_is_rejected_before_reading(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, _ = self.fixture(directory, 'classic')
            private = root / 'credentials.json'
            private.write_bytes(log.with_suffix('.artifacts.jsonl').read_bytes())
            with self.assertRaisesRegex(ValueError, 'receipt'):
                MODULE.collect_ffi_objects(root, log, private)

    def test_cargo_log_and_receipt_must_share_the_report_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, _ = self.fixture(directory, 'classic')
            foreign = Path(directory) / 'foreign'
            foreign.mkdir()
            receipt = foreign / log.with_suffix('.artifacts.jsonl').name
            receipt.write_bytes(log.with_suffix('.artifacts.jsonl').read_bytes())
            with self.assertRaisesRegex(ValueError, 'report directory'):
                MODULE.collect_ffi_objects(root, log, receipt)

    def test_cli_cannot_select_a_foreign_cargo_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, _ = self.fixture(directory, 'classic')
            result = subprocess.run([sys.executable, str(HELPER), '--target-root', str(root),
                '--cargo-log', str(log), '--artifacts-json',
                str(log.with_suffix('.artifacts.jsonl')), '--manifest-path',
                str(Path(directory) / 'foreign-Cargo.toml')], capture_output=True)
            self.assertNotEqual(result.returncode, 0, 'a CLI override must not select executable project code')

    def test_cli_preserves_paths_with_spaces_as_null_delimited_objects(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, objects = self.fixture(directory, 'cargo-1.100')
            result = subprocess.run([sys.executable, str(HELPER), '--target-root', str(root),
                                     '--cargo-log', str(log), '--artifacts-json',
                                     str(log.with_suffix('.artifacts.jsonl'))], capture_output=True, check=True)
            self.assertEqual(result.stdout.split(b'\0'),
                             [os.fsencode(path) for path in objects] + [b''])


if __name__ == '__main__':
    unittest.main()
