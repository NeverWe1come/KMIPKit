"""Command-Line Interface for KMIP 2.1 Mock KMS Simulator."""

from __future__ import annotations

import argparse
import json
import signal
import sys
import time
from pathlib import Path

from .dispatcher import MockKmsDispatcher
from .server import MockKmsServer


def main() -> None:
    parser = argparse.ArgumentParser(description="KMIP 2.1 Multi-Profile Mock KMS Server")
    parser.add_argument("--profile", type=Path, default=Path("interop/shared/profiles/generic-kmip-2.1.json"), help="KMS profile JSON")
    parser.add_argument("--host", type=str, default="127.0.0.1", help="Host interface to bind (default: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=5696, help="Port to bind (default: 5696, use 0 for ephemeral)")
    parser.add_argument("--output-connection", type=Path, help="Path to write server-connection.json")
    parser.add_argument("--timeout", type=float, help="Optional duration in seconds before self-termination")

    args = parser.parse_args()

    profile_p = Path(args.profile).resolve()
    if not profile_p.exists():
        print(f"Error: Profile file not found: {profile_p}", file=sys.stderr)
        sys.exit(1)

    dispatcher = MockKmsDispatcher.from_profile_file(profile_p)
    server = MockKmsServer(dispatcher=dispatcher, host=args.host, port=args.port)

    server.start()
    conn_info = server.get_connection_config()

    print("\n=======================================================")
    print("🛡️  KMIPKit Mock KMS Server Active")
    print("=======================================================")
    print(f"Server ID: {conn_info['server_identifier']}")
    print(f"Endpoint:  {conn_info['host']}:{conn_info['port']} ({conn_info['transport']})")
    print(f"Profile:   {profile_p.name}")
    print("=======================================================\n")

    if args.output_connection:
        out_p = Path(args.output_connection).resolve()
        out_p.parent.mkdir(parents=True, exist_ok=True)
        with open(out_p, "w", encoding="utf-8") as f:  # NOSONAR
            json.dump(conn_info, f, indent=2)
            f.write("\n")
        print(f"Wrote connection configuration to: {out_p}")

    def _shutdown_handler(sig, frame):
        print("\nShutting down Mock KMS Server...")
        server.stop()
        sys.exit(0)

    signal.signal(signal.SIGINT, _shutdown_handler)
    signal.signal(signal.SIGTERM, _shutdown_handler)

    if args.timeout:
        time.sleep(args.timeout)
        server.stop()
    else:
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            _shutdown_handler(None, None)


if __name__ == "__main__":
    main()
