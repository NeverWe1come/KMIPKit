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

_LIBRARY_NAMES = ('libkmipkit_ffi.so', 'libkmipkit_ffi.a')
_CONSUMER_PREFIXES = {'unittests src/lib.rs': 'kmipkit_ffi-',
                      'tests/c_api_coverage.rs': 'c_api_coverage-'}


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
    root = target_root.resolve(strict=True)
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
            expected = _CONSUMER_PREFIXES[role]
            if not Path(path).name.startswith(expected):
                raise ValueError(f'{role} names an unexpected coverage executable')
            roles[role].append(Path(path))
    objects.extend(_unique_object(paths, root, role, executable=True)
                   for role, paths in roles.items())
    return objects


def record_ffi_artifacts(target_root: Path, manifest_path: Path, output: Path) -> None:
    """Ask Cargo for current artifacts using cargo-llvm-cov's exact environment."""
    environment = {**os.environ, 'CARGO_TARGET_DIR': str(target_root.resolve())}
    coverage = subprocess.run(['cargo', 'llvm-cov', 'show-env', '--sh'],
                              cwd=manifest_path.parent, env=environment,
                              text=True, capture_output=True, check=True)
    for line in coverage.stdout.splitlines():
        words = shlex.split(line)
        if len(words) != 2 or words[0] != 'export' or '=' not in words[1]:
            raise ValueError('cargo-llvm-cov returned an invalid environment assignment')
        name, value = words[1].split('=', 1)
        environment[name] = value
    with output.open('w', encoding='utf-8') as receipt:
        subprocess.run(['cargo', 'test', '--no-run', '--message-format=json',
                        '--manifest-path', str(manifest_path), '-p', 'kmipkit-ffi',
                        '--all-targets', '--all-features', '--locked'],
                       cwd=manifest_path.parent, env=environment, stdout=receipt, check=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-root', type=Path, required=True)
    parser.add_argument('--cargo-log', type=Path, required=True)
    parser.add_argument('--artifacts-json', type=Path, required=True)
    parser.add_argument('--refresh-artifacts', action='store_true')
    parser.add_argument('--manifest-path', type=Path,
                        default=Path(__file__).resolve().parents[1] / 'Cargo.toml')
    args = parser.parse_args()
    try:
        if args.refresh_artifacts:
            record_ffi_artifacts(args.target_root, args.manifest_path, args.artifacts_json)
        objects = collect_ffi_objects(args.target_root, args.cargo_log, args.artifacts_json)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.error(str(error))
    for path in objects:
        sys.stdout.buffer.write(os.fsencode(path) + b'\0')


if __name__ == '__main__':
    main()
