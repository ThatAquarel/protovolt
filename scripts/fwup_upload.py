#!/usr/bin/env python3
"""Upload signed firmware to ProtoV over USB CDC (SCPI FWUP protocol)."""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

try:
    import serial
except ImportError:
    print("pyserial is required: pip install pyserial", file=sys.stderr)
    sys.exit(1)

DEFAULT_CHUNK = 4096
STAR_TIMEOUT_S = 120.0
IO_TIMEOUT_S = 30.0


def ieee_definite_block(data: bytes) -> bytes:
    n = len(data)
    digits = str(n)
    header = f"#{len(digits)}{digits}".encode("ascii")
    return header + data


def read_line(ser: serial.Serial, timeout: float = IO_TIMEOUT_S) -> str:
    deadline = time.monotonic() + timeout
    buf = bytearray()
    while time.monotonic() < deadline:
        chunk = ser.read(ser.in_waiting or 1)
        if chunk:
            buf.extend(chunk)
            if b"\n" in buf:
                line = buf.split(b"\n", 1)[0]
                return line.decode("ascii", errors="replace").strip()
        else:
            time.sleep(0.01)
    raise TimeoutError("timed out waiting for response")


def write_cmd(ser: serial.Serial, cmd: str, timeout: float = IO_TIMEOUT_S) -> str:
    ser.write((cmd + "\n").encode("ascii"))
    ser.flush()
    return read_line(ser, timeout=timeout)


def write_data_block(ser: serial.Serial, payload: bytes) -> str:
    prefix = b"SYST:FWUP:DATA "
    ser.write(prefix + ieee_definite_block(payload) + b"\n")
    ser.flush()
    return read_line(ser)


def query_err(ser: serial.Serial) -> None:
    try:
        err = write_cmd(ser, "SYST:ERR?")
        stat = write_cmd(ser, "SYST:FWUP:STAT?")
        print(f"SYST:ERR? -> {err}", file=sys.stderr)
        print(f"SYST:FWUP:STAT? -> {stat}", file=sys.stderr)
    except TimeoutError:
        pass


def wait_receiving(ser: serial.Serial, total: int, timeout: float = STAR_TIMEOUT_S) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        stat = write_cmd(ser, "SYST:FWUP:STAT?")
        if stat.startswith("RECV,") or stat.startswith("READY,"):
            return
        if stat == "ERROR":
            raise RuntimeError("device reported ERROR during prepare")
        if stat.startswith("PREPARE,"):
            time.sleep(0.2)
            continue
        time.sleep(0.1)
    raise TimeoutError(f"prepare did not reach RECV within {timeout}s (last stat may be PREPARE)")


def upload(
    port: str,
    firmware: Path,
    signature: Path,
    chunk_size: int,
) -> None:
    fw = firmware.read_bytes()
    sig = signature.read_bytes()
    if len(sig) != 64:
        raise ValueError(f"signature must be 64 bytes, got {len(sig)}")

    with serial.Serial(port, baudrate=115200, timeout=0.1) as ser:
        time.sleep(0.5)
        print(write_cmd(ser, "*IDN?"))
        print(write_cmd(ser, "SYST:FWUP:STAT?"))

        star = write_cmd(ser, f"SYST:FWUP:STAR {len(fw)}", timeout=STAR_TIMEOUT_S)
        print(f"STAR -> {star}")
        if star != "OK":
            query_err(ser)
            raise RuntimeError(f"STAR failed: {star}")

        wait_receiving(ser, len(fw))

        offset = 0
        while offset < len(fw):
            chunk = fw[offset : offset + chunk_size]
            resp = write_data_block(ser, chunk)
            print(f"DATA @{offset} ({len(chunk)} B) -> {resp}")
            if resp != "OK":
                query_err(ser)
                raise RuntimeError(f"DATA block @ {offset} failed: {resp}")
            offset += len(chunk)

        sig_hex = sig.hex().upper()
        appl = write_cmd(ser, f"SYST:FWUP:APPL #H{sig_hex}")
        print(f"APPL -> {appl}")
        if appl != "OK":
            query_err(ser)
            raise RuntimeError(f"APPL failed: {appl}")

        print("Upload complete; device should reset if verify succeeded.")


def main() -> None:
    parser = argparse.ArgumentParser(description="Upload signed firmware via SCPI FWUP")
    parser.add_argument("--port", required=False, help="Serial port (e.g. /dev/ttyACM0)", default="/dev/ttyACM2")
    parser.add_argument("--firmware", type=Path, required=False, help="Raw .bin firmware", default="dist/protov.bin")
    parser.add_argument("--signature", type=Path, required=False, help="64-byte Ed25519 signature", default="dist/protov_sign.bin")
    parser.add_argument(
        "--chunk-size",
        type=int,
        default=DEFAULT_CHUNK,
        help=f"Bytes per DATA block (default {DEFAULT_CHUNK})",
    )
    args = parser.parse_args()

    try:
        upload(args.port, args.firmware, args.signature, args.chunk_size)
    except (RuntimeError, TimeoutError, serial.SerialException) as exc:
        print(f"error: {exc}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
