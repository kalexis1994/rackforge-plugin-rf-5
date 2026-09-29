//! The Rev 3's cassette interface, as recordings of it.
//!
//! The Prophet writes its program memory to tape as a string of half-cycles
//! whose lengths count in "minims" of 232.4 µs. A bit is sixteen minims: a one
//! is twelve one-minim half-cycles and a four-minim one, a zero four
//! four-minim half-cycles. The Prophet's reader tells them apart by counting
//! edges in a bit's time, so neither the waveform nor its polarity matters.
//!
//! A recording opens with a leader of ones (about 18 s from the Prophet),
//! then a zero as start bit, then the program bytes, most significant bit
//! first: a whole program file of 40 programs (960 bytes) or one bank of 8
//! (192 bytes). A last byte carries their sum, modulo 256, which the Prophet
//! checks when it verifies a tape. All of it was read off the factory tapes
//! Sequential shipped with the Rev 3.
//!
//! Reading copes with what real recordings do to that: any gain and DC,
//! hum and hiss, a tape running a little fast or slow and drifting while it
//! plays, stereo decks with one dead channel, and any PCM or float WAV.

use super::{
    INIT_PROGRAM, Medium, Memory, PROGRAM_BYTES, Program, ReadError, Reading, bank_place, error,
};

/// The interface's unit of time.
const MINIM_SECONDS: f64 = 232.4e-6;
/// A bit's length, in minims.
const BIT_MINIMS: u32 = 16;
/// A whole program file, and one bank of it.
pub const FILE_PROGRAMS: usize = 40;
pub const BANK_PROGRAMS: usize = 8;
/// The leader the factory tapes open with, in bits.
const LEADER_BITS: usize = 4_767;
/// Ones written after the checksum, so its last half-cycle is closed.
const TAIL_BITS: usize = 16;
/// Leader ones a reading must hear before it trusts a start bit.
const LEADER_MINIMUM: usize = 32;
/// The recordings written: CD rate, sixteen bits, mono.
pub const SAMPLE_RATE: u32 = 44_100;
/// The written square wave's level, of full scale.
const LEVEL: f64 = 0.6;
/// Silence round a written recording, in seconds.
const QUIET_SECONDS: f64 = 0.5;
/// Below this cutoff the reader ignores what it hears: DC, hum, rumble.
const HIGH_PASS_HZ: f64 = 60.0;
/// The reader's hysteresis, of the recording's peak level.
const HYSTERESIS: f32 = 0.2;

/// A recording, reduced to the one channel the tape was heard on.
#[derive(Clone, Debug, PartialEq)]
pub struct Wave {
    pub rate: u32,
    pub samples: Vec<f32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Encoding {
    Integer,
    Float,
}

#[derive(Clone, Copy, Debug)]
struct Format {
    encoding: Encoding,
    channels: usize,
    rate: u32,
    container: usize,
}

fn le_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn le_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn parse_format(body: &[u8]) -> Result<Format, ReadError> {
    if body.len() < 16 {
        return error("DAMAGED WAV HEADER");
    }
    let mut tag = le_u16(body, 0);
    if tag == 0xFFFE && body.len() >= 26 {
        tag = le_u16(body, 24);
    }
    let channels = usize::from(le_u16(body, 2));
    let rate = le_u32(body, 4);
    let block = usize::from(le_u16(body, 12));
    let bits = le_u16(body, 14);
    let encoding = match (tag, bits) {
        (1, 8 | 16 | 24 | 32) => Encoding::Integer,
        (3, 32 | 64) => Encoding::Float,
        _ => return error("UNSUPPORTED WAV ENCODING"),
    };
    if channels == 0 || block == 0 || block % channels != 0 {
        return error("DAMAGED WAV HEADER");
    }
    let container = block / channels;
    if container * 8 < usize::from(bits) || container > 8 {
        return error("DAMAGED WAV HEADER");
    }
    if rate < 16_000 {
        return error("RECORDING RATE TOO LOW FOR TAPE DATA");
    }
    Ok(Format {
        encoding,
        channels,
        rate,
        container,
    })
}

fn sample(format: &Format, bytes: &[u8]) -> f32 {
    match (format.encoding, format.container) {
        (Encoding::Float, 4) => f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        (Encoding::Float, _) => {
            let mut raw = [0; 8];
            raw.copy_from_slice(&bytes[..8]);
            f64::from_le_bytes(raw) as f32
        }
        (Encoding::Integer, 1) => (f32::from(bytes[0]) - 128.0) / 128.0,
        (Encoding::Integer, width) => {
            // Little-endian, sign-extended from the container's top byte.
            let mut value: i64 = 0;
            for (shift, byte) in bytes[..width].iter().enumerate() {
                value |= i64::from(*byte) << (8 * shift);
            }
            let bits = 8 * width as u32;
            let value = (value << (64 - bits)) >> (64 - bits);
            (value as f64 / (1_i64 << (bits - 1)) as f64) as f32
        }
    }
}

fn finite(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

/// Reads a WAV recording, keeping the channel the tape is loudest on.
pub fn parse_wav(bytes: &[u8]) -> Result<Wave, ReadError> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return error("NOT A WAV RECORDING");
    }
    let (mut format, mut data) = (None, None);
    let mut offset = 12;
    while offset + 8 <= bytes.len() && (format.is_none() || data.is_none()) {
        let size = le_u32(bytes, offset + 4) as usize;
        let start = offset + 8;
        // A recorder stopped mid-write leaves a size past the end.
        let end = start.saturating_add(size).min(bytes.len());
        match &bytes[offset..offset + 4] {
            b"fmt " => format = Some(parse_format(&bytes[start..end])?),
            b"data" => data = Some(&bytes[start..end]),
            _ => {}
        }
        offset = start.saturating_add(size).saturating_add(size & 1);
    }
    let (Some(format), Some(data)) = (format, data) else {
        return error("DAMAGED WAV RECORDING");
    };
    let frame = format.container * format.channels;
    let frames = data.len() / frame;
    // The channel that carries the tape: a stereo deck may have recorded
    // only one, the other holding nothing but hiss.
    let channel = (0..format.channels)
        .map(|channel| {
            let (mut sum, mut squares) = (0.0_f64, 0.0_f64);
            for index in (0..frames).step_by(7) {
                let at = index * frame + channel * format.container;
                let value = f64::from(finite(sample(&format, &data[at..])));
                sum += value;
                squares += value * value;
            }
            let count = frames.div_ceil(7).max(1) as f64;
            (channel, squares / count - (sum / count).powi(2))
        })
        .max_by(|left, right| left.1.total_cmp(&right.1))
        .map_or(0, |(channel, _)| channel);
    let samples = (0..frames)
        .map(|index| {
            finite(sample(
                &format,
                &data[index * frame + channel * format.container..],
            ))
        })
        .collect();
    Ok(Wave {
        rate: format.rate,
        samples,
    })
}

/// Where the recording crosses zero on each of its transitions, in samples,
/// with DC and hum filtered out and hysteresis against hiss.
fn edges(wave: &Wave) -> Result<Vec<f64>, ReadError> {
    let rate = f64::from(wave.rate);
    let rc = 1.0 / (2.0 * core::f64::consts::PI * HIGH_PASS_HZ);
    let alpha = (rc / (rc + 1.0 / rate)) as f32;
    let mut filtered = Vec::with_capacity(wave.samples.len());
    let (mut previous_in, mut previous_out) = (wave.samples.first().copied().unwrap_or(0.0), 0.0);
    for &value in &wave.samples {
        previous_out = alpha * (previous_out + value - previous_in);
        previous_in = value;
        filtered.push(previous_out);
    }
    // The peak level, as the 99th percentile, so a click does not set it.
    let stride = (filtered.len() / 1_000_000).max(1);
    let mut levels: Vec<f32> = filtered
        .iter()
        .step_by(stride)
        .map(|value| value.abs())
        .collect();
    if levels.is_empty() {
        return error("NOTHING RECORDED");
    }
    let rank = (levels.len() * 99 / 100).min(levels.len() - 1);
    let (_, peak, _) = levels.select_nth_unstable_by(rank, f32::total_cmp);
    let peak = *peak;
    if peak < 1.0e-4 {
        return error("NOTHING RECORDED");
    }
    let threshold = HYSTERESIS * peak;
    let mut edges = Vec::new();
    let mut high = filtered[0] > 0.0;
    let mut last_zero = 0.0;
    for index in 1..filtered.len() {
        let (before, after) = (filtered[index - 1], filtered[index]);
        if (before <= 0.0) != (after <= 0.0) {
            last_zero = (index - 1) as f64 + f64::from(before / (before - after));
        }
        if !high && after > threshold {
            high = true;
            edges.push(last_zero);
        } else if high && after < -threshold {
            high = false;
            edges.push(last_zero);
        }
    }
    Ok(edges)
}

/// The length of a minim in the recording, from its leader's half-cycles:
/// a tape running fast or slow shortens or stretches them all alike.
fn calibrate(intervals: &[f64], rate: u32) -> Option<f64> {
    let nominal = MINIM_SECONDS * f64::from(rate);
    let mut shorts: Vec<f64> = intervals
        .iter()
        .copied()
        .filter(|interval| (0.6 * nominal..1.6 * nominal).contains(interval))
        .collect();
    if shorts.len() < 200 {
        return None;
    }
    let middle = shorts.len() / 2;
    let (_, median, _) = shorts.select_nth_unstable_by(middle, f64::total_cmp);
    Some(*median)
}

enum Phase {
    /// Looking for the leader: runs of short half-cycles, a long one ending
    /// each bit.
    Seeking {
        shorts: usize,
    },
    /// In the leader, counting its ones, waiting for the start bit.
    Leader {
        ones: usize,
    },
    Data,
}

struct Bits {
    bits: Vec<bool>,
    /// Bits whose half-cycles did not add up as the Prophet writes them.
    damaged: Vec<usize>,
}

/// The bits after the start bit, framed as the Prophet frames them.
fn bits(intervals: &[f64], mut minim: f64, limit: usize) -> Bits {
    let mut phase = Phase::Seeking { shorts: 0 };
    let (mut units, mut count) = (0_u32, 0_u32);
    let mut out = Bits {
        bits: Vec::new(),
        damaged: Vec::new(),
    };
    for &interval in intervals {
        let length = (interval / minim).round().clamp(1.0, 8.0) as u32;
        // The tape drifts as it plays: follow it on every clean half-cycle.
        match length {
            1 => minim += 0.01 * (interval - minim),
            4 => minim += 0.01 * (interval / 4.0 - minim),
            _ => {}
        }
        if let Phase::Seeking { shorts } = &mut phase {
            match length {
                1 => *shorts += 1,
                4 if *shorts >= 8 => {
                    phase = Phase::Leader { ones: 0 };
                    (units, count) = (0, 0);
                }
                _ => *shorts = 0,
            }
            continue;
        }
        units += length;
        count += 1;
        if units < BIT_MINIMS {
            continue;
        }
        let bit = count > 7;
        let clean = units == BIT_MINIMS && (count == 13 || count == 4);
        (units, count) = (0, 0);
        match &mut phase {
            Phase::Leader { ones } => {
                if !clean {
                    phase = Phase::Seeking { shorts: 0 };
                } else if bit {
                    *ones += 1;
                } else if *ones >= LEADER_MINIMUM {
                    phase = Phase::Data;
                } else {
                    phase = Phase::Seeking { shorts: 0 };
                }
            }
            Phase::Data => {
                if !clean {
                    out.damaged.push(out.bits.len());
                }
                out.bits.push(bit);
                if out.bits.len() == limit {
                    return out;
                }
            }
            Phase::Seeking { .. } => {}
        }
    }
    // A recording may stop on the checksum's last half-cycle: it has no
    // closing edge, but its edges so far already tell the bit.
    if matches!(phase, Phase::Data) && units >= BIT_MINIMS / 2 {
        out.bits.push(count > 7);
    }
    out
}

fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0_u8, |sum, byte| sum.wrapping_add(*byte))
}

fn pack(bits: &[bool]) -> Vec<u8> {
    bits.as_chunks::<8>()
        .0
        .iter()
        .map(|byte| {
            byte.iter()
                .fold(0_u8, |value, bit| value << 1 | u8::from(*bit))
        })
        .collect()
}

/// Reads the programs off a tape recording.
pub fn read(bytes: &[u8]) -> Result<Reading, ReadError> {
    let wave = parse_wav(bytes)?;
    let edges = edges(&wave)?;
    let intervals: Vec<f64> = edges.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let Some(minim) = calibrate(&intervals, wave.rate) else {
        return error("NO PROPHET-5 TAPE SIGNAL");
    };
    let file_bytes = FILE_PROGRAMS * PROGRAM_BYTES;
    let bank_bytes = BANK_PROGRAMS * PROGRAM_BYTES;
    let read = bits(&intervals, minim, (file_bytes + 1) * 8);
    if read.bits.is_empty() {
        return error("NO PROPHET-5 TAPE SIGNAL");
    }
    let bytes = pack(&read.bits);
    let verified =
        |length: usize| bytes.len() > length && checksum(&bytes[..length]) == bytes[length];
    let (length, checked) = if verified(file_bytes) {
        (file_bytes, true)
    } else if verified(bank_bytes) {
        (bank_bytes, true)
    } else if bytes.len() >= file_bytes {
        (file_bytes, false)
    } else if bytes.len() >= bank_bytes {
        (bank_bytes, false)
    } else {
        return error("THE TAPE ENDS BEFORE ITS PROGRAMS DO");
    };
    let damaged = read
        .damaged
        .iter()
        .filter(|bit| **bit < (length + 1) * 8)
        .count();
    let warning = match (checked, damaged) {
        (true, 0) => None,
        (true, _) => Some(format!("{damaged} UNCLEAR BITS, CHECKSUM GOOD")),
        (false, 0) => Some("CHECKSUM ERROR: PROGRAMS MAY BE DAMAGED".to_owned()),
        (false, _) => Some(format!(
            "CHECKSUM ERROR, {damaged} DAMAGED BITS: PROGRAMS MAY BE DAMAGED"
        )),
    };
    let bank = length == bank_bytes;
    let programs = bytes[..length]
        .as_chunks::<PROGRAM_BYTES>()
        .0
        .iter()
        .enumerate()
        .map(|(index, chunk)| {
            let memory = *chunk;
            Program {
                place: if bank {
                    format!("B-{}", index + 1)
                } else {
                    bank_place(index)
                },
                name: None,
                memory,
            }
        })
        .collect();
    Ok(Reading {
        medium: Medium::Tape,
        programs,
        warning,
    })
}

/// The bits a tape carries: leader, start bit, data, checksum, tail.
fn tape_bits(data: &[u8]) -> Vec<bool> {
    let mut bits = vec![true; LEADER_BITS];
    bits.push(false);
    for byte in data.iter().chain([&checksum(data)]) {
        bits.extend((0..8).rev().map(|shift| byte >> shift & 1 == 1));
    }
    bits.extend([true; TAIL_BITS]);
    bits
}

/// Records programs as the Prophet would to tape: up to eight as one bank,
/// more as a whole file of forty, the places left filled with a plain
/// program. Only the first forty are written.
pub fn write(programs: &[Memory]) -> Vec<u8> {
    let count = if programs.len() <= BANK_PROGRAMS {
        BANK_PROGRAMS
    } else {
        FILE_PROGRAMS
    };
    let mut data = Vec::with_capacity(count * PROGRAM_BYTES);
    for index in 0..count {
        data.extend(programs.get(index).unwrap_or(&INIT_PROGRAM));
    }
    let rate = f64::from(SAMPLE_RATE);
    let minim = MINIM_SECONDS * rate;
    let quiet = QUIET_SECONDS * rate;
    // Half-cycle lengths, in minims.
    let mut lengths = Vec::new();
    for bit in tape_bits(&data) {
        if bit {
            lengths.extend([1_u32; 12]);
            lengths.push(4);
        } else {
            lengths.extend([4_u32; 4]);
        }
    }
    let total: u32 = lengths.iter().sum();
    let samples = (2.0 * quiet + f64::from(total) * minim).ceil() as usize;
    // Each half-cycle adds its level to the samples it covers, in
    // proportion: the edges fall between samples, not on them.
    let mut signal = vec![0.0_f64; samples];
    let mut start = quiet;
    let mut level = LEVEL;
    for length in lengths {
        let end = start + f64::from(length) * minim;
        let first = start.floor() as usize;
        let last = (end.ceil() as usize).min(samples);
        for (index, value) in signal.iter_mut().enumerate().take(last).skip(first) {
            let from = start.max(index as f64);
            let to = end.min(index as f64 + 1.0);
            if to > from {
                *value += level * (to - from);
            }
        }
        start = end;
        level = -level;
    }
    wav_16(SAMPLE_RATE, &signal)
}

/// A mono, sixteen-bit PCM WAV file.
pub fn wav_16(rate: u32, signal: &[f64]) -> Vec<u8> {
    let data = (signal.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + signal.len() * 2);
    bytes.extend(b"RIFF");
    bytes.extend((36 + data).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * 2).to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(data.to_le_bytes());
    for value in signal {
        let value = (value.clamp(-1.0, 1.0) * 32_767.0).round() as i16;
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn programs(count: usize, seed: u32) -> Vec<Memory> {
        let mut state = seed;
        (0..count)
            .map(|_| {
                let mut memory = [0; PROGRAM_BYTES];
                for byte in &mut memory {
                    state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    *byte = (state >> 24) as u8;
                }
                memory
            })
            .collect()
    }

    fn samples(bytes: &[u8]) -> Vec<f64> {
        parse_wav(bytes)
            .unwrap()
            .samples
            .iter()
            .map(|value| f64::from(*value))
            .collect()
    }

    /// A WAV in any format, channels interleaved.
    fn wav(rate: u32, tag: u16, bits: u16, channels: &[Vec<f64>]) -> Vec<u8> {
        let width = usize::from(bits / 8);
        let frames = channels[0].len();
        let mut data = Vec::new();
        for frame in 0..frames {
            for channel in channels {
                let value = channel[frame].clamp(-1.0, 1.0);
                match (tag, bits) {
                    (3, 32) => data.extend((value as f32).to_le_bytes()),
                    (3, 64) => data.extend(value.to_le_bytes()),
                    (1, 8) => data.push((value * 127.0 + 128.0).round() as u8),
                    _ => {
                        let scaled = (value * ((1_i64 << (bits - 1)) - 1) as f64).round() as i64;
                        data.extend(&scaled.to_le_bytes()[..width]);
                    }
                }
            }
        }
        let block = (width * channels.len()) as u16;
        let mut bytes = Vec::new();
        bytes.extend(b"RIFF");
        bytes.extend((36 + data.len() as u32 + 10).to_le_bytes());
        bytes.extend(b"WAVE");
        // A chunk the reader must step over, odd-sized.
        bytes.extend(b"LIST");
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([0, 0]);
        bytes.extend(b"fmt ");
        bytes.extend(16_u32.to_le_bytes());
        bytes.extend(tag.to_le_bytes());
        bytes.extend((channels.len() as u16).to_le_bytes());
        bytes.extend(rate.to_le_bytes());
        bytes.extend((rate * u32::from(block)).to_le_bytes());
        bytes.extend(block.to_le_bytes());
        bytes.extend(bits.to_le_bytes());
        bytes.extend(b"data");
        bytes.extend((data.len() as u32).to_le_bytes());
        bytes.extend(data);
        bytes
    }

    #[test]
    fn a_file_round_trips_through_tape() {
        let written = programs(40, 7);
        let reading = read(&write(&written)).unwrap();
        assert_eq!(reading.medium, Medium::Tape);
        assert_eq!(reading.warning, None);
        let memories: Vec<Memory> = reading.programs.iter().map(|p| p.memory).collect();
        assert_eq!(memories, written);
        assert_eq!(reading.programs[0].place, "1-1");
        assert_eq!(reading.programs[39].place, "5-8");
    }

    #[test]
    fn a_few_programs_are_written_as_a_bank() {
        let written = programs(3, 11);
        let reading = read(&write(&written)).unwrap();
        assert_eq!(reading.programs.len(), BANK_PROGRAMS);
        assert_eq!(reading.warning, None);
        assert_eq!(reading.programs[2].memory, written[2]);
        assert_eq!(reading.programs[3].memory, INIT_PROGRAM);
        assert_eq!(reading.programs[0].place, "B-1");
    }

    #[test]
    fn written_tapes_have_the_factory_tapes_timing() {
        let bytes = write(&programs(40, 3));
        let wave = parse_wav(&bytes).unwrap();
        let seconds = wave.samples.len() as f64 / f64::from(wave.rate);
        // Leader and data as long as the factory tapes, plus the quiet.
        assert!((46.0..48.5).contains(&seconds), "{seconds}");
    }

    /// A tape through a poor cassette deck: slow and drifting, quiet, off
    /// centre, humming and hissing, inverted, on one channel of two.
    #[test]
    fn a_worn_recording_still_reads() {
        let written = programs(40, 23);
        let clean = samples(&write(&written));
        let rate = 48_000_u32;
        let mut noise = 0x1234_5678_u32;
        let mut hiss = || {
            noise = noise.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            f64::from(noise >> 8) / f64::from(1_u32 << 24) - 0.5
        };
        let mut signal = Vec::new();
        let mut position = 0.0_f64;
        let mut time = 0.0_f64;
        while (position as usize) + 1 < clean.len() {
            let index = position as usize;
            let fraction = position - index as f64;
            let value = clean[index] * (1.0 - fraction) + clean[index + 1] * fraction;
            let hum = 0.01 * (2.0 * core::f64::consts::PI * 50.0 * time).sin();
            signal.push(-0.05 * value + 0.2 + hum + 0.012 * hiss());
            // 6 % slow, wandering by 1 % over a few seconds.
            let speed = 0.94 + 0.01 * (time * 0.7).sin();
            position += speed * f64::from(SAMPLE_RATE) / f64::from(rate);
            time += 1.0 / f64::from(rate);
        }
        let dead: Vec<f64> = (0..signal.len()).map(|_| 0.004 * hiss()).collect();
        let bytes = wav(rate, 1, 24, &[dead, signal]);
        let reading = read(&bytes).unwrap();
        assert_eq!(reading.warning, None);
        let memories: Vec<Memory> = reading.programs.iter().map(|p| p.memory).collect();
        assert_eq!(memories, written);
    }

    #[test]
    fn every_wav_encoding_reads() {
        let written = programs(8, 5);
        let clean = samples(&write(&written));
        for (tag, bits) in [(1, 8), (1, 16), (1, 32), (3, 32), (3, 64)] {
            let bytes = wav(SAMPLE_RATE, tag, bits, std::slice::from_ref(&clean));
            let reading = read(&bytes).unwrap_or_else(|error| panic!("{tag}/{bits}: {error}"));
            assert_eq!(reading.programs[7].memory, written[7], "{tag}/{bits}");
        }
    }

    #[test]
    fn a_damaged_tape_says_so() {
        let written = programs(40, 13);
        let mut bits = tape_bits(&{
            let mut data = Vec::new();
            for memory in &written {
                data.extend(memory);
            }
            data
        });
        // One data bit flipped: the Prophet's checksum catches it.
        bits[LEADER_BITS + 1 + 100] ^= true;
        let rate = f64::from(SAMPLE_RATE);
        let minim = MINIM_SECONDS * rate;
        let mut signal = Vec::new();
        let mut level = LEVEL;
        let mut clock = 0.0;
        for bit in bits {
            let lengths: &[u32] = if bit { &[1; 12] } else { &[4; 4] };
            for &length in lengths.iter().chain(if bit { &[4][..] } else { &[][..] }) {
                clock += f64::from(length) * minim;
                while (signal.len() as f64) < clock {
                    signal.push(level);
                }
                level = -level;
            }
        }
        let reading = read(&wav_16(SAMPLE_RATE, &signal)).unwrap();
        assert_eq!(reading.programs.len(), FILE_PROGRAMS);
        assert!(reading.warning.unwrap().starts_with("CHECKSUM ERROR"));
    }

    #[test]
    fn noise_is_not_a_tape() {
        let mut state = 99_u32;
        let signal: Vec<f64> = (0..200_000)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                f64::from(state >> 8) / f64::from(1_u32 << 24) - 0.5
            })
            .collect();
        assert!(read(&wav_16(SAMPLE_RATE, &signal)).is_err());
        assert!(read(&wav_16(SAMPLE_RATE, &vec![0.0; 50_000])).is_err());
        assert!(read(b"RIFF\0\0\0\0WAVE").is_err());
    }

    /// The factory tapes Sequential shipped with the Rev 3, when a copy is
    /// at hand (references-local/ is not part of the repository).
    #[test]
    fn the_factory_tapes_read_and_verify() {
        let folder = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../references-local/p5-transfer"
        );
        for name in ["FACT1R3.WAV", "FACT2R3.WAV", "FACT3R3.WAV"] {
            let Ok(bytes) = std::fs::read(format!("{folder}/{name}")) else {
                continue;
            };
            let reading = read(&bytes).unwrap();
            assert_eq!(reading.programs.len(), FILE_PROGRAMS, "{name}");
            assert_eq!(reading.warning, None, "{name}");
        }
    }
}
