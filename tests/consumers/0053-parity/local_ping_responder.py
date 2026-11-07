"""Loopback-only TLS 1.3/mTLS fixture for the four external Ping consumers."""

from __future__ import annotations

import argparse
from pathlib import Path
import socket
import ssl
import struct


def _item(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def _structure(tag: int, *children: bytes) -> bytes:
    return _item(tag, 1, b"".join(children))


def _ping_response() -> bytes:
    version = _structure(
        0x420069,
        _item(0x42006A, 2, struct.pack(">i", 2)),
        _item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _structure(
        0x42007A,
        version,
        _item(0x420092, 9, struct.pack(">q", 1)),
        _item(0x42000D, 2, struct.pack(">i", 1)),
    )
    response_item = _structure(
        0x42000F,
        _item(0x42005C, 5, struct.pack(">I", 0x3B)),
        _item(0x42007F, 5, struct.pack(">I", 0)),
        _structure(0x42007C),
    )
    return _structure(0x42007B, header, response_item)


def _read_exact(stream: ssl.SSLSocket, length: int) -> bytes:
    data = bytearray()
    while len(data) < length:
        chunk = stream.recv(length - len(data))
        if not chunk:
            raise RuntimeError("consumer closed before sending a complete TTLV item")
        data.extend(chunk)
    return bytes(data)


def _read_frame(stream: ssl.SSLSocket) -> bytes:
    header = _read_exact(stream, 8)
    body_length = int.from_bytes(header[4:8], "big")
    if body_length > 16 * 1024 * 1024 or body_length % 8:
        raise RuntimeError("consumer sent an invalid or oversized TTLV frame")
    return header + _read_exact(stream, body_length)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ready-file", type=Path, required=True)
    parser.add_argument("--fixture-dir", type=Path, required=True)
    parser.add_argument("--connections", type=int, default=4)
    args = parser.parse_args()

    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.minimum_version = ssl.TLSVersion.TLSv1_3
    context.maximum_version = ssl.TLSVersion.TLSv1_3
    context.load_cert_chain(args.fixture_dir / "server-cert.pem", args.fixture_dir / "server-key.pem")
    context.load_verify_locations(cafile=args.fixture_dir / "client-cert.pem")
    context.verify_mode = ssl.CERT_REQUIRED

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        listener.bind(("127.0.0.1", 0))
        listener.listen(args.connections)
        # Native/JNI builds run before Java connects in the consumer runner.
        listener.settimeout(180)
        args.ready_file.write_text(str(listener.getsockname()[1]), encoding="ascii")
        for connection_index in range(args.connections):
            raw, _address = listener.accept()
            raw.settimeout(10)
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                peer_cert = stream.getpeercert(binary_form=True)
                if stream.version() != "TLSv1.3" or not peer_cert:
                    raise RuntimeError("fixture requires an authenticated TLS 1.3 client")
                request = _read_frame(stream)
                ping_operation = bytes.fromhex("42005c05000000040000003b")
                if request[:4] != bytes.fromhex("42007801") or ping_operation not in request:
                    raise RuntimeError("fixture accepts only a KMIP Ping request")
                stream.sendall(_ping_response())


if __name__ == "__main__":
    main()
