//! The Prophet-5 Rev 4's MIDI SysEx program dumps: the files Prophet-5
//! owners trade today, the factory sets Sequential publishes among them.
//!
//! A program dump is `F0 01 32 02 <group> <program> <152 bytes> F7` (an edit
//! buffer dump, `F0 01 32 03 <152 bytes> F7`, has no address). The 152 bytes
//! are the program's parameters, one byte each in the order of the Rev 4's
//! NRPN list, packed seven to eight: each packet of eight opens with the top
//! bits of the seven bytes after it. The name is bytes 65-84.
//!
//! The Rev 4 keeps the vintage 7-bit pot positions and the switches, so a
//! Rev 3 program crosses both ways exactly; what the Rev 4 adds (velocity,
//! aftertouch, the vintage knob, unison voices) is written as its own
//! factory recreation of the Rev 3 programs sets it, and left out on the
//! way in.

use super::{Medium, Memory, PROGRAM_BYTES, Program, ReadError, Reading, clean_name, error};

const SYSEX: u8 = 0xF0;
const END: u8 = 0xF7;
const SEQUENTIAL: u8 = 0x01;
/// The Prophet-5's device id: 0x32 in Sequential's own files, 0x31 in the
/// Rev 4's MIDI implementation.
const PROPHET_5: [u8; 2] = [0x32, 0x31];
const PROGRAM_DUMP: u8 = 0x02;
const EDIT_BUFFER_DUMP: u8 = 0x03;
const PACKED_BYTES: usize = 152;
/// The parameters a dump carries.
const PARAMETERS: usize = 128;
const NAME: core::ops::Range<usize> = 65..85;
/// The Rev 4's programs: 5 user and 5 factory groups of 40.
const GROUP_PROGRAMS: usize = 40;
pub const MAX_PROGRAMS: usize = 5 * GROUP_PROGRAMS;

/// The Rev 4 parameter of each Rev 3 pot, in program-memory order.
const POTS: [usize; PROGRAM_BYTES] = [
    43, 45, 47, 49, // filter ADSR
    44, 46, 48, 50, // amplifier ADSR
    17, 40, 15, 9, // cutoff, filter envelope amount, OSC B level and PW
    14, 8, 16, 18, // OSC A level and PW, noise, resonance
    13, 21, 26, 33, // glide, LFO rate, wheel source mix, poly-mod OSC B
    32, 0, 1, 2, // poly-mod filter envelope, OSC A and B frequency, B fine
];

/// The Rev 4 parameter of each Rev 3 switch, in program-memory order.
const SWITCHES: [usize; 22] = [
    4, 3, 10, 5, 6, 7, 12, 52, // OSC A, sync, OSC B, B keyboard, unison
    34, 35, 36, 23, 24, 25, 19, 51, // poly-mod, LFO shape, filter keyboard, release
    27, 28, 29, 30, 31, 11, // wheel-mod, B low frequency
];
const FILTER_KEYBOARD: usize = 19;
const UNISON: usize = 52;

/// The Rev 4's own settings, as its factory recreation of the Rev 3
/// programs has them: the Rev 3 filter, a touch of vintage, no velocity or
/// aftertouch, no unison detune. Unison takes all the voices.
const REV4_DEFAULTS: [(usize, u8); 4] = [
    (20, 1),  // filter: Rev 3
    (37, 42), // vintage
    (86, 6),
    (94, 1),
];
const UNISON_VOICES: usize = 53;
const UNISON_ALL_VOICES: u8 = 4;

fn unpack(packed: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(PARAMETERS + 7);
    for packet in packed.chunks(8) {
        let (top, rest) = packet.split_first().unwrap_or((&0, &[]));
        for (bit, low) in rest.iter().enumerate() {
            bytes.push(low & 0x7F | ((top >> bit) & 1) << 7);
        }
    }
    bytes
}

fn pack(bytes: &[u8]) -> Vec<u8> {
    let mut packed = Vec::with_capacity(PACKED_BYTES);
    for chunk in bytes.chunks(7) {
        let mut top = 0;
        for (bit, byte) in chunk.iter().enumerate() {
            top |= (byte >> 7) << bit;
        }
        packed.push(top);
        packed.extend(chunk.iter().map(|byte| byte & 0x7F));
        packed.extend(core::iter::repeat_n(0, 7 - chunk.len()));
    }
    packed
}

/// A Rev 3 program from a Rev 4 parameter set, and whether any value had
/// to be brought back into the Rev 3's range.
fn memory(parameters: &[u8]) -> (Memory, bool) {
    let mut memory = [0; PROGRAM_BYTES];
    let mut clamped = false;
    for (index, byte) in memory.iter_mut().enumerate() {
        let pot = parameters[POTS[index]];
        clamped |= pot > 0x7F;
        *byte = pot.min(0x7F);
        // FILTER KEYBOARD is OFF/HALF/FULL on the Rev 4: any tracking is on.
        if let Some(&switch) = SWITCHES.get(index)
            && parameters[switch] != 0
        {
            *byte |= 0x80;
        }
    }
    (memory, clamped)
}

fn parameters(memory: &Memory, name: &str) -> [u8; PARAMETERS] {
    let mut parameters = [0; PARAMETERS];
    for (index, byte) in memory.iter().enumerate() {
        parameters[POTS[index]] = byte & 0x7F;
        if let Some(&switch) = SWITCHES.get(index) {
            let on = byte & 0x80 != 0;
            parameters[switch] = match (switch, on) {
                (FILTER_KEYBOARD, true) => 2,
                (_, on) => u8::from(on),
            };
        }
    }
    for (parameter, value) in REV4_DEFAULTS {
        parameters[parameter] = value;
    }
    if parameters[UNISON] != 0 {
        parameters[UNISON_VOICES] = UNISON_ALL_VOICES;
    }
    // No unison chord: every note slot empty.
    parameters[55..65].fill(0x7F);
    let name = name.to_ascii_uppercase();
    let mut characters = name.chars().map(|character| {
        if (' '..='~').contains(&character) {
            character as u8
        } else {
            b'?'
        }
    });
    for slot in &mut parameters[NAME] {
        *slot = characters.next().unwrap_or(b' ');
    }
    parameters
}

/// Reads every Prophet-5 program dump in a SysEx file, in order.
pub fn read(bytes: &[u8]) -> Result<Reading, ReadError> {
    let mut programs = Vec::new();
    let (mut skipped, mut clamped) = (0, 0);
    let mut rest = bytes;
    while let Some(start) = rest.iter().position(|byte| *byte == SYSEX) {
        let Some(length) = rest[start..].iter().position(|byte| *byte == END) else {
            skipped += 1;
            break;
        };
        let message = &rest[start..=start + length];
        rest = &rest[start + length + 1..];
        let (place, packed) = match message {
            [
                SYSEX,
                SEQUENTIAL,
                device,
                PROGRAM_DUMP,
                group,
                number,
                packed @ ..,
                END,
            ] if PROPHET_5.contains(device) && packed.len() == PACKED_BYTES => {
                let number = usize::from(*number);
                (
                    format!("{}:{}-{}", group, number / 8 + 1, number % 8 + 1),
                    packed,
                )
            }
            [
                SYSEX,
                SEQUENTIAL,
                device,
                EDIT_BUFFER_DUMP,
                packed @ ..,
                END,
            ] if PROPHET_5.contains(device) && packed.len() == PACKED_BYTES => {
                ("EDIT".to_owned(), packed)
            }
            _ => {
                skipped += 1;
                continue;
            }
        };
        let parameters = unpack(packed);
        let (memory, out_of_range) = memory(&parameters);
        clamped += usize::from(out_of_range);
        let name: String = parameters[NAME]
            .iter()
            .map(|byte| char::from(*byte))
            .collect();
        programs.push(Program {
            place,
            name: clean_name(&name),
            memory,
        });
    }
    if programs.is_empty() {
        return error("NO PROPHET-5 PROGRAMS IN THIS SYSEX");
    }
    let warning = match (skipped, clamped) {
        (0, 0) => None,
        (skipped, 0) => Some(format!("{skipped} OTHER SYSEX MESSAGES SKIPPED")),
        (_, clamped) => Some(format!(
            "{clamped} PROGRAMS HAD VALUES PAST THE REV 3'S RANGE"
        )),
    };
    Ok(Reading {
        medium: Medium::Sysex,
        programs,
        warning,
    })
}

/// Writes programs as Rev 4 program dumps, into the user groups from
/// group 0 program 0 on. Only the first 200 are written.
pub fn write<'a>(programs: impl IntoIterator<Item = (&'a str, &'a Memory)>) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (index, (name, memory)) in programs.into_iter().take(MAX_PROGRAMS).enumerate() {
        bytes.extend([
            SYSEX,
            SEQUENTIAL,
            PROPHET_5[0],
            PROGRAM_DUMP,
            (index / GROUP_PROGRAMS) as u8,
            (index % GROUP_PROGRAMS) as u8,
        ]);
        bytes.extend(pack(&parameters(memory, name)));
        bytes.push(END);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: Memory = [
        0x31, 0xc3, 0x36, 0x30, 0x2d, 0x28, 0xcc, 0x36, 0x06, 0x62, 0x78, 0x11, 0xf8, 0x0c, 0x80,
        0x85, 0x80, 0xdc, 0x00, 0x00, 0x00, 0x19, 0x31, 0x00,
    ];

    #[test]
    fn packing_is_the_rev_4s() {
        let bytes: Vec<u8> = (0..PARAMETERS).map(|index| (index * 37) as u8).collect();
        let packed = pack(&bytes);
        assert_eq!(packed.len(), PACKED_BYTES);
        assert!(packed.iter().all(|byte| *byte < 0x80));
        assert_eq!(&unpack(&packed)[..PARAMETERS], &bytes[..]);
    }

    #[test]
    fn programs_round_trip_with_their_names() {
        let mut other = SAMPLE;
        other[14] &= 0x7F;
        let file = write([("1-1 Brass", &SAMPLE), ("Señor", &other)]);
        assert_eq!(file.len(), 2 * (7 + PACKED_BYTES));
        assert_eq!(&file[..6], &[0xF0, 0x01, 0x32, 0x02, 0, 0]);
        let reading = read(&file).unwrap();
        assert_eq!(reading.medium, Medium::Sysex);
        assert_eq!(reading.warning, None);
        assert_eq!(reading.programs[0].memory, SAMPLE);
        assert_eq!(reading.programs[0].name.as_deref(), Some("1-1 BRASS"));
        assert_eq!(reading.programs[0].place, "0:1-1");
        assert_eq!(reading.programs[1].memory, other);
        assert_eq!(reading.programs[1].name.as_deref(), Some("SE?OR"));
    }

    #[test]
    fn the_rev_4s_own_settings_follow_its_rev_3_recreations() {
        let parameters = parameters(&SAMPLE, "");
        // FILTER KEYBOARD on is FULL; the Rev 3 filter; no unison.
        assert_eq!(parameters[FILTER_KEYBOARD], 2);
        assert_eq!(parameters[20], 1);
        assert_eq!(parameters[UNISON_VOICES], 0);
        let mut unison = SAMPLE;
        unison[7] |= 0x80;
        assert_eq!(
            super::parameters(&unison, "")[UNISON_VOICES],
            UNISON_ALL_VOICES
        );
    }

    #[test]
    fn edit_buffers_half_tracking_and_other_messages() {
        let mut parameters = parameters(&SAMPLE, "HALF");
        parameters[FILTER_KEYBOARD] = 1;
        parameters[POTS[0]] = 0x90;
        let mut file = vec![0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7];
        file.extend([0xF0, 0x01, 0x31, 0x03]);
        file.extend(pack(&parameters));
        file.push(0xF7);
        let reading = read(&file).unwrap();
        assert_eq!(reading.programs.len(), 1);
        assert_eq!(reading.programs[0].place, "EDIT");
        assert_eq!(reading.programs[0].memory[14] & 0x80, 0x80);
        assert_eq!(reading.programs[0].memory[0], 0x7F);
        assert!(reading.warning.unwrap().contains("PAST THE REV 3"));
        assert!(read(&[0xF0, 0x7E, 0xF7]).is_err());
        assert!(read(&[0xF0, 0x01, 0x32, 0x02]).is_err());
    }

    /// Sequential's factory sets for the Rev 4, when a copy is at hand.
    #[test]
    fn sequentials_factory_dumps_read() {
        let Ok(path) = std::env::var("RF5_REV4_SYSEX") else {
            return;
        };
        let bytes = std::fs::read(path).unwrap();
        let reading = read(&bytes).unwrap();
        assert_eq!(reading.programs.len(), 200);
    }
}
