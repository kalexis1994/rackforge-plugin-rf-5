#![cfg_attr(target_arch = "wasm32", no_std)]

#[cfg(target_arch = "wasm32")]
use rackforge_plugin_sdk::export_processor;
use rackforge_plugin_sdk::{MidiEvent, ParameterEvent, Processor};
use rf_5_dsp::Engine;

const MAX_OUTPUT_CHANNELS: usize = 2;

#[derive(Default)]
pub struct Rf5Processor {
    engine: Engine,
}

impl Processor for Rf5Processor {
    fn prepare(
        &mut self,
        sample_rate: f64,
        _maximum_frames: u32,
        input_channels: u32,
        output_channels: u32,
    ) -> bool {
        input_channels == 0
            && output_channels > 0
            && output_channels <= MAX_OUTPUT_CHANNELS as u32
            && self.engine.prepare(sample_rate)
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

    fn process(
        &mut self,
        _input: &[f32],
        output: &mut [f32],
        midi: &[MidiEvent],
        parameters: &[ParameterEvent],
        frames: u32,
        _input_channels: u32,
        output_channels: u32,
    ) {
        let channels = output_channels as usize;
        let mut midi_index = 0;
        let mut parameter_index = 0;

        for frame in 0..frames as usize {
            while let Some(event) = midi.get(midi_index) {
                if event.frame as usize != frame {
                    break;
                }
                self.engine.handle_midi(event.data);
                midi_index += 1;
            }
            while let Some(event) = parameters.get(parameter_index) {
                if event.frame as usize != frame {
                    break;
                }
                let _ = self.engine.set_parameter(event.index, event.value);
                parameter_index += 1;
            }

            let sample = self.engine.next_sample();
            for channel in 0..channels {
                output[frame * channels + channel] = sample;
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
export_processor!(
    Rf5Processor,
    max_frames = 4096,
    max_input_channels = 0,
    max_output_channels = 2,
    max_midi_events = 256,
    max_parameter_events = 256,
    max_transfer_bytes = 4096
);

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(all(target_arch = "wasm32", not(test)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_renders_midi_and_sample_accurate_automation() {
        let mut processor = Rf5Processor::default();
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
        let mut processor = Rf5Processor::default();
        assert!(!processor.prepare(48_000.0, 256, 1, 2));
        assert!(!processor.prepare(48_000.0, 256, 0, 3));
    }
}
