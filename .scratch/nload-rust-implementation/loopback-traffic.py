#!/usr/bin/env python3
"""Throwaway, nonprivileged live-smoke traffic; not an acceptance rate oracle."""

import argparse
import math
import os
from pathlib import Path
import socket
import time


def loopback_counters():
    for row in Path("/proc/net/dev").read_bytes().splitlines()[2:]:
        name, fields = row.split(b":", 1)
        if name.strip() == b"lo":
            values = fields.split()
            return int(values[0]), int(values[8])
    raise RuntimeError("lo is absent from the accessible proc view")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--phase-seconds", type=float, default=10.0)
    args = parser.parse_args()
    if not math.isfinite(args.phase_seconds) or args.phase_seconds <= 0:
        parser.error("--phase-seconds must be finite and positive")

    before = loopback_counters()
    print(f"uid/euid={os.getuid()}/{os.geteuid()}; interface=lo; IPv4 UDP 127.0.0.1", flush=True)
    print(f"Before: RX={before[0]} TX={before[1]} bytes", flush=True)
    payload = b"n" * 8192
    datagrams = 0
    started = time.monotonic()
    # Receive each datagram immediately, avoiding an unbounded queue or a thread.
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as receiver:
        receiver.bind(("127.0.0.1", 0))
        receiver.settimeout(2.0)
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sender:
            sender.connect(receiver.getsockname())
            sender.settimeout(2.0)
            for label, packets_per_second in (
                ("low", 32), ("high", 128), ("idle", 0),
                ("low", 32), ("high", 128), ("idle", 0),
            ):
                print(f"Phase: {label}, {args.phase_seconds:g} seconds", flush=True)
                deadline = time.monotonic() + args.phase_seconds
                if not packets_per_second:
                    time.sleep(max(0.0, deadline - time.monotonic()))
                    continue
                while time.monotonic() < deadline:
                    tick = time.monotonic()
                    sender.send(payload)
                    received = receiver.recv(65535)
                    if received != payload:
                        raise RuntimeError("unexpected loopback datagram")
                    datagrams += 1
                    time.sleep(max(0.0, min(deadline, tick + 1 / packets_per_second) - time.monotonic()))

    after = loopback_counters()
    print(f"Elapsed: {time.monotonic() - started:.3f} seconds; received datagrams={datagrams}")
    print(f"After: RX={after[0]} TX={after[1]} bytes")
    print(f"Observed proc delta: RX={after[0] - before[0]} TX={after[1] - before[1]} bytes")
    print("Proc deltas include any other loopback traffic; payload/pace are not exact UI-rate expectations.")


if __name__ == "__main__":
    main()
