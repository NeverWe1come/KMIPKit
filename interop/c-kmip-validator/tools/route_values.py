"""Execute named enum/mask cases without substituting another KMIP type."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re

from artifact_support import ROOT
from route_ttlv import TreeAbi, digest, encode


def norm(value):
    return re.sub('[^a-z0-9]', '', value.lower())


class ValueCases:
    def __init__(self, library: Path, catalog):
        self.abi = TreeAbi(library)
        self.catalog = catalog
        self.tags = {norm(target['name']): int(target['wire_value'], 16)
                     for target in catalog['normative_targets'] if target['kind'] == 'tag'}

    def execute(self, case, version):
        requires = case['requires']
        type_name = requires.get('enum_type') or requires.get('bitmask_type')
        if not type_name:
            return None
        if type_name == 'Item Type':
            vectors = json.loads((ROOT / 'interop/shared/vectors/ttlv-boundaries.json').read_text())
            results, symbols, count = [], set(), 0
            for vector in vectors['vectors']:
                result = self.abi.boundary_case(vector)
                results.append([vector['type_code'], result['tree_sha256']])
                symbols.update(result['invoked_symbols'])
                count += result['sample_count']
            return {'tree_sha256': digest(results), 'invoked_symbols': symbols,
                    'sample_count': count, 'known_values_checked': 11,
                    'unknown_value_preserved': False, 'tag': None}
        tag = self.tags.get(norm(type_name))
        if tag is None:
            return None
        mask = 'bitmask_type' in requires
        parents = ('BITMASK' if mask else 'ENUMERATION')
        known = []
        for target in self.catalog['normative_targets']:
            kinds = {'bitmask_value'} if mask else {'enumeration_value'}
            if type_name == 'Tag':
                kinds.add('tag')
            if type_name in ('PKCS#11 Function', 'PKCS#11 Return Code'):
                kinds.add('referenced_enum_value')
            if target['kind'] not in kinds:
                continue
            if version not in target['protocol_versions']:
                continue
            if not (target.get('parent_name') == type_name or any(norm(re.sub('^KMIPKIT-ELEM-' + parents + '-', '', parent)) == norm(type_name)
                       for parent in target.get('parent_element_ids', []))):
                continue
            raw = target.get('wire_value', '')
            if re.fullmatch('[0-9a-fA-F]{1,8}', raw):
                known.append(int(raw, 16))
            elif re.fullmatch('[0-9a-fA-F]{8} - [0-9a-fA-F]{8}', raw):
                known.extend(int(boundary, 16) for boundary in raw.split(' - '))
        known = sorted(set(known))
        if not known:
            return None
        specification = requires.get('exhaustive_bitmask_combinations')
        if specification:
            flags = [int(value, 16) for value in specification['flag_wire_values']]
            count = 1 << len(flags)
            if specification['combination_count'] != count:
                raise ValueError('catalog_mask_combination_count_mismatch')
            values = (sum(flag for index, flag in enumerate(flags) if pattern & (1 << index))
                      for pattern in range(count))
        else:
            count = 0
            values = iter(known)
        state, symbols, sample_count = hashlib.sha256(), set(), 0
        for value in values:
            result = self.abi.roundtrip(encode((tag, 2 if mask else 5, value.to_bytes(4, 'big'))))
            state.update(json.dumps(result['normalized_tree'], separators=(',', ':')).encode())
            state.update(b'\n')
            symbols.update(result['invoked_symbols'])
            sample_count += 1
        if mask:
            all_known = 0
            for flag in known:
                all_known |= flag
            unknown = next((1 << bit for bit in reversed(range(32)) if not all_known & (1 << bit)), None)
            if unknown is None:
                raise ValueError('no_unknown_bit_available_for_mask')
            unknown_values = [unknown, unknown | all_known]
        else:
            unknown_values = [next(value for value in (0xFFFFFFFF, 0xF1234567, 0x80000000) if value not in known)]
        for value in unknown_values:
            result = self.abi.roundtrip(encode((tag, 2 if mask else 5, value.to_bytes(4, 'big'))))
            state.update(json.dumps(result['normalized_tree'], separators=(',', ':')).encode())
            state.update(b'\n')
            symbols.update(result['invoked_symbols'])
            sample_count += 1
        return {'tree_sha256': state.hexdigest(), 'invoked_symbols': symbols, 'sample_count': sample_count,
                'known_values_checked': len(known), 'unknown_value_preserved': True,
                'combination_count': count, 'tag': tag}
