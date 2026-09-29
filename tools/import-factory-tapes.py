#!/usr/bin/env python3
"""Read the Prophet-5 Rev 3 factory program files off their cassette tapes.

Sequential shipped the Rev 3 with its 120 factory programs as three 40-program
files on cassette; the owner's manual (CM1000D, section 8) says loading them
restores the instrument's original programs. These are the programs exactly
as the Rev 3 stored them: 24 bytes each, a pot's seven-bit code and a switch
bit per byte, never a pot above the ADC's panel ceiling of code 120.

The tape format (see docs/fidelity/ORIGINAL_FACTORY_PROGRAMS.md): half-cycles
counted in 232.4 us minims, a bit sixteen minims (a one twelve short
half-cycles and a long one, a zero four long ones), a leader of ones, a zero
start bit, 960 data bytes most significant bit first and their sum modulo
256. Every file must pass that checksum.

One slot is not taken from the tape: on the File 1 recording, program 1-8
holds a byte-exact copy of 1-1 (Brass), recorded over the Percussive Organ.
Program 1-8 is therefore kept from Sequential's Rev 4 recreation of the
original programs (tools/import-original-programs.py), which Sequential's
original patch sheet for 1-8 confirms.

Names come from the manual's program maps; File 1 keeps the names RF-5 has
always used.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import re
import wave
import zipfile
from pathlib import Path

EXPECTED_ZIP_SHA256 = "c2bc0b117c5b76db512c720a742aa36eb0795e67823cb5f4e2091a597d5e48eb"
TAPES = {
    "FACT1R3.WAV": "e2e838624e3f548786f375cb2da1cffa809126a71263889c9d59bb7aede04266",
    "FACT2R3.WAV": "071e790e867c09d4233d9307916d5f9190cb33dfa5a0732d452eddd0f5833564",
    "FACT3R3.WAV": "0ef2884297965c344e72750e6b43b8b4b925635f974216c82ef72114f6bd1229",
}
MINIM_SECONDS = 232.4e-6
PROGRAM_BYTES = 24
PROGRAMS_PER_FILE = 40
FILE_BYTES = PROGRAM_BYTES * PROGRAMS_PER_FILE
PANEL_CEILING = 121

# Program 1-8, Percussive Organ, from Group 5 program 8 of Sequential's
# P5_Factory_Programs_v1.02.syx (SHA-256 0050b8ed...), converted by
# tools/import-original-programs.py.
PERCUSSIVE_ORGAN = bytes.fromhex("804a0e258000f822232a5e3fab3b800000db000000571800")

FILE_1 = (
    "Brass", "Low Strings", "Muted Clavinet", "Percussive e Piano",
    "Flutes", "Harpsichord", "Sync I", "Percussive Organ",
    "Unison Glide w Res", "Harmonium", "Organ w Resonance", "Toy Piano",
    "Trumpet Flute", "Filter Mod", "Reed Organ", "Bass in Fifths",
    "Pipe Organ Flutes", "Sync II", "Electric Piano I", "High Strings",
    "Octave Sawteeth", "Release Repeat", "Delayed Harmonic", "Echo Repeat",
    "Pulse Width Mod", "Slow Sync Sweep", "Fourths w Resonance", "Sweeping Harmonics",
    "Slow Sync", "Random Arpeggiator", "Sawtooth Arpeggiator", "Clangorous Bells",
    "Alien", "Noise Sweep", "Descending Bells", "Descending PWM",
    "Helicopter", "Resonance Bells", "Hollow Sound", "Cat",
)
FILE_2 = (
    "Percussive Flute w Delay", "Comping #42", "Ensemble w Release", "Cutting Through",
    "Quack #9", "Camp Argon", "Reedy Solo", "Solo Sync #73",
    "Bass Guitar Unison", "Spitting Organ", "Slow Orchestral Brass", "Pulse Width Echo",
    "Dept. Store Clarinet", "Slow Strings", "Solo Violin", "High Violins",
    "Clav Slightly Muted", "Electric Piano II", "Barking Organ", "1960's Organ",
    "Jazz Organ", "Rock Organ", "Angelic", "Fat Solo in Sync",
    "Painful Solo", "Electrocution", "Launching Spacecrafts", "Digital Indigestion",
    "Whistle w Glissando", "Rubber Knife", "Digital Solo", "Banshee Jets",
    "Descending Resonances", "Cats Meow", "Birds", "Dog",
    "Whistle", "Ascending Bleeps", "Morning After", "N.Y.C.",
)
FILE_3 = (
    "Malleted Marimba", "Trumpet w Spit", "Oboe", "Trombones",
    "Percolating Grunge", "Wurlie Electric Piano", "Church Organ", "Slow Brass",
    "Triffid Organ", "Plucked Harp", "Arco Violin", "Arco Ensemble",
    "Harpsichord", "Slow Strings", "Plucking w Brass on VCF Pedal", "Water Organ",
    "Pulse Width Delay", "Dulcimer Organ", "Banjo", "Du. Keg Solo",
    "Sync Comp", "Flutes in Fifths", "Slide Guitar", "Steel Drums",
    "The Nuge R&R Guitar", "Video Games", "Final Frontier", "The Landing",
    "Dead Droids", "Steam Engine", "Detroit Solo", "Formula II Race Cars",
    "Snare", "Tom-Toms", "Tympani", "Electric Drums",
    "Bombs Dropping", "Gunshots", "Wind", "Thunder",
)

# (tape, id prefix, bank id, bank name, tag, names)
FILES = (
    ("FACT1R3.WAV", "original", "factory.rf5.original", "RF-5 Original 40", "original-40", FILE_1),
    ("FACT2R3.WAV", "file2", "factory.rf5.file2", "RF-5 File 2", "file-2", FILE_2),
    ("FACT3R3.WAV", "file3", "factory.rf5.file3", "RF-5 File 3", "file-3", FILE_3),
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def samples(recording: bytes) -> tuple[list[int], int]:
    with wave.open(io.BytesIO(recording)) as reader:
        if reader.getnchannels() != 1 or reader.getsampwidth() != 2:
            raise ValueError("expected a mono 16-bit recording")
        frames = reader.readframes(reader.getnframes())
        rate = reader.getframerate()
    values = [int.from_bytes(frames[i:i + 2], "little", signed=True) for i in range(0, len(frames), 2)]
    return values, rate


def half_cycles(values: list[int]) -> list[float]:
    """Zero-crossing times with hysteresis, interpolated between samples."""
    peak = sorted(abs(value) for value in values[::7])[-len(values[::7]) // 100]
    threshold = 0.2 * peak
    edges, high, last_zero = [], values[0] > 0, 0.0
    for index in range(1, len(values)):
        before, after = values[index - 1], values[index]
        if (before <= 0) != (after <= 0):
            last_zero = index - 1 + before / (before - after)
        if not high and after > threshold:
            high = True
            edges.append(last_zero)
        elif high and after < -threshold:
            high = False
            edges.append(last_zero)
    return [b - a for a, b in zip(edges, edges[1:])]


def tape_bytes(recording: bytes) -> bytes:
    values, rate = samples(recording)
    intervals = half_cycles(values)
    nominal = MINIM_SECONDS * rate
    shorts = sorted(i for i in intervals if 0.6 * nominal < i < 1.6 * nominal)
    minim = shorts[len(shorts) // 2]
    bits: list[int] = []
    units = count = 0
    aligned = in_data = False
    run = ones = 0
    for interval in intervals:
        length = max(1, min(8, round(interval / minim)))
        if length == 1:
            minim += 0.01 * (interval - minim)
        elif length == 4:
            minim += 0.01 * (interval / 4 - minim)
        if not aligned:
            if length == 1:
                run += 1
            elif length == 4 and run >= 8:
                aligned, units, count = True, 0, 0
            else:
                run = 0
            continue
        units += length
        count += 1
        if units < 16:
            continue
        bit = 1 if count > 7 else 0
        if units != 16 or count not in (4, 13):
            raise ValueError(f"damaged bit after {len(bits)} data bits")
        units = count = 0
        if in_data:
            bits.append(bit)
            if len(bits) == (FILE_BYTES + 1) * 8:
                break
        elif bit:
            ones += 1
        elif ones >= 32:
            in_data = True
        else:
            aligned, run, ones = False, 0, 0
    # The recording may stop on the checksum's last half-cycle.
    if in_data and len(bits) == (FILE_BYTES + 1) * 8 - 1 and units >= 8:
        bits.append(1 if count > 7 else 0)
    if len(bits) != (FILE_BYTES + 1) * 8:
        raise ValueError(f"the tape ends after {len(bits)} data bits")
    data = bytes(int("".join(map(str, bits[i:i + 8])), 2) for i in range(0, len(bits), 8))
    if sum(data[:FILE_BYTES]) % 256 != data[FILE_BYTES]:
        raise ValueError("checksum failed")
    return data[:FILE_BYTES]


def slug(name: str) -> str:
    name = name.replace("&", " and ").replace("'", "")
    return "-".join(re.sub(r"[^a-z0-9]+", " ", name.lower()).split())


def factory_programs(archive: Path) -> list[tuple[str, str, str, str, bytes]]:
    """(id, name, bank id, tag, raw) for all 120 programs, in catalog order."""
    container = archive.read_bytes()
    if sha256(container) != EXPECTED_ZIP_SHA256:
        raise ValueError(f"unexpected tape archive SHA-256 {sha256(container)}")
    programs = []
    with zipfile.ZipFile(io.BytesIO(container)) as tapes:
        for tape, prefix, bank, _, tag, names in FILES:
            recording = tapes.read(tape)
            if sha256(recording) != TAPES[tape]:
                raise ValueError(f"unexpected {tape} SHA-256")
            data = tape_bytes(recording)
            for index, name in enumerate(names):
                raw = data[index * PROGRAM_BYTES:(index + 1) * PROGRAM_BYTES]
                if tape == "FACT1R3.WAV" and index == 7:
                    if raw != data[:PROGRAM_BYTES]:
                        raise ValueError("File 1 slot 1-8 is no longer the copy of 1-1")
                    raw = PERCUSSIVE_ORGAN
                elif any(byte & 0x7F > PANEL_CEILING for byte in raw):
                    raise ValueError(f"{tape} program {index} exceeds the panel ceiling")
                if raw[22] & 0x80 or raw[23] & 0x80:
                    raise ValueError(f"{tape} program {index} sets an unused switch bit")
                place = f"{index // 8 + 1}{index % 8 + 1}"
                programs.append((f"{prefix}-{place}-{slug(name)}", name, bank, tag, raw))
    return programs


def render_rust(programs, digest: str) -> str:
    matrix = b"".join(raw for *_, raw in programs)
    lines = [
        "// @generated by tools/import-factory-tapes.py; do not edit by hand.",
        f"// Source tape archive SHA-256: {digest}",
        f"// Program matrix SHA-256: {sha256(matrix)}",
        "",
        "use rf_5_contract::hardware::{PROGRAM_BYTES, ProgramByte};",
        "",
        "#[derive(Clone, Copy, Debug)]",
        "pub(crate) struct OriginalProgram {",
        "    pub id: &'static str,",
        "    pub raw: [ProgramByte; PROGRAM_BYTES],",
        "}",
        "",
        "/// The Rev 3's three factory program files, 40 programs each.",
        f"pub(crate) const ORIGINAL_PROGRAMS: [OriginalProgram; {len(programs)}] = [",
    ]
    for program_id, _, _, _, raw in programs:
        lines.append("    OriginalProgram {")
        lines.append(f'        id: "{program_id}",')
        lines.append("        raw: [")
        lines.extend(f"            ProgramByte::from_raw(0x{value:02x})," for value in raw)
        lines.append("        ],")
        lines.append("    },")
    lines.extend(("];", ""))
    return "\n".join(lines)


def render_catalog(programs, extra_banks) -> str:
    banks = [{"id": bank, "name": name, "order": order}
             for order, (_, _, bank, name, _, _) in enumerate(FILES)] + extra_banks
    catalog_order = 0
    lines = ["{", '  "banks": [']
    lines.append(",\n".join(f"    {inline(bank)}" for bank in banks))
    lines.extend(("  ],", '  "presets": ['))
    presets = []
    for index, (program_id, name, bank, tag, _) in enumerate(programs):
        place = index % PROGRAMS_PER_FILE
        presets.append({
            "bank": bank,
            "editable": False,
            "id": program_id,
            "name": f"{place // 8 + 1}-{place % 8 + 1} {name}",
            "order": catalog_order,
            "tags": ["rf5", tag, "rev3"],
        })
        catalog_order += 1
    lines.append(",\n".join(f"    {inline(preset)}" for preset in presets))
    lines.extend(("  ],", '  "schema_version": 1', "}", ""))
    return "\n".join(lines)


def inline(value: dict) -> str:
    """One catalog entry on one line, as presets.json is laid out."""
    return "{ " + ", ".join(f"{json.dumps(key)}: {json.dumps(item, ensure_ascii=False)}"
                            for key, item in value.items()) + " }"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("archive", type=Path, help="synthchaser's Prophet_5_Factory_Patches_Rev_3.zip")
    parser.add_argument("--data", type=Path, default=Path("crates/rf-5-dsp/src/original_programs_data.rs"))
    parser.add_argument("--catalog", type=Path, default=Path("plugin/package/metadata/presets.json"))
    args = parser.parse_args()
    programs = factory_programs(args.archive)
    digest = sha256(args.archive.read_bytes())
    args.data.write_text(render_rust(programs, digest), encoding="utf-8", newline="\n")
    # Banks the catalog declares beyond the factory files (the USER bank)
    # are kept.
    existing = json.loads(args.catalog.read_text(encoding="utf-8"))
    factory_banks = {bank for _, _, bank, _, _, _ in FILES}
    extra = [bank for bank in existing["banks"] if bank["id"] not in factory_banks]
    args.catalog.write_text(render_catalog(programs, extra), encoding="utf-8", newline="\n")
    print(f"wrote {len(programs)} factory programs to {args.data} and {args.catalog}")


if __name__ == "__main__":
    main()
