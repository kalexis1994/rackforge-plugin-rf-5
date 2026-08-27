#![cfg_attr(target_arch = "wasm32", no_std)]

use core::{mem, slice};

use rackforge_plugin_sdk::{
    BlockContext, ParallelProcessor, PlanWriter, UnitContext, UnitMix, export_parallel_processor,
};
use rf_5_dsp::{
    CommonVoiceFrame, Engine, ParallelVoiceUnit, PreparedSample, VOICE_COUNT, VoiceCalibration,
    VoiceCommand, VoiceCommandKind,
};

const MAX_FRAMES: usize = 4096;
const MAX_OUTPUT_CHANNELS: usize = 2;
const MAX_MIDI_EVENTS: usize = 256;
const MAX_PARAMETER_EVENTS: usize = 256;
const MAX_COMMANDS_PER_UNIT: usize = 800;
const WIRE_VERSION: u32 = 1;
const SHARED_MAGIC: u32 = u32::from_le_bytes(*b"RFSH");
const DISPATCH_MAGIC: u32 = u32::from_le_bytes(*b"RFDU");

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
struct SharedHeader {
    magic: u32,
    version: u32,
    frames: u32,
    sample_rate_bits: u32,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
struct DispatchHeader {
    magic: u32,
    version: u32,
    frames: u32,
    initial_epoch: u32,
    command_count: u32,
    reserved: u32,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
struct WireVoiceCommand {
    frame: u32,
    epoch: u32,
    kind: u8,
    channel: u8,
    note: u8,
    velocity: u8,
    unit: u8,
    reserved: [u8; 3],
}

impl WireVoiceCommand {
    fn from_command(frame: u32, command: VoiceCommand) -> Self {
        Self {
            frame,
            epoch: command.epoch,
            kind: command.kind as u8,
            channel: command.channel,
            note: command.note,
            velocity: command.velocity,
            unit: command.unit,
            reserved: [0; 3],
        }
    }

    fn decode(self) -> Option<VoiceCommand> {
        let kind = match self.kind {
            0 => VoiceCommandKind::Reset,
            1 => VoiceCommandKind::Start,
            2 => VoiceCommandKind::Retune,
            3 => VoiceCommandKind::Release,
            _ => return None,
        };
        Some(VoiceCommand {
            unit: self.unit,
            kind,
            channel: self.channel,
            note: self.note,
            velocity: self.velocity,
            reserved: [0; 3],
            epoch: self.epoch,
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct EndFrame {
    a440: f32,
    master_volume: f32,
    tuning: bool,
}

const SHARED_HEADER_BYTES: usize = mem::size_of::<SharedHeader>();
const COMMON_FRAME_BYTES: usize = mem::size_of::<CommonVoiceFrame>();
const SHARED_CAPACITY: usize = SHARED_HEADER_BYTES + MAX_FRAMES * COMMON_FRAME_BYTES;
const DISPATCH_HEADER_BYTES: usize = mem::size_of::<DispatchHeader>();
const CALIBRATION_BYTES: usize = MAX_FRAMES * mem::size_of::<VoiceCalibration>();
const COMMAND_BYTES: usize = MAX_COMMANDS_PER_UNIT * mem::size_of::<WireVoiceCommand>();
const DISPATCH_STRIDE: usize = DISPATCH_HEADER_BYTES + CALIBRATION_BYTES + COMMAND_BYTES;

const _: () = assert!(SHARED_CAPACITY.is_multiple_of(8));
const _: () = assert!(DISPATCH_STRIDE.is_multiple_of(8));
const _: () =
    assert!(MAX_COMMANDS_PER_UNIT >= MAX_MIDI_EVENTS + MAX_PARAMETER_EVENTS * 2 + VOICE_COUNT);

pub struct Rf5Processor {
    engine: Engine,
    end_frames: [EndFrame; MAX_FRAMES],
    calibrations: [[VoiceCalibration; MAX_FRAMES]; VOICE_COUNT],
    commands: [[WireVoiceCommand; MAX_COMMANDS_PER_UNIT]; VOICE_COUNT],
    command_counts: [usize; VOICE_COUNT],
    dispatch_scratch: [u8; DISPATCH_STRIDE],
    command_overflow: bool,
}

impl Default for Rf5Processor {
    fn default() -> Self {
        Self {
            engine: Engine::default(),
            end_frames: [EndFrame::default(); MAX_FRAMES],
            calibrations: [[VoiceCalibration::default(); MAX_FRAMES]; VOICE_COUNT],
            commands: [[WireVoiceCommand::default(); MAX_COMMANDS_PER_UNIT]; VOICE_COUNT],
            command_counts: [0; VOICE_COUNT],
            dispatch_scratch: [0; DISPATCH_STRIDE],
            command_overflow: false,
        }
    }
}

impl Rf5Processor {
    fn capture_commands(&mut self, frame: u32) {
        let mut pending = [VoiceCommand::default(); VOICE_COUNT * 4];
        let count = self.engine.drain_voice_commands(&mut pending);
        for command in pending[..count].iter().copied() {
            let unit = command.unit as usize;
            let Some(destination) = self.commands.get_mut(unit) else {
                self.command_overflow = true;
                continue;
            };
            let index = self.command_counts[unit];
            let Some(slot) = destination.get_mut(index) else {
                self.command_overflow = true;
                continue;
            };
            *slot = WireVoiceCommand::from_command(frame, command);
            self.command_counts[unit] += 1;
        }
    }

    fn write_dispatch(&mut self, unit: usize, frames: usize, initial_epoch: u32) -> usize {
        self.dispatch_scratch.fill(0);
        let command_count = self.command_counts[unit];
        let header = DispatchHeader {
            magic: DISPATCH_MAGIC,
            version: WIRE_VERSION,
            frames: frames as u32,
            initial_epoch,
            command_count: command_count as u32,
            reserved: 0,
        };
        let mut offset = 0;
        write_value(&mut self.dispatch_scratch, &mut offset, &header);
        write_values(
            &mut self.dispatch_scratch,
            &mut offset,
            &self.calibrations[unit][..frames],
        );
        write_values(
            &mut self.dispatch_scratch,
            &mut offset,
            &self.commands[unit][..command_count],
        );
        offset
    }
}

impl ParallelProcessor for Rf5Processor {
    type Unit = ParallelVoiceUnit;

    fn prepare(
        &mut self,
        sample_rate: f64,
        maximum_frames: u32,
        input_channels: u32,
        output_channels: u32,
    ) -> bool {
        input_channels == 0
            && output_channels > 0
            && output_channels <= MAX_OUTPUT_CHANNELS as u32
            && maximum_frames as usize <= MAX_FRAMES
            && self.engine.prepare(sample_rate)
            && {
                self.engine.capture_voice_commands(true);
                true
            }
    }

    fn set_parameter(&mut self, index: u32, value: f64) -> bool {
        self.engine.set_parameter(index, value)
    }

    fn get_parameter(&self, index: u32) -> Option<f64> {
        self.engine.parameter(index)
    }

    fn reset(&mut self) {
        self.engine.reset_voices();
    }

    fn load_preset(&mut self, id: &str) -> bool {
        self.engine.load_program(id)
    }

    fn save_state(&self, destination: &mut [u8]) -> Option<usize> {
        self.engine.save_state(destination)
    }

    fn load_state(&mut self, state: &[u8]) -> bool {
        self.engine.load_state(state)
    }

    fn begin_block(&mut self, context: &BlockContext<'_>, plan: &mut PlanWriter<'_>) {
        let frames = context.frames as usize;
        if frames == 0 || frames > MAX_FRAMES {
            return;
        }
        self.command_counts.fill(0);
        self.command_overflow = false;
        let initial_epoch = self.engine.voice_epoch();
        self.capture_commands(0);

        let shared_header = SharedHeader {
            magic: SHARED_MAGIC,
            version: WIRE_VERSION,
            frames: context.frames,
            sample_rate_bits: self.engine.sample_rate().to_bits(),
        };
        let mut shared_offset = 0;
        write_value(plan.shared_buffer(), &mut shared_offset, &shared_header);

        let mut midi_index = 0;
        let mut parameter_index = 0;
        for frame in 0..frames {
            while let Some(event) = context.midi.get(midi_index) {
                if event.frame as usize != frame {
                    break;
                }
                self.engine.handle_midi(event.data);
                self.capture_commands(frame as u32);
                midi_index += 1;
            }
            while let Some(event) = context.parameters.get(parameter_index) {
                if event.frame as usize != frame {
                    break;
                }
                let _ = self.engine.set_parameter(event.index, event.value);
                self.capture_commands(frame as u32);
                parameter_index += 1;
            }

            let prepared = self.engine.prepare_next_sample();
            self.end_frames[frame] = EndFrame {
                a440: prepared.a440,
                master_volume: prepared.master_volume,
                tuning: prepared.tuning,
            };
            for unit in 0..VOICE_COUNT {
                self.calibrations[unit][frame] = prepared.calibration[unit];
            }
            write_value(plan.shared_buffer(), &mut shared_offset, &prepared.common);
        }

        if !plan.commit_shared(shared_offset) || self.command_overflow {
            return;
        }
        for unit in 0..VOICE_COUNT {
            if !self.engine.voice_initialized(unit) {
                continue;
            }
            let payload_bytes = self.write_dispatch(unit, frames, initial_epoch);
            let activated = plan.activate(unit as u32, &self.dispatch_scratch[..payload_bytes]);
            debug_assert!(activated);
        }
    }

    fn render_unit(
        unit_index: u32,
        unit: &mut Self::Unit,
        payload: &[u8],
        context: &UnitContext<'_>,
        output: &mut [f32],
    ) {
        let channels = context.output_channels as usize;
        let samples = context.frames as usize * channels;
        output[..samples].fill(0.0);
        if channels == 0 || channels > MAX_OUTPUT_CHANNELS {
            return;
        }

        let Some((shared_header, mut shared_offset)) =
            read_value::<SharedHeader>(context.shared, 0)
        else {
            return;
        };
        let Some((dispatch_header, mut calibration_offset)) =
            read_value::<DispatchHeader>(payload, 0)
        else {
            return;
        };
        let frames = context.frames as usize;
        if shared_header.magic != SHARED_MAGIC
            || shared_header.version != WIRE_VERSION
            || shared_header.frames != context.frames
            || dispatch_header.magic != DISPATCH_MAGIC
            || dispatch_header.version != WIRE_VERSION
            || dispatch_header.frames != context.frames
            || dispatch_header.command_count as usize > MAX_COMMANDS_PER_UNIT
        {
            return;
        }
        let sample_rate = f32::from_bits(shared_header.sample_rate_bits);
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return;
        }
        unit.synchronize_epoch(dispatch_header.initial_epoch, unit_index as usize);

        let command_offset = calibration_offset
            .checked_add(frames * mem::size_of::<VoiceCalibration>())
            .unwrap_or(payload.len());
        let mut command_index = 0;
        for frame in 0..frames {
            while command_index < dispatch_header.command_count as usize {
                let offset = command_offset + command_index * mem::size_of::<WireVoiceCommand>();
                let Some((wire, _)) = read_value::<WireVoiceCommand>(payload, offset) else {
                    return;
                };
                if wire.frame as usize != frame {
                    break;
                }
                if wire.unit != unit_index as u8 {
                    return;
                }
                let Some(command) = wire.decode() else {
                    return;
                };
                unit.apply_command(command);
                command_index += 1;
            }
            let Some((common, next_shared)) =
                read_value::<CommonVoiceFrame>(context.shared, shared_offset)
            else {
                return;
            };
            shared_offset = next_shared;
            let Some((calibration, next_calibration)) =
                read_value::<VoiceCalibration>(payload, calibration_offset)
            else {
                return;
            };
            calibration_offset = next_calibration;
            let sample = unit.next(sample_rate, common, calibration);
            for channel in 0..channels {
                output[frame * channels + channel] = sample;
            }
        }
    }

    fn end_block(
        &mut self,
        mix: &UnitMix<'_>,
        output: &mut [f32],
        frames: u32,
        output_channels: u32,
    ) {
        let channels = output_channels as usize;
        let frames = frames as usize;
        for frame in 0..frames {
            let mut voice_sum = 0.0;
            for unit in mix.active_units() {
                voice_sum += mix.slot(unit)[frame * channels];
            }
            let end = self.end_frames[frame];
            let sample = self.engine.finish_prepared_sample(
                PreparedSample {
                    a440: end.a440,
                    master_volume: end.master_volume,
                    tuning: end.tuning,
                    ..PreparedSample::default()
                },
                voice_sum,
            );
            for channel in 0..channels {
                output[frame * channels + channel] = sample;
            }
        }
    }
}

fn write_value<T: Copy>(destination: &mut [u8], offset: &mut usize, value: &T) {
    let bytes = unsafe {
        // SAFETY: `value` is a fully initialized plain-data wire structure;
        // the payload is private to this exact component build.
        slice::from_raw_parts((value as *const T).cast::<u8>(), mem::size_of::<T>())
    };
    destination[*offset..*offset + bytes.len()].copy_from_slice(bytes);
    *offset += bytes.len();
}

fn write_values<T: Copy>(destination: &mut [u8], offset: &mut usize, values: &[T]) {
    let byte_count = mem::size_of_val(values);
    let bytes = unsafe {
        // SAFETY: same private wire format as `write_value`; the slice is
        // initialized and remains alive for the duration of this copy.
        slice::from_raw_parts(values.as_ptr().cast::<u8>(), byte_count)
    };
    destination[*offset..*offset + byte_count].copy_from_slice(bytes);
    *offset += byte_count;
}

fn read_value<T: Copy>(source: &[u8], offset: usize) -> Option<(T, usize)> {
    let end = offset.checked_add(mem::size_of::<T>())?;
    let bytes = source.get(offset..end)?;
    let value = unsafe {
        // SAFETY: all decoded wire types contain only integer/f32 fields and
        // were written by the same component. Unaligned reads are explicit.
        core::ptr::read_unaligned(bytes.as_ptr().cast::<T>())
    };
    Some((value, end))
}

export_parallel_processor!(
    Rf5Processor,
    max_units = VOICE_COUNT,
    dispatch_stride = DISPATCH_STRIDE,
    shared_capacity = SHARED_CAPACITY,
    max_frames = MAX_FRAMES,
    max_input_channels = 0,
    max_output_channels = MAX_OUTPUT_CHANNELS,
    max_midi_events = MAX_MIDI_EVENTS,
    max_parameter_events = MAX_PARAMETER_EVENTS,
    max_transfer_bytes = 4096
);

#[cfg(all(target_arch = "wasm32", not(test)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rackforge_plugin_sdk::{MidiEvent, ParameterEvent, Processor};
    use std::sync::{Mutex, MutexGuard};

    const TEST_FRAMES: u32 = 256;
    static PARALLEL_EXPORT_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn parallel_export_test_guard() -> MutexGuard<'static, ()> {
        PARALLEL_EXPORT_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn render_reference(
        engine: &mut Engine,
        midi: &[MidiEvent],
        parameters: &[ParameterEvent],
    ) -> [f32; TEST_FRAMES as usize * 2] {
        let mut output = [0.0; TEST_FRAMES as usize * 2];
        let mut midi_index = 0;
        let mut parameter_index = 0;
        for frame in 0..TEST_FRAMES as usize {
            while let Some(event) = midi.get(midi_index) {
                if event.frame as usize != frame {
                    break;
                }
                engine.handle_midi(event.data);
                midi_index += 1;
            }
            while let Some(event) = parameters.get(parameter_index) {
                if event.frame as usize != frame {
                    break;
                }
                assert!(engine.set_parameter(event.index, event.value));
                parameter_index += 1;
            }
            let sample = engine.next_sample();
            output[frame * 2] = sample;
            output[frame * 2 + 1] = sample;
        }
        output
    }

    fn midi(frame: u32, data: [u8; 3]) -> MidiEvent {
        MidiEvent {
            frame,
            data,
            length: 3,
        }
    }

    #[test]
    fn adapter_renders_midi_and_sample_accurate_automation() {
        let _guard = parallel_export_test_guard();
        let mut processor = RackForgeParallelExport::default();
        assert!(processor.prepare(48_000.0, 256, 0, 2));
        let mut output = [0.0_f32; 512];
        processor.process(
            &[],
            &mut output,
            &[MidiEvent {
                frame: 0,
                data: [0x90, 69, 110],
                length: 3,
            }],
            &[ParameterEvent {
                frame: 128,
                index: 0,
                value: 0.25,
            }],
            256,
            0,
            2,
        );
        assert!(output.iter().any(|sample| sample.abs() > 0.001));
        assert_eq!(processor.get_parameter(0), Some(0.25_f32 as f64));
    }

    #[test]
    fn adapter_rejects_audio_inputs_and_excess_outputs() {
        let _guard = parallel_export_test_guard();
        let mut processor = RackForgeParallelExport::default();
        assert!(!processor.prepare(48_000.0, 256, 1, 2));
        assert!(!processor.prepare(48_000.0, 256, 0, 3));
    }

    #[test]
    fn composed_parallel_contract_is_bit_exact_with_the_sequential_engine() {
        let _guard = parallel_export_test_guard();
        let mut reference = Engine::default();
        let mut parallel = RackForgeParallelExport::default();
        assert!(reference.prepare(48_000.0));
        assert!(parallel.prepare(48_000.0, TEST_FRAMES, 0, 2));
        assert!(reference.load_program("original-34-high-strings"));
        assert!(parallel.load_preset("original-34-high-strings"));

        let scripts: [(&[MidiEvent], &[ParameterEvent]); 10] = [
            (
                &[
                    midi(3, [0x90, 48, 110]),
                    midi(3, [0x90, 55, 100]),
                    midi(3, [0x90, 60, 96]),
                    midi(97, [0xb0, 1, 87]),
                ],
                &[],
            ),
            (
                &[
                    midi(11, [0x90, 64, 103]),
                    midi(29, [0x90, 67, 101]),
                    midi(47, [0x90, 72, 99]),
                ],
                &[],
            ),
            (
                &[midi(17, [0xe0, 0, 112]), midi(123, [0xb0, 64, 127])],
                &[ParameterEvent {
                    frame: 61,
                    index: 4,
                    value: 0.71,
                }],
            ),
            (
                &[
                    midi(9, [0x80, 48, 0]),
                    midi(10, [0x80, 55, 0]),
                    midi(11, [0x80, 60, 0]),
                ],
                &[],
            ),
            (
                &[
                    midi(12, [0x80, 64, 0]),
                    midi(13, [0x80, 67, 0]),
                    midi(14, [0x80, 72, 0]),
                ],
                &[],
            ),
            (&[midi(44, [0xb0, 64, 0])], &[]),
            (
                &[midi(18, [0x90, 64, 120])],
                &[ParameterEvent {
                    frame: 17,
                    index: 46,
                    value: 1.0,
                }],
            ),
            (
                &[midi(71, [0x90, 52, 118]), midi(149, [0xb0, 1, 127])],
                &[ParameterEvent {
                    frame: 70,
                    index: 45,
                    value: 0.63,
                }],
            ),
            (&[midi(37, [0x80, 52, 0]), midi(38, [0x80, 64, 0])], &[]),
            (&[midi(211, [0xb0, 123, 0])], &[]),
        ];

        for (block, (midi_events, parameter_events)) in scripts.into_iter().enumerate() {
            let expected = render_reference(&mut reference, midi_events, parameter_events);
            let mut actual = [0.0; TEST_FRAMES as usize * 2];
            parallel.process(
                &[],
                &mut actual,
                midi_events,
                parameter_events,
                TEST_FRAMES,
                0,
                2,
            );
            assert_eq!(
                actual, expected,
                "parallel contract diverged at block {block}"
            );
        }

        assert!(reference.load_program("original-17-sync-i"));
        assert!(parallel.load_preset("original-17-sync-i"));
        let program_note = [midi(23, [0x90, 57, 104])];
        let expected = render_reference(&mut reference, &program_note, &[]);
        let mut actual = [0.0; TEST_FRAMES as usize * 2];
        parallel.process(&[], &mut actual, &program_note, &[], TEST_FRAMES, 0, 2);
        assert_eq!(
            actual, expected,
            "program load did not reach every unit exactly"
        );

        reference.reset_voices();
        parallel.reset();
        let expected = render_reference(&mut reference, &[], &[]);
        parallel.process(&[], &mut actual, &[], &[], TEST_FRAMES, 0, 2);
        assert_eq!(actual, expected, "reset did not reach every physical unit");
    }

    #[test]
    fn percussive_originals_retrigger_every_voice_through_the_parallel_contract() {
        let _guard = parallel_export_test_guard();
        for program in ["original-14-percussive-e-piano", "original-16-harpsichord"] {
            let mut reference = Engine::default();
            let mut parallel = RackForgeParallelExport::default();
            assert!(reference.prepare(48_000.0));
            assert!(parallel.prepare(48_000.0, TEST_FRAMES, 0, 2));
            assert!(reference.load_program(program));
            assert!(parallel.load_preset(program));

            // Let the physical CPU/CV scan and sample-hold network settle the
            // recalled patch before the first key, as a player naturally can.
            for block in 0..375 {
                let expected = render_reference(&mut reference, &[], &[]);
                let mut actual = [0.0; TEST_FRAMES as usize * 2];
                parallel.process(&[], &mut actual, &[], &[], TEST_FRAMES, 0, 2);
                assert_eq!(
                    actual, expected,
                    "{program} diverged during recall block {block}"
                );
            }

            let mut strike_peaks = [0.0_f32; 30];
            for strike in 0..30_u8 {
                let note = 60;
                let channel = strike % VOICE_COUNT as u8;
                let mut strike_peak = 0.0_f32;
                for block in 0..12 {
                    let note_on = [midi(7, [0x90 | channel, note, 112])];
                    let note_off = [midi(193, [0x80 | channel, note, 0])];
                    let events: &[MidiEvent] = match block {
                        0 => &note_on,
                        7 => &note_off,
                        _ => &[],
                    };
                    let expected = render_reference(&mut reference, events, &[]);
                    let mut actual = [0.0; TEST_FRAMES as usize * 2];
                    parallel.process(&[], &mut actual, events, &[], TEST_FRAMES, 0, 2);
                    assert_eq!(
                        actual, expected,
                        "{program} diverged on strike {strike}, block {block}"
                    );
                    if block < 7 {
                        strike_peak = strike_peak.max(
                            actual
                                .chunks_exact(2)
                                .map(|frame| frame[0].abs())
                                .fold(0.0_f32, f32::max),
                        );
                    }
                }
                assert!(
                    strike_peak > 1.0e-4,
                    "{program} lost strike {strike} on physical voice {}",
                    strike as usize % VOICE_COUNT
                );
                strike_peaks[strike as usize] = strike_peak;
            }
            for strike in 10..strike_peaks.len() {
                let settled_reference = strike_peaks[5 + strike % VOICE_COUNT];
                assert!(
                    strike_peaks[strike] >= settled_reference * 0.45,
                    "{program} decayed on physical voice {}: strike {} was {} after {}",
                    strike % VOICE_COUNT,
                    strike,
                    strike_peaks[strike],
                    settled_reference,
                );
            }
        }
    }

    #[test]
    fn harpsichord_physical_voice_operating_points_remain_settled() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.load_program("original-16-harpsichord"));
        for _ in 0..375 * TEST_FRAMES {
            let prepared = engine.prepare_next_sample();
            for unit in 0..VOICE_COUNT {
                let _ = engine.render_prepared_voice(unit, prepared);
            }
        }

        let mut peaks = [0.0_f32; 30];
        for strike in 0..30_u8 {
            let channel = strike % VOICE_COUNT as u8;
            let target = strike as usize % VOICE_COUNT;
            for block in 0..12 {
                for frame in 0..TEST_FRAMES as usize {
                    if block == 0 && frame == 7 {
                        engine.handle_midi([0x90 | channel, 60, 112]);
                    }
                    if block == 7 && frame == 193 {
                        engine.handle_midi([0x80 | channel, 60, 0]);
                    }
                    let prepared = engine.prepare_next_sample();
                    for unit in 0..VOICE_COUNT {
                        let sample = engine.render_prepared_voice(unit, prepared);
                        if unit == target && block < 7 {
                            peaks[strike as usize] = peaks[strike as usize].max(sample.abs());
                        }
                    }
                }
            }
        }
        for strike in 10..peaks.len() {
            let settled_reference = peaks[5 + strike % VOICE_COUNT];
            assert!(
                peaks[strike] >= settled_reference * 0.75,
                "harpsichord voice {} lost its settled operating point: strike {} was {} after {}",
                strike % VOICE_COUNT,
                strike,
                peaks[strike],
                settled_reference,
            );
        }
    }

    #[test]
    fn every_packaged_preset_is_loadable_by_the_engine() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../package/metadata/presets.json")).unwrap();
        let presets = catalog["presets"].as_array().unwrap();
        assert_eq!(presets.len(), 40);

        let mut processor = RackForgeParallelExport::default();
        for preset in presets {
            let id = preset["id"].as_str().unwrap();
            assert!(processor.load_preset(id), "engine rejected preset {id}");
        }

        assert!(!processor.load_preset("baseline-pad"));
        assert!(!processor.load_preset("audition-filter-resonance"));
    }
}
