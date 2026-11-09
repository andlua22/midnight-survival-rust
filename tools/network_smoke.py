#!/usr/bin/env python3
"""Exercise the real server and load binaries over loopback UDP (Linux).

This checks both load scenarios and clean shutdown. It is not an Internet
capacity benchmark: the server and all clients share this machine.
"""

import argparse
from datetime import datetime, timezone
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tempfile
import time


def stop_server(process):
    if process.poll() is None:
        process.send_signal(signal.SIGINT)
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
            raise RuntimeError("server did not shut down within 10 seconds")


def ready_address(process, log):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"server exited before readiness; see {log}")
        match = re.search(r"Midnight server: (127\.0\.0\.1:\d+)", log.read_text())
        if match:
            return match.group(1)
        time.sleep(0.05)
    raise RuntimeError(f"server readiness timed out; see {log}")


def scenario(args, name, directory, identity, invitation):
    server_report = directory / f"{name}-server.json"
    load_report = directory / f"{name}-load.json"
    log = directory / f"{name}-server.log"
    with log.open("w") as output:
        process = subprocess.Popen(
            [str(args.bin_dir / "server"), "serve", "--listen", "127.0.0.1:0",
             "--data-dir", str(identity), "--max-players", str(args.clients),
             "--map", "load-arena" if name == "spread" else "polygon",
             "--layout", name, "--report", str(server_report)],
            stdout=output, stderr=subprocess.STDOUT,
        )
        try:
            address = ready_address(process, log)
            subprocess.run(
                [str(args.bin_dir / "net_load"), "--server", address,
                 "--code-file", str(invitation), "--clients", str(args.clients),
                 "--seconds", str(args.seconds), "--scenario", name,
                 "--report", str(load_report)],
                check=True, timeout=args.seconds + 60,
            )
            # Let the server process the clients' explicit CONNECTION_CLOSE.
            time.sleep(0.5)
        finally:
            stop_server(process)
    if process.returncode != 0:
        raise RuntimeError(f"server failed with status {process.returncode}; see {log}")
    server = json.loads(server_report.read_text())
    load = json.loads(load_report.read_text())
    if server["peak_players"] != args.clients or load["clients"] != args.clients:
        raise RuntimeError(f"{name}: not all requested clients were admitted")
    if server["players"] != 0:
        raise RuntimeError(f"{name}: disconnected clients were not removed")
    if server["seconds_at_capacity"] < args.seconds * 0.9:
        raise RuntimeError(f"{name}: requested population was not sustained")
    if any(bot["snapshots"] == 0 or bot["input_packets"] == 0 for bot in load["bots"]):
        raise RuntimeError(f"{name}: a client did not exchange game state")
    if name == "crowd" and any(bot["peak_visible"] != args.clients - 1 for bot in load["bots"]):
        raise RuntimeError("crowd: AOI did not include every nearby player")
    if name == "spread" and args.clients > 4 and any(
        bot["peak_visible"] >= args.clients - 1 for bot in load["bots"]
    ):
        raise RuntimeError("spread: AOI did not filter distant players")
    minimum_snapshot_hz = min(bot["owner_snapshot_hz"] for bot in load["bots"])
    print(
        f"{name}: {server['peak_players']} players, "
        f"{server['seconds_at_capacity']:.1f}s at capacity, "
        f"tick p99={server['tick_p99_ms_at_capacity']:.2f}ms, "
        f"rate at capacity={server['ticks_per_second_at_capacity']:.2f}Hz, "
        f"physics p99={server['physics_p99_ms_at_capacity']:.2f}ms, "
        f"replication p99={server['replication_p99_ms_at_capacity']:.2f}ms, "
        f"min received snapshots={minimum_snapshot_hz:.2f}Hz, "
        f"RSS={server['current_rss_mb']:.1f}MiB, cleanup OK",
        flush=True,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bin-dir", type=Path, default=Path("target/release"))
    parser.add_argument("--seconds", type=float, default=30)
    parser.add_argument("--clients", type=int, default=150)
    parser.add_argument("--report-dir", type=Path, default=Path(".local/network-smoke"))
    args = parser.parse_args()
    if not math.isfinite(args.seconds) or args.seconds < 5 or not 1 <= args.clients <= 150:
        parser.error("use --seconds >= 5 and --clients 1..150")
    args.bin_dir = args.bin_dir.resolve()
    for binary in ["server", "net_load"]:
        if not os.access(args.bin_dir / binary, os.X_OK):
            parser.error(f"build {args.bin_dir / binary} first (cargo build --release -p server --bins)")
    directory = args.report_dir.resolve() / datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    directory.mkdir(parents=True)
    print(f"Loopback smoke test; reports: {directory}", flush=True)
    with tempfile.TemporaryDirectory(prefix="midnight-network-") as private:
        identity = Path(private) / "identity"
        invitation = Path(private) / "test.invite"
        descriptor = os.open(invitation, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descriptor, "w") as output:
            subprocess.run(
                [str(args.bin_dir / "server"), "invite", "--data-dir", str(identity)],
                stdout=output, check=True,
            )
        for name in ["crowd", "spread"]:
            scenario(args, name, directory, identity, invitation)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"network_smoke: {error}", file=sys.stderr)
        sys.exit(1)
