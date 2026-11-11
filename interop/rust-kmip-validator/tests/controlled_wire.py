"""Independent controlled-peer KMIP Profile codecs; bodies stay in memory.

OASIS KMIP Profiles 2.1 §§5.3–5.5; normative names are read from the pinned
inventory. This fixture codec never imports an implementation codec.
"""
from datetime import datetime, timezone
import json
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[3]
ELEMENTS = json.loads((ROOT / 'specification/catalog/kmip-2.1.json').read_bytes())['elements']
TYPES = ('', 'Structure', 'Integer', 'LongInteger', 'BigInteger', 'Enumeration', 'Boolean', 'TextString', 'ByteString', 'DateTime', 'Interval', 'DateTimeExtended')
CONTENT_TYPES = {'TTLV': 'application/octet-stream', 'JSON': 'application/json', 'XML': 'text/xml'}
LIMIT = 16 * 1024 * 1024


def name_key(value):
    value = ''.join(ch.casefold() for ch in value if ch.isalnum())
    digits = len(value) - len(value.lstrip('0123456789'))
    return value[digits:] + value[:digits]


TAGS = {name_key(e['name']): int(e['wire_value'], 16) for e in ELEMENTS
        if e['kind'] == 'tag' and e.get('allocation') == 'assigned'}
FAMILIES = {e['element_id']: e['name'] for e in ELEMENTS if e['kind'] in {'enumeration', 'bitmask'}}
VALUES = {}
for element in ELEMENTS:
    if element['kind'] in {'enumeration_value', 'bitmask_value'} and element.get('allocation') == 'assigned':
        for parent in element['parent_element_ids']:
            if parent in FAMILIES and name_key(FAMILIES[parent]) in TAGS:
                tag = TAGS[name_key(FAMILIES[parent])]
                VALUES[tag, name_key(element['name'])] = int(element['wire_value'], 16)


def raw_tag(value):
    return int(value, 16) if value.startswith('0x') else TAGS[name_key(value)]


def scalar(tag, kind, value):
    if kind in (4, 8):
        return bytes.fromhex(value.removeprefix('0x'))
    if kind == 7:
        return value
    if kind == 6:
        return int(value if isinstance(value, bool) else value == 'true')
    if isinstance(value, str) and value.startswith('0x'):
        result = int(value, 16)
        bits = 32 if kind == 2 else 64
        return result - 2 ** bits if kind in (2, 3, 9, 11) and result >= 2 ** (bits - 1) else result
    if kind == 9 and isinstance(value, str):
        return int(datetime.fromisoformat(value.replace('Z', '+00:00')).timestamp())
    if isinstance(value, str) and not value.lstrip('-').isdecimal():
        if kind == 2 and '|' in value:
            result = 0
            for bit in value.split('|'):
                result |= VALUES[tag, name_key(bit)]
            return result
        return VALUES[tag, name_key(value)]
    return int(value)


def decode(body, encoding):
    if encoding == 'JSON':
        def item(node):
            tag = raw_tag(node['tag'])
            kind = TYPES.index(node.get('type', 'Structure'))
            value = [item(v) for v in node['value']] if kind == 1 else scalar(tag, kind, node['value'])
            return tag, kind, value
        return [item(json.loads(body))]
    def item(node):
        tag = raw_tag(node.attrib['tag'] if node.tag.split('}')[-1] == 'TTLV' else node.tag.split('}')[-1])
        kind = TYPES.index(node.get('type', 'Structure'))
        value = [item(v) for v in node] if kind == 1 else scalar(tag, kind, node.attrib['value'])
        return tag, kind, value
    return [item(ET.fromstring(body))]


def encoded(tree, encoding):
    def value(kind, value):
        if kind in (4, 8):
            return value.hex()
        if kind == 5:
            return f'0x{value:08x}'
        if kind == 6:
            return bool(value) if encoding == 'JSON' else str(bool(value)).lower()
        if kind == 9:
            return datetime.fromtimestamp(value, timezone.utc).isoformat().replace('+00:00', 'Z')
        return value
    if encoding == 'JSON':
        def item(node):
            tag, kind, data = node
            return {'tag': f'0x{tag:06x}', 'type': TYPES[kind],
                    'value': [item(v) for v in data] if kind == 1 else value(kind, data)}
        return json.dumps(item(tree[0]), separators=(',', ':')).encode()
    def item(node):
        tag, kind, data = node
        node = ET.Element('TTLV', {'tag': f'0x{tag:06x}', 'type': TYPES[kind]})
        if kind == 1:
            node.extend(item(v) for v in data)
        else:
            node.set('value', str(value(kind, data)))
        return node
    return ET.tostring(item(tree[0]), encoding='utf-8')


def receive(channel, encoding, transport, ttlv_receive):
    if transport == 'https':
        header = bytearray()
        while not header.endswith(b'\r\n\r\n'):
            part = channel.recv(1)
            if not part:
                return None
            header.extend(part)
            if len(header) > 8192:
                raise AssertionError('controlled HTTP headers exceed bound')
        lines = header.decode('ascii').split('\r\n')
        if lines[0] != 'POST /kmip HTTP/1.1':
            raise AssertionError('controlled HTTP request target changed')
        fields = dict(line.split(': ', 1) for line in lines[1:] if ': ' in line)
        size = int(next(value for name, value in fields.items() if name.lower() == 'content-length'))
        content_type = next(value for name, value in fields.items() if name.lower() == 'content-type')
        if content_type != CONTENT_TYPES[encoding] or not 0 < size <= LIMIT:
            raise AssertionError('controlled HTTP framing mismatch')
        body = bytearray()
        while len(body) < size:
            part = channel.recv(min(size - len(body), 65536))
            if not part:
                raise AssertionError('controlled HTTP body truncated')
            body.extend(part)
        return body
    if encoding == 'TTLV':
        return ttlv_receive(channel)
    body = bytearray()
    while len(body) < LIMIT:
        part = channel.recv(min(65536, LIMIT - len(body)))
        if not part:
            return None if not body else (_ for _ in ()).throw(AssertionError('controlled document truncated'))
        body.extend(part)
        try:
            json.loads(body) if encoding == 'JSON' else ET.fromstring(body)
            return body
        except (ValueError, ET.ParseError):
            continue
    raise AssertionError('controlled document exceeds bound')


def send(channel, body, encoding, transport):
    if transport == 'https':
        header = f'HTTP/1.1 200 OK\r\nContent-Type: {CONTENT_TYPES[encoding]}\r\nContent-Length: {len(body)}\r\nConnection: close\r\n\r\n'
        channel.sendall(header.encode('ascii') + body)
    else:
        channel.sendall(body)
