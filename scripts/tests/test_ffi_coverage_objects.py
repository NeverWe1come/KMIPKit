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
        log = root / 'cargo-llvm-cov.log'
        log.write_text(f'Running unittests src/lib.rs ({objects[2]})\n'
                       f'Running tests/c_api_coverage.rs ({objects[3]})\n')
        (log.parent / 'cargo-artifacts.jsonl').write_text(json.dumps({
            'reason': 'compiler-artifact', 'target': {'name': 'kmipkit_ffi'},
            'profile': {'test': False}, 'filenames': [str(path) for path in objects[:2]],
        }) + '\n')
        return root, log, objects

    def test_preserves_both_libraries_and_consumers_in_both_cargo_layouts(self):
        for layout in ('classic', 'cargo-1.100'):
            with self.subTest(layout=layout), tempfile.TemporaryDirectory() as directory:
                root, log, objects = self.fixture(directory, layout)
                self.assertEqual(MODULE.collect_ffi_objects(root, log, (log.parent / 'cargo-artifacts.jsonl')), objects)

    def test_missing_static_library_cannot_reduce_the_coverage_denominator(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, objects = self.fixture(directory, 'cargo-1.100')
            objects[1].unlink()
            with self.assertRaisesRegex(ValueError, 'libkmipkit_ffi.a'):
                MODULE.collect_ffi_objects(root, log, (log.parent / 'cargo-artifacts.jsonl'))

    def test_unreferenced_stale_library_cannot_enter_the_coverage_report(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, objects = self.fixture(directory, 'cargo-1.100')
            stale = root / 'debug/build/kmipkit-ffi/stale/out/libkmipkit_ffi.so'
            stale.parent.mkdir(parents=True)
            stale.write_bytes(b'stale object')
            self.assertEqual(MODULE.collect_ffi_objects(
                root, log, (log.parent / 'cargo-artifacts.jsonl')), objects)
            with (log.parent / 'cargo-artifacts.jsonl').open('a') as receipt:
                receipt.write(json.dumps({'reason': 'compiler-artifact',
                    'target': {'name': 'kmipkit_ffi'}, 'profile': {'test': False},
                    'filenames': [str(stale)]}) + '\n')
            with self.assertRaisesRegex(ValueError, 'ambiguous'):
                MODULE.collect_ffi_objects(root, log, (log.parent / 'cargo-artifacts.jsonl'))

    def test_duplicate_hardlink_does_not_make_the_current_library_ambiguous(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, objects = self.fixture(directory, 'classic')
            alias = root / 'debug/libkmipkit_ffi.so'
            os.link(objects[0], alias)
            actual = MODULE.collect_ffi_objects(root, log, (log.parent / 'cargo-artifacts.jsonl'))
            self.assertEqual(len(actual), len(objects))
            self.assertTrue(all(reported.samefile(expected)
                                for reported, expected in zip(actual, objects)))

    def test_reported_consumer_must_belong_to_the_instrumented_target(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, objects = self.fixture(directory, 'classic')
            outside = Path(directory) / objects[3].name
            outside.write_bytes(b'foreign consumer')
            outside.chmod(0o755)
            log.write_text(log.read_text().replace(str(objects[3]), str(outside)))
            with self.assertRaisesRegex(ValueError, 'outside'):
                MODULE.collect_ffi_objects(root, log, (log.parent / 'cargo-artifacts.jsonl'))

    def test_arbitrary_receipt_filename_is_rejected_before_reading(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, _ = self.fixture(directory, 'classic')
            private = root / 'credentials.json'
            private.write_bytes((log.parent / 'cargo-artifacts.jsonl').read_bytes())
            with self.assertRaisesRegex(ValueError, 'receipt'):
                MODULE.collect_ffi_objects(root, log, private)

    def test_cargo_log_and_receipt_must_share_the_report_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, _ = self.fixture(directory, 'classic')
            foreign = Path(directory) / 'foreign'
            foreign.mkdir()
            receipt = foreign / 'cargo-artifacts.jsonl'
            receipt.write_bytes((log.parent / 'cargo-artifacts.jsonl').read_bytes())
            with self.assertRaisesRegex(ValueError, 'report directory'):
                MODULE.collect_ffi_objects(root, log, receipt)

    def test_cli_cannot_select_a_foreign_cargo_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, _ = self.fixture(directory, 'classic')
            result = subprocess.run([sys.executable, str(HELPER), '--target-root', str(root),
                '--report-dir', str(log.parent), '--manifest-path',
                str(Path(directory) / 'foreign-Cargo.toml')], capture_output=True)
            self.assertNotEqual(result.returncode, 0, 'a CLI override must not select executable project code')

    def test_report_directory_is_rejected_before_creation_outside_authorized_roots(self):
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            root, _, _ = self.fixture(directory, 'classic')
            outside = Path(directory) / 'outside'
            self.assertTrue(callable(getattr(MODULE, 'prepare_report_dir', None)),
                            'coverage report directory guard is missing')
            with patch.object(MODULE, '_authorized_roots', return_value=[root.resolve()]):
                with self.assertRaisesRegex(ValueError, 'outside'):
                    MODULE.prepare_report_dir(outside)
            self.assertFalse(outside.exists())

    def test_arbitrary_log_filename_is_rejected_before_reading(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, _ = self.fixture(directory, 'classic')
            private = root / 'credentials.log'
            private.write_bytes(log.read_bytes())
            with self.assertRaisesRegex(ValueError, 'log'):
                MODULE.collect_ffi_objects(root, private, (log.parent / 'cargo-artifacts.jsonl'))

    @unittest.skipUnless(sys.platform == 'linux', 'the real FFI coverage collector is Linux-only')
    def test_collector_rejects_symlinked_output_before_overwriting_its_target(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, _ = self.fixture(directory, 'classic')
            private = Path(directory) / 'private-fixture.log'
            private.write_bytes(b'preserve this fixture')
            log.unlink()
            log.symlink_to(private)
            tools = Path(directory) / 'tools'
            tools.mkdir()
            cargo = tools / 'cargo'
            cargo.write_text('#!/bin/sh\nprintf "fake compiler output\\n"\nexit 1\n')
            cargo.chmod(0o755)
            environment = {**os.environ, 'PATH': str(tools) + os.pathsep + os.environ.get('PATH', '')}
            result = subprocess.run(['bash', str(HELPER.with_name('collect_ffi_coverage.sh')), str(root)],
                                     env=environment, capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(private.read_bytes(), b'preserve this fixture')

    def test_cli_preserves_paths_with_spaces_as_null_delimited_objects(self):
        with tempfile.TemporaryDirectory() as directory:
            root, log, objects = self.fixture(directory, 'cargo-1.100')
            result = subprocess.run([sys.executable, str(HELPER), '--target-root', str(root),
                                     '--report-dir', str(log.parent)], capture_output=True, check=True)
            reported = result.stdout.split(b'\0')
            self.assertEqual(reported.pop(), b'')
            # Windows can resolve a temporary path from its short 8.3 alias to
            # the canonical long path (including different case). Compare the
            # filesystem objects while still checking that the CLI emits one
            # NUL-delimited path per selected artifact.
            self.assertEqual(len(reported), len(objects))
            self.assertTrue(all(Path(os.fsdecode(actual)).samefile(expected)
                                for actual, expected in zip(reported, objects)))


if __name__ == '__main__':
    unittest.main()
