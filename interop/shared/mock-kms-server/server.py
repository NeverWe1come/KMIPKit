"""Lightweight Socket Server for KMIP 2.1 Mock KMS Simulator."""

from __future__ import annotations

import json
import socket
import socketserver
import ssl
import threading
from pathlib import Path
from typing import Any

from .dispatcher import MockKmsDispatcher


class _KmsTcpHandler(socketserver.StreamRequestHandler):
    def handle(self) -> None:
        dispatcher: MockKmsDispatcher = self.server.dispatcher  # type: ignore
        for line in self.rfile:
            line_str = line.decode("utf-8").strip()
            if not line_str:
                continue
            try:
                request = json.loads(line_str)
                response = dispatcher.dispatch(request)
            except Exception as e:
                response = {
                    "result_status": "OperationFailed",
                    "result_reason": "InvalidMessage",
                    "message": f"Malformed request: {e}",
                }
            resp_bytes = json.dumps(response, ensure_ascii=False).encode("utf-8") + b"\n"
            self.wfile.write(resp_bytes)
            self.wfile.flush()


class MockKmsServer:
    """Threaded TCP server simulating a KMIP 2.1 KMS endpoint."""

    def __init__(
        self,
        dispatcher: MockKmsDispatcher | None = None,
        profile_path: Path | None = None,
        host: str = "127.0.0.1",
        port: int = 0,
        tls: bool = False,
        certfile: str | None = None,
        keyfile: str | None = None,
        ca_cert: str | None = None,
    ) -> None:
        if profile_path:
            self.dispatcher = MockKmsDispatcher.from_profile_file(profile_path)
        else:
            self.dispatcher = dispatcher or MockKmsDispatcher()

        self.host = host
        self._requested_port = port
        self.tls = tls
        self.certfile = certfile
        self.keyfile = keyfile
        self.ca_cert = ca_cert

        self._server: socketserver.ThreadingTCPServer | None = None
        self._thread: threading.Thread | None = None
        self.port: int = port

    def start(self) -> None:
        """Start the server in a background daemon thread."""
        server = socketserver.ThreadingTCPServer(
            (self.host, self._requested_port),
            _KmsTcpHandler,
            bind_and_activate=True,
        )
        server.dispatcher = self.dispatcher  # type: ignore

        if self.tls:
            context = ssl.create_default_context(ssl.Purpose.CLIENT_AUTH)
            context.minimum_version = ssl.TLSVersion.TLSv1_3  # Enforce TLS 1.3 only
            if self.certfile and self.keyfile:
                context.load_cert_chain(certfile=self.certfile, keyfile=self.keyfile)
            if self.ca_cert:
                context.load_verify_locations(cafile=self.ca_cert)
                context.verify_mode = ssl.CERT_REQUIRED
            server.socket = context.wrap_socket(server.socket, server_side=True)  # NOSONAR

        self.port = server.server_address[1]
        self._server = server
        self._thread = threading.Thread(target=server.serve_forever, daemon=True)
        self._thread.start()

    def stop(self) -> None:
        """Shutdown the server and release sockets."""
        if self._server:
            self._server.shutdown()
            self._server.server_close()
            self._server = None
        if self._thread:
            self._thread.join(timeout=2.0)
            self._thread = None

    def __enter__(self) -> MockKmsServer:
        self.start()
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        self.stop()

    def get_connection_config(self) -> dict[str, Any]:
        """Generate a connection configuration dictionary to target this server."""
        return {
            "server_identifier": self.dispatcher.profile.get("server", {}).get("id", "mock-kms"),
            "host": self.host,
            "port": self.port,
            "transport": "raw_tls" if self.tls else "plain_tcp",
            "encoding": "JSON",
        }
