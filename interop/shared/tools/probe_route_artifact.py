#!/usr/bin/env python3
"""Probe every declared constructor in an independently installed package.

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
import sys

from route_capabilities import ROOT, validate, validate_artifact_identity, validate_installed_symbols


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_digest(path, expected):
    if sha256(path) != expected:
        raise ValueError('installed artifact digest mismatch')


def probe(data, language, artifact, provenance, *, python=None, build_dir=None):
    validate_artifact_identity(data, provenance)
    symbols = {r['public_api_symbol'] for r in data['routes'] if r['language'] == language}
    if language == 'c':
        for name, digest in provenance['files'].items():
            verify_digest(artifact / name, digest)
        library = ctypes.CDLL(str(artifact / 'lib/libkmipkit_ffi.so'))
        found = {symbol for symbol in symbols if callable(getattr(library, symbol, None))}
        digest = provenance['files']['lib/libkmipkit_ffi.so']
    elif language == 'python':
        verify_digest(artifact, provenance['artifact_sha256'])
        if python is None:
            raise ValueError('the installed wheel Python interpreter is required')
        script = '''import importlib,json,sys
from pathlib import Path
from importlib.metadata import distribution
info=json.loads(distribution('kmipkit').read_text('direct_url.json'))
assert info['archive_info']['hashes']['sha256']==sys.argv[1], 'installed wheel identity mismatch'
found=[]
for symbol in json.load(sys.stdin):
    parts=symbol.split('.')
    for end in range(len(parts)-1,0,-1):
        try: value=importlib.import_module('.'.join(parts[:end]))
        except ModuleNotFoundError: continue
        try:
            for part in parts[end:]: value=getattr(value,part)
            if callable(value): found.append(symbol)
        except AttributeError: pass
        break
print(json.dumps(found))
'''
        result = subprocess.run([str(python), '-I', '-c', script, provenance['artifact_sha256']],
            input=json.dumps(sorted(symbols)), capture_output=True, text=True, check=True)
        found = set(json.loads(result.stdout))
        digest = provenance['artifact_sha256']
    elif language == 'java':
        verify_digest(artifact, provenance['artifact_sha256'])
        classes = {symbol.rsplit('.', 1)[0] for symbol in symbols}
        found = set()
        for class_name in sorted(classes):
            public = subprocess.check_output(['javap', '-public', '-classpath', str(artifact), class_name], text=True)
            found.update(symbol for symbol in symbols if symbol.rsplit('.', 1)[0] == class_name
                and re.search(r'\b' + re.escape(symbol.rsplit('.', 1)[1]) + r'\(', public))
        digest = provenance['artifact_sha256']
    else:
        # The installer records the distribution-bundle digest. Rust imports are
        # compiled against extracted .crate packages, never workspace sources.
        bundle = artifact.parent / 'kmipkit-rust-1.0.0.tar.gz'
        verify_digest(bundle, provenance['artifact_sha256'])
        if (artifact / 'library-commit').read_text().strip() != data['library_commit']:
            raise ValueError('installed Rust artifact commit mismatch')
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
            if symbol == 'kmipkit_protocol::QueryRequest::new':
                body.append('let _ = || kmipkit_protocol::QueryRequest::new(Vec::<kmipkit_protocol::QueryFunction>::new());')
            elif symbol == 'kmipkit_protocol::GetAttributesRequest::try_new':
                body.append('let _ = || kmipkit_protocol::GetAttributesRequest::try_new(None, Vec::<kmipkit_protocol::AttributeReference>::new());')
            else:
                body.append('let _ = ' + symbol + ';')
        (build_dir / 'src/main.rs').write_text('fn main() {\n' + '\n'.join(body) + '\n}\n')
        subprocess.run(['cargo', '+1.94', 'check', '--manifest-path', str(build_dir / 'Cargo.toml')], check=True)
        found, digest = symbols, provenance['artifact_sha256']
    validate_installed_symbols(data, language, found)
    return {'language': language, 'library_commit': data['library_commit'],
            'artifact_sha256': digest, 'public_symbols_checked': len(found),
            'scope': 'public-constructor-availability', 'status': 'verified'}


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
