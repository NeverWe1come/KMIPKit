#!/usr/bin/env python3
"""Generate link-time C ABI call observations from the exact installed header.

Only calls made by this independent C consumer are wrapped. The shared KMIPKit
library and its sources are unchanged. Observations contain only public symbol
names and numeric status codes, and are written only when explicitly enabled.
"""
import argparse
import json
from pathlib import Path
import re

from safe_paths import resolve_within_root

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_ROOT = ROOT / 'interop/shared/coverage'
OUTPUT_ROOT = ROOT / 'interop/c-kmip-validator/build'


def resolve_manifest_path(path: Path) -> Path:
    """Accept only the checked-in route manifest tree as a generator input."""
    return resolve_within_root(path, MANIFEST_ROOT)


def resolve_output_path(path: Path) -> Path:
    """Keep generated wrapper code inside the C validator build directory."""
    return resolve_within_root(path, OUTPUT_ROOT)


def parameter_name(parameter: str) -> str:
    """Return a simple C parameter identifier without regex backtracking."""
    declaration = parameter.split('[', 1)[0].replace('*', ' ').strip()
    tokens = declaration.split()
    name = tokens[-1] if tokens else ''
    if not name.isidentifier():
        raise ValueError('unsupported_installed_signature')
    return name


def generate(header: str, symbols: list[str]) -> str:
    result = ['/* Generated from installed kmipkit.h; do not edit. */',
              '#include "kmipkit.h"', '#include <stdio.h>', '#include <stdlib.h>',
              'static void observe(const char *symbol, int32_t status) {',
              '  const char *path = getenv("KMIPKIT_C_ROUTE_OBSERVATIONS");',
              '  if (!path || !*path) return;',
              '  FILE *output = fopen(path, "ab");',
              '  if (!output) return;',
              '  fprintf(output, "%s\\t%d\\n", symbol, (int)status);',
              '  fclose(output);', '}']
    for symbol in sorted(set(symbols)):
        match = re.search(r'int32_t\s+' + re.escape(symbol) + r'\(([^;]*)\);', header)
        if not match:
            raise ValueError(f'installed_header_signature_missing:{symbol}')
        arguments = match[1]
        parameters = [] if arguments.strip() in ('', 'void') else arguments.split(',')
        names = []
        for parameter in parameters:
            try:
                name = parameter_name(parameter)
            except ValueError:
                raise ValueError(f'unsupported_installed_signature:{symbol}')
            names.append(name)
        result.extend([f'int32_t __real_{symbol}({arguments});',
                       f'int32_t __wrap_{symbol}({arguments}) {{',
                       f'  int32_t status = __real_{symbol}({", ".join(names)});',
                       f'  observe("{symbol}", status);', '  return status;', '}'])
    return '\n'.join(result) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--header', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    manifest_path = resolve_manifest_path(args.manifest)
    output_path = resolve_output_path(args.output)
    manifest = json.loads(manifest_path.read_text())
    symbols = [r['public_api_symbol'] for r in manifest['routes'] + manifest['execution_routes'] if r['language'] == 'c']
    output_path.write_text(generate(args.header.read_text(), symbols))


if __name__ == '__main__':
    main()
