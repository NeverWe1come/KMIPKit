"""Resolve complete instrumented FFI objects without assuming Cargo's layout."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tempfile

_REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
_CARGO_MANIFEST = _REPOSITORY_ROOT / 'Cargo.toml'
_CARGO_LOG_NAME = 'cargo-llvm-cov.log'
_RECEIPT_NAME = 'cargo-artifacts.jsonl'
_LIBRARY_NAMES = ('libkmipkit_ffi.so', 'libkmipkit_ffi.a')
_CONSUMER_PREFIXES = {'unittests src/lib.rs': 'kmipkit_ffi-',
                      'tests/c_api_coverage.rs': 'c_api_coverage-'}


def _authorized_roots() -> list[Path]:
    roots = [_REPOSITORY_ROOT, Path(tempfile.gettempdir()).resolve()]
    # An explicit Cargo target directory authorizes its build/report artifacts.
    configured = os.environ.get('CARGO_TARGET_DIR')
    if configured:
        roots.append(Path(configured).resolve())
    return roots


def _directory(path: Path, *, create=False) -> Path:
    resolved = path.resolve()
    if not any(resolved.is_relative_to(root) for root in _authorized_roots()):
        raise ValueError('coverage directory is outside authorized output roots')
    if create:
        resolved.mkdir(parents=True, exist_ok=True)
    if not resolved.is_dir():
        raise ValueError('coverage directory does not exist')
    return resolved


def prepare_report_dir(path: Path) -> Path:
    """Validate a report directory before creating any filesystem entries."""
    return _directory(path, create=True)


def _report_file(path: Path, name: str, label: str, *, report_dir=None, must_exist=True) -> Path:
    if path.name != name:
        raise ValueError(f'{label} must use its fixed coverage report filename')
    parent = _directory(path.parent)
    if report_dir is not None and parent != report_dir:
        raise ValueError(f'{label} must share the coverage report directory')
    resolved = path.resolve(strict=must_exist)
    if resolved.parent != parent or resolved.name != name:
        raise ValueError(f'{label} escapes its coverage report directory')
    if must_exist and not resolved.is_file():
        raise ValueError(f'{label} is not an existing report file')
    return resolved


def _unique_object(candidates, root, label, *, executable=False):
    objects = {}
    for candidate in sorted(candidates):
        try:
            path = candidate.resolve(strict=True)
        except OSError as error:
            raise ValueError(f'{label} is missing from the instrumented target') from error
        if not path.is_relative_to(root):
            raise ValueError(f'{label} is outside the instrumented target')
        if not path.is_file() or executable and not os.access(path, os.X_OK):
            raise ValueError(f'{label} is not a usable coverage object')
        info = path.stat()
        objects.setdefault((info.st_dev, info.st_ino), path)
    if len(objects) != 1:
        raise ValueError(f'{label} is missing or ambiguous in the instrumented target')
    return next(iter(objects.values()))


def collect_ffi_objects(target_root: Path, cargo_log: Path, artifacts_json: Path) -> list[Path]:
    """Keep both complete libraries and the actual unit/C-consumer executables."""
    root = _directory(target_root)
    cargo_log = _report_file(cargo_log, _CARGO_LOG_NAME, 'Cargo log')
    artifacts_json = _report_file(artifacts_json, _RECEIPT_NAME, 'Cargo receipt',
                                  report_dir=cargo_log.parent)
    libraries = {name: [] for name in _LIBRARY_NAMES}
    for line in artifacts_json.read_text(encoding='utf-8').splitlines():
        artifact = json.loads(line)
        if (artifact.get('reason') != 'compiler-artifact'
                or artifact.get('target', {}).get('name') != 'kmipkit_ffi'
                or artifact.get('profile', {}).get('test')):
            continue
        for filename in artifact.get('filenames', []):
            path = Path(filename)
            if path.name in libraries:
                libraries[path.name].append(path)
    objects = [_unique_object(paths, root, name) for name, paths in libraries.items()]
    roles = {role: [] for role in _CONSUMER_PREFIXES}
    pattern = re.compile(r'\bRunning (unittests src/lib\.rs|tests/c_api_coverage\.rs) \((.+)\)$')
    for line in cargo_log.read_text(encoding='utf-8').splitlines():
        match = pattern.search(line.strip())
        if match is not None:
            role, path = match.groups()
            if not Path(path).name.startswith(_CONSUMER_PREFIXES[role]):
                raise ValueError(f'{role} names an unexpected coverage executable')
            roles[role].append(Path(path))
    objects.extend(_unique_object(paths, root, role, executable=True)
                   for role, paths in roles.items())
    return objects


def record_ffi_artifacts(target_root: Path, output: Path) -> None:
    """Capture current artifacts for this repository's fixed Cargo manifest."""
    target_root = _directory(target_root)
    output = _report_file(output, _RECEIPT_NAME, 'Cargo receipt', must_exist=False)
    environment = {**os.environ, 'CARGO_TARGET_DIR': str(target_root)}
    coverage = subprocess.run(['cargo', 'llvm-cov', 'show-env', '--sh'],
                              cwd=_REPOSITORY_ROOT, env=environment,
                              text=True, capture_output=True, check=True)
    for line in coverage.stdout.splitlines():
        words = shlex.split(line)
        if len(words) != 2 or words[0] != 'export' or '=' not in words[1]:
            raise ValueError('cargo-llvm-cov returned an invalid environment assignment')
        name, value = words[1].split('=', 1)
        environment[name] = value
    with tempfile.TemporaryDirectory(prefix='.ffi-coverage-', dir=output.parent) as raw:
        receipt_path = Path(raw) / _RECEIPT_NAME
        with receipt_path.open('w', encoding='utf-8') as receipt:
            subprocess.run(['cargo', 'test', '--no-run', '--message-format=json',
                            '--manifest-path', str(_CARGO_MANIFEST),
                            '-p', 'kmipkit-ffi', '--all-targets', '--all-features', '--locked'],
                           cwd=_REPOSITORY_ROOT, env=environment, stdout=receipt, check=True)
        receipt_path.replace(output)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-root', type=Path)
    parser.add_argument('--report-dir', type=Path, required=True)
    parser.add_argument('--prepare-report-dir', action='store_true')
    parser.add_argument('--refresh-artifacts', action='store_true')
    args = parser.parse_args()
    try:
        if args.prepare_report_dir:
            print(prepare_report_dir(args.report_dir))
            return
        if args.target_root is None:
            parser.error('--target-root is required when discovering coverage objects')
        report_dir = _directory(args.report_dir)
        receipt = report_dir / _RECEIPT_NAME
        if args.refresh_artifacts:
            record_ffi_artifacts(args.target_root, receipt)
        objects = collect_ffi_objects(args.target_root, report_dir / _CARGO_LOG_NAME, receipt)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.error(str(error))
    for path in objects:
        sys.stdout.buffer.write(os.fsencode(path) + b'\0')


if __name__ == '__main__':
    main()
