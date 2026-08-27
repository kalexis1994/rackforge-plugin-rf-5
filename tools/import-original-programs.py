#!/usr/bin/env python3
"""Convert Sequential's official Rev4 Group 5 SysEx into V8.1 patch bytes.

The official Group 5 programs preserve the vintage 7-bit panel codes and
switch states. This tool deliberately emits only the 40 compact 24-byte Rev 3
records needed by RF-5; the source SysEx remains an external provenance input.
"""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import zipfile


EXPECTED_SYSEX_SHA256 = "0050b8ed0021cb21262e96b0513068c5d36524e04dc45a1095bb73ccaf3fc8f1"
MESSAGE_BYTES = 159
PACKED_BYTES = 152
PROGRAM_BYTES = 24
PROGRAM_COUNT = 40
ORIGINAL_GROUP = 4

# Rev4 NRPN indices in the V8.1 24-pot hardware-multiplexer order.
ANALOG_NRPNS = (
    43, 45, 47, 49,  # filter ADSR
    44, 46, 48, 50,  # amplifier ADSR
    17, 40, 15, 9,   # cutoff, filter env, oscillator B level/PW
    14, 8, 16, 18,   # oscillator A level/PW, noise, resonance
    13, 21, 26, 33,  # glide, LFO rate, Wheel source, Poly oscillator B
    32, 0, 1, 2,     # Poly filter env, oscillator A/B frequency, B fine
)

# Rev4 NRPN indices in the V8.1 22-switch program-storage order.
SWITCH_NRPNS = (
    4, 3, 10, 5, 6, 7, 12, 52,
    34, 35, 36, 23, 24, 25, 19, 51,
    27, 28, 29, 30, 31, 11,
)

PROGRAM_NAMES = (
    "Brass",
    "Low Strings",
    "Muted Clavinet",
    "Percussive e Piano",
    "Flutes",
    "Harpsichord",
    "Sync I",
    "Percussive Organ",
    "Unison Glide w Res",
    "Harmonium",
    "Organ w Resonance",
    "Toy Piano",
    "Trumpet Flute",
    "Filter Mod",
    "Reed Organ",
    "Bass In Fifths",
    "Pipe Organ Flutes",
    "Sync II",
    "Electric Piano I",
    "High Strings",
    "Octave Sawteeth",
    "Release Repeat",
    "Delayed Harmonic",
    "Echo Repeat",
    "Pulse Width Mod",
    "Slow Sync Sweep",
    "Fourths w Resonance",
    "Sweeping Harmonics",
    "Slow Sync",
    "Random Arpeggiator",
    "Sawtooth Arpeggiator",
    "Clangorous Bells",
    "Alien",
    "Noise Sweep",
    "Descending Bells",
    "Descending PWM",
    "Helicopter",
    "Resonance Bells",
    "Hollow Sound",
    "Cat",
)


def source_bytes(path: Path) -> bytes:
    if path.suffix.lower() != ".zip":
        return path.read_bytes()
    with zipfile.ZipFile(path) as archive:
        candidates = [
            name
            for name in archive.namelist()
            if name.lower().endswith(".syx") and not name.startswith("__MACOSX/")
        ]
        if len(candidates) != 1:
            raise ValueError(f"expected one SysEx file in {path}, found {candidates}")
        return archive.read(candidates[0])


def unpack_midi(data: bytes) -> bytes:
    if len(data) != PACKED_BYTES:
        raise ValueError(f"expected {PACKED_BYTES} packed bytes, got {len(data)}")
    unpacked = bytearray()
    for offset in range(0, PACKED_BYTES, 8):
        ms_bits = data[offset]
        for bit, low_bits in enumerate(data[offset + 1 : offset + 8]):
            unpacked.append(low_bits | (((ms_bits >> bit) & 1) << 7))
            if len(unpacked) == 128:
                return bytes(unpacked)
    raise AssertionError("packed SysEx did not yield 128 program bytes")


def original_programs(sysex: bytes) -> list[bytes]:
    digest = hashlib.sha256(sysex).hexdigest()
    if digest != EXPECTED_SYSEX_SHA256:
        raise ValueError(f"unexpected SysEx SHA-256 {digest}")
    if len(sysex) != MESSAGE_BYTES * 200:
        raise ValueError(f"expected 200 program messages, got {len(sysex) / MESSAGE_BYTES}")

    programs: dict[tuple[int, int], bytes] = {}
    for offset in range(0, len(sysex), MESSAGE_BYTES):
        message = sysex[offset : offset + MESSAGE_BYTES]
        if message[:4] != bytes((0xF0, 0x01, 0x32, 0x02)) or message[-1] != 0xF7:
            raise ValueError(f"invalid program message at byte {offset}")
        group, number = message[4], message[5]
        if group > 4 or number >= PROGRAM_COUNT:
            raise ValueError(f"invalid program address group={group} number={number}")
        key = (group, number)
        if key in programs:
            raise ValueError(f"duplicate program address {key}")
        programs[key] = unpack_midi(message[6:-1])

    converted = []
    for number in range(PROGRAM_COUNT):
        parameters = programs[(ORIGINAL_GROUP, number)]
        raw = bytearray()
        for storage_index, analog_nrpn in enumerate(ANALOG_NRPNS):
            pot = parameters[analog_nrpn]
            if pot > 0x7F:
                raise ValueError(f"program {number} pot {storage_index} is not seven-bit")
            switch = False
            if storage_index < len(SWITCH_NRPNS):
                switch_value = parameters[SWITCH_NRPNS[storage_index]]
                # Rev4 represents the vintage FILTER KEYBOARD on-state as FULL=2.
                allowed = (0, 2) if SWITCH_NRPNS[storage_index] == 19 else (0, 1)
                if switch_value not in allowed:
                    raise ValueError(
                        f"program {number} switch {storage_index} has value {switch_value}"
                    )
                switch = switch_value != 0
            raw.append(pot | (0x80 if switch else 0))
        if len(raw) != PROGRAM_BYTES:
            raise AssertionError("wrong converted program size")
        converted.append(bytes(raw))
    return converted


def slug(name: str) -> str:
    return "-".join("".join(c.lower() if c.isalnum() else " " for c in name).split())


def render_rust(programs: list[bytes]) -> str:
    matrix = b"".join(programs)
    lines = [
        "// @generated by tools/import-original-programs.py; do not edit by hand.",
        f"// Source SysEx SHA-256: {EXPECTED_SYSEX_SHA256}",
        f"// Converted V8.1 matrix SHA-256: {hashlib.sha256(matrix).hexdigest()}",
        "",
        "use rf_5_contract::hardware::{PROGRAM_BYTES, ProgramByte};",
        "",
        "#[derive(Clone, Copy, Debug)]",
        "pub(crate) struct OriginalProgram {",
        "    pub id: &'static str,",
        "    pub raw: [ProgramByte; PROGRAM_BYTES],",
        "}",
        "",
        f"pub(crate) const ORIGINAL_PROGRAMS: [OriginalProgram; {PROGRAM_COUNT}] = [",
    ]
    for index, (name, raw) in enumerate(zip(PROGRAM_NAMES, programs, strict=True)):
        bank, slot = divmod(index, 8)
        program_id = f"original-{bank + 1}{slot + 1}-{slug(name)}"
        bytes_rust = ", ".join(f"ProgramByte::from_raw(0x{value:02x})" for value in raw)
        lines.extend(
            (
                "    OriginalProgram {",
                f'        id: "{program_id}",',
                f"        raw: [{bytes_rust}],",
                "    },",
            )
        )
    lines.extend(("];", ""))
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path, help="official Sequential ZIP or .syx file")
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("crates/rf-5-dsp/src/original_programs_data.rs"),
    )
    args = parser.parse_args()
    programs = original_programs(source_bytes(args.source))
    rendered = render_rust(programs)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(rendered, encoding="utf-8", newline="\n")
    print(f"wrote {len(programs)} exact V8.1 programs to {args.output}")


if __name__ == "__main__":
    main()
