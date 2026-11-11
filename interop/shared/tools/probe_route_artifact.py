#!/usr/bin/env python3
"""Probe every declared public API route in an independently installed package.

This checks public availability, not case recipe behavior or interoperability.
Artifacts must carry provenance for the exact capability-manifest commit.
"""
from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile

from route_capabilities import ROOT, validate, validate_artifact_identity, validate_installed_symbols, validate_installed_values


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_digest(path, expected):
    if sha256(path) != expected:
        raise ValueError('installed artifact digest mismatch')


def probe(data, language, artifact, provenance, *, python=None, build_dir=None):
    validate_artifact_identity(data, provenance)
    symbols = {r['public_api_symbol'] for r in data['routes'] if r['language'] == language}
    symbols.update(r['public_api_symbol'] for r in data['execution_routes'] if r['language'] == language)
    values = [row for row in data['value_symbols'] if row['language'] == language and row['availability'] == 'named-constant']
    numeric_values = [row for row in data['value_symbols'] if row['language'] == language and row['availability'] == 'numeric-only']
    checked_values = {}
    if language == 'c':
        for name, digest in provenance['files'].items():
            verify_digest(artifact / name, digest)
        library = ctypes.CDLL(str(artifact / 'lib/libkmipkit_ffi.so'))
        found = {symbol for symbol in symbols if callable(getattr(library, symbol, None))}
        with tempfile.TemporaryDirectory(prefix='kmipkit-c-route-probe-') as directory:
            consumer = Path(directory) / 'values.c'
            assertions = [f'_Static_assert({row["public_api_symbol"]} == {row["wire_value"]}U, "named value mismatch");' for row in values]
            consumer.write_text('#include "kmipkit.h"\n' + '\n'.join(assertions) + '\n')
            subprocess.run(['cc', '-std=c11', '-fsyntax-only', '-I', str(artifact / 'include'), str(consumer)], check=True)
        checked_values = {row['public_api_symbol']: row['wire_value'] for row in values}
        digest = provenance['files']['lib/libkmipkit_ffi.so']
    elif language == 'python':
        verify_digest(artifact, provenance['artifact_sha256'])
        if python is None:
            raise ValueError('the installed wheel Python interpreter is required')
        script = '''import dataclasses,importlib,json,sys,zipfile
from pathlib import Path
from importlib.metadata import distribution
info=json.loads(distribution('kmipkit').read_text('direct_url.json'))
assert info['archive_info']['hashes']['sha256']==sys.argv[1], 'installed wheel identity mismatch'
with zipfile.ZipFile(sys.argv[2]) as wheel:
    for name in wheel.namelist():
        if name.startswith('kmipkit/') and not name.endswith('/'):
            assert distribution('kmipkit').locate_file(name).read_bytes()==wheel.read(name), 'installed wheel file mismatch'
inputs=json.load(sys.stdin)
found=[]
observed_values={}
for symbol in inputs['symbols'] + [row['public_api_symbol'] for row in inputs['values']]:
    parts=symbol.split('.')
    for end in range(len(parts)-1,0,-1):
        try: value=importlib.import_module('.'.join(parts[:end]))
        except ModuleNotFoundError: continue
        try:
            for part in parts[end:-1]: value=getattr(value,part)
            owner=value
            attribute=parts[-1]
            value=getattr(owner,attribute)
            declared_fields=getattr(owner,'__annotations__',{})
            is_dataclass_field=dataclasses.is_dataclass(owner) and any(field.name==attribute for field in dataclasses.fields(owner))
            if callable(value) or isinstance(value,property) or attribute in declared_fields or is_dataclass_field: found.append(symbol)
            elif isinstance(value,int): observed_values[symbol]=value
        except AttributeError: pass
        break
print(json.dumps({'symbols':found, 'values':observed_values}))
'''
        result = subprocess.run([str(python), '-I', '-c', script, provenance['artifact_sha256'], str(artifact)],
            input=json.dumps({'symbols': sorted(symbols), 'values': values}), capture_output=True, text=True, check=True)
        observations = json.loads(result.stdout)
        found = set(observations['symbols'])
        checked_values = observations['values']
        digest = provenance['artifact_sha256']
    elif language == 'java':
        verify_digest(artifact, provenance['artifact_sha256'])
        classes = {symbol.rsplit('.', 1)[0] for symbol in symbols}
        classes.update(row['public_api_symbol'].rsplit('.', 1)[0] for row in values)
        found = set()

        def binary_name(class_name):
            parts = class_name.split('.')
            class_start = next((index for index, part in enumerate(parts)
                                if part[:1].isupper() or part == 'KmipValues'), len(parts) - 1)
            return '.'.join(parts[:class_start] + ['$'.join(parts[class_start:])])

        def public_hierarchy(class_name, seen=None):
            seen = set() if seen is None else seen
            runtime_class = binary_name(class_name)
            if runtime_class in seen:
                return ''
            seen.add(runtime_class)
            output = subprocess.check_output(['javap', '-public', '-constants', '-classpath', str(artifact), runtime_class], text=True)
            parent = re.search(r'\bextends\s+([\w.$]+)', output)
            if parent and parent[1] != 'java.lang.Object':
                output += '\n' + public_hierarchy(parent[1].replace('$', '.'), seen)
            return output

        for class_name in sorted(classes):
            public = public_hierarchy(class_name)
            found.update(symbol for symbol in symbols if symbol.rsplit('.', 1)[0] == class_name
                and re.search(r'\b' + re.escape(symbol.rsplit('.', 1)[1]) + r'\(', public))
            for row in values:
                if row['public_api_symbol'].rsplit('.', 1)[0] == class_name:
                    field = row['public_api_symbol'].rsplit('.', 1)[1]
                    match = re.search(r'\b' + re.escape(field) + r' = (\d+)[lL]?;', public)
                    if match:
                        checked_values[row['public_api_symbol']] = int(match[1])
        digest = provenance['artifact_sha256']
    else:
        # The installer records the distribution-bundle digest. Rust imports are
        # compiled against extracted .crate packages, never workspace sources.
        bundle = artifact.parent / 'kmipkit-rust-1.0.0.tar.gz'
        verify_digest(bundle, provenance['artifact_sha256'])
        if (artifact / 'library-commit').read_text().strip() != data['library_commit']:
            raise ValueError('installed Rust artifact commit mismatch')
        with tarfile.open(bundle) as distribution:
            for member in distribution.getmembers():
                if member.isfile() and (artifact / member.name).read_bytes() != distribution.extractfile(member).read():
                    raise ValueError('installed Rust distribution file mismatch')
        for package in sorted((artifact / 'packages').glob('*.crate')):
            with tarfile.open(package) as archive:
                for member in archive.getmembers():
                    if member.isfile():
                        installed_file = artifact / 'crates' / member.name
                        if installed_file.read_bytes() != archive.extractfile(member).read():
                            raise ValueError('installed Rust package file mismatch')
        if build_dir is None:
            raise ValueError('Rust consumer build directory is required')
        build_dir.mkdir(parents=True, exist_ok=True)
        (build_dir / 'src').mkdir(exist_ok=True)
        packages = ('kmipkit', 'kmipkit-ttlv', 'kmipkit-protocol', 'kmipkit-client', 'kmipkit-transport')
        manifest = '[package]\nname="kmipkit-route-probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\n'
        manifest += '\n'.join(f'{name} = {{ path = "{artifact}/crates/{name}-1.0.0" }}' for name in packages)
        manifest += '\n[patch.crates-io]\n'
        manifest += '\n'.join(f'{name} = {{ path = "{artifact}/crates/{name}-1.0.0" }}' for name in packages)
        manifest += f'\naws-lc-sys = {{ path = "{artifact}/vendor/aws-lc-sys-0.45.0" }}\n'
        (build_dir / 'Cargo.toml').write_text(manifest)
        body = []
        for symbol in sorted(symbols):
            if symbol == 'kmipkit_protocol::AttributeReference::name':
                body.append('let _ = || kmipkit_protocol::AttributeReference::name(String::new(), String::new());')
            elif symbol == 'kmipkit_protocol::QueryAsyncRequestsRequest::with_operations':
                body.append('let _ = kmipkit_protocol::QueryAsyncRequestsRequest::with_operations::<Vec<u32>>;')
            elif symbol == 'kmipkit_protocol::QueryAsyncRequestsRequest::with_correlation_values':
                body.append('let _ = kmipkit_protocol::QueryAsyncRequestsRequest::with_correlation_values::<Vec<Vec<u8>>>;')
            elif symbol == 'kmipkit_protocol::AttributeSet::try_new':
                body.append('let _ = || kmipkit_protocol::AttributeSet::try_new(Vec::<kmipkit_ttlv::Item>::new());')
            elif symbol == 'kmipkit_protocol::QueryRequest::new':
                body.append('let _ = || kmipkit_protocol::QueryRequest::new(Vec::<kmipkit_protocol::QueryFunction>::new());')
            elif symbol == 'kmipkit_protocol::GetAttributesRequest::try_new':
                body.append('let _ = || kmipkit_protocol::GetAttributesRequest::try_new(None, Vec::<kmipkit_protocol::AttributeReference>::new());')
            elif symbol.endswith('::with_ttlv'):
                owner = symbol.removesuffix('::with_ttlv')
                body.append(f'let _ = |value: &{owner}| value.with_ttlv(|_| ());')
            elif symbol.endswith('::with_value'):
                owner = symbol.removesuffix('::with_value')
                body.append(f'let _ = |value: &{owner}| value.with_value(|_| ());')
            else:
                body.append('let _ = ' + symbol + ';')
        body.extend(f'const _: () = assert!({row["public_api_symbol"]} == {row["wire_value"]});' for row in values)
        (build_dir / 'src/main.rs').write_text('fn main() {\n' + '\n'.join(body) + '\n}\n')
        subprocess.run(['cargo', '+1.94', 'check', '--manifest-path', str(build_dir / 'Cargo.toml')], check=True)
        found, digest = symbols, provenance['artifact_sha256']
        checked_values = {row['public_api_symbol']: row['wire_value'] for row in values}
    validate_installed_symbols(data, language, found)
    validate_installed_values(data, language, checked_values)
    return {'language': language, 'library_commit': data['library_commit'],
            'artifact_sha256': digest, 'public_symbols_checked': len(found), 'named_values_checked': len(checked_values), 'numeric_values_supported': len(numeric_values),
            'scope': 'public-api-symbol-availability', 'status': 'verified'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--language', choices=('rust', 'c', 'java', 'python'), required=True)
    parser.add_argument('--artifact', type=Path, required=True)
    parser.add_argument('--provenance', type=Path, required=True)
    parser.add_argument('--python', type=Path)
    parser.add_argument('--build-dir', type=Path)
    args = parser.parse_args()
    data = validate(json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text()))
    print(json.dumps(probe(data, args.language, args.artifact.resolve(),
        json.loads(args.provenance.read_text()), python=args.python, build_dir=args.build_dir), sort_keys=True))


if __name__ == '__main__':
    main()
