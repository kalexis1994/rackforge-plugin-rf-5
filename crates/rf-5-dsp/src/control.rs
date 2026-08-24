//! CPU control-loop and sample-and-hold timing candidate.
//!
//! The hardware scans 24 pots and refreshes its CV destinations sequentially.
//! An unchanged loop takes about 6 ms and a changed loop about 11 ms. MIDI
//! performance data remains sample accurate; this scheduler applies only to
//! stored panel controls and switch latches.

use rf_5_contract::{
    PARAMETER_COUNT, Parameter, Settings,
    hardware::{
        ANALOG_POT_COUNT, AnalogPot, CONTROL_LOOP_CHANGED_MICROSECONDS,
        CONTROL_LOOP_IDLE_MICROSECONDS,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct ControlScheduler {
    applied: Settings,
    scan_index: usize,
    samples_until_scan: u32,
    cycle_samples: u32,
    changed_cycle: bool,
}

impl Default for ControlScheduler {
    fn default() -> Self {
        Self {
            applied: Settings::default(),
            scan_index: 0,
            samples_until_scan: 1,
            cycle_samples: ANALOG_POT_COUNT as u32,
            changed_cycle: false,
        }
    }
}

impl ControlScheduler {
    pub fn prepare(&mut self, target: Settings, sample_rate: f32) {
        self.applied = target;
        self.begin_cycle(target, sample_rate);
    }

    pub fn notify_change(&mut self, sample_rate: f32) {
        if self.changed_cycle {
            return;
        }
        let old_cycle = self.cycle_samples.max(1);
        let new_cycle = cycle_samples(sample_rate, CONTROL_LOOP_CHANGED_MICROSECONDS);
        self.samples_until_scan = self
            .samples_until_scan
            .saturating_mul(new_cycle)
            .div_ceil(old_cycle)
            .max(1);
        self.cycle_samples = new_cycle;
        self.changed_cycle = true;
    }

    pub fn next(&mut self, target: Settings, sample_rate: f32) -> Settings {
        if self.samples_until_scan > 1 {
            self.samples_until_scan -= 1;
            return self.with_direct_controls(target);
        }

        let pot = AnalogPot::try_from(self.scan_index as u8).expect("valid scan position");
        copy_parameter(&mut self.applied, target, pot.parameter());
        self.scan_index += 1;

        if self.scan_index == ANALOG_POT_COUNT {
            self.copy_switch_latches(target);
            self.begin_cycle(target, sample_rate);
        } else {
            self.samples_until_scan = scan_spacing(self.cycle_samples, self.scan_index);
        }
        self.with_direct_controls(target)
    }

    fn begin_cycle(&mut self, target: Settings, sample_rate: f32) {
        self.scan_index = 0;
        self.changed_cycle = target != self.applied;
        let microseconds = if self.changed_cycle {
            CONTROL_LOOP_CHANGED_MICROSECONDS
        } else {
            CONTROL_LOOP_IDLE_MICROSECONDS
        };
        self.cycle_samples = cycle_samples(sample_rate, microseconds);
        self.samples_until_scan = scan_spacing(self.cycle_samples, 0);
    }

    fn copy_switch_latches(&mut self, target: Settings) {
        for index in 0..PARAMETER_COUNT as u32 {
            let parameter = Parameter::try_from(index).expect("contiguous parameter contract");
            if !is_scanned_pot(parameter) && !is_direct_control(parameter) {
                copy_parameter(&mut self.applied, target, parameter);
            }
        }
    }

    fn with_direct_controls(&self, target: Settings) -> Settings {
        let mut result = self.applied;
        for parameter in [Parameter::MasterVolume, Parameter::VintageSpread] {
            copy_parameter(&mut result, target, parameter);
        }
        result
    }
}

fn copy_parameter(destination: &mut Settings, source: Settings, parameter: Parameter) {
    let copied = destination.set(parameter as u32, f64::from(source.get(parameter)));
    debug_assert!(copied);
}

fn is_scanned_pot(parameter: Parameter) -> bool {
    (0..ANALOG_POT_COUNT as u8).any(|index| {
        AnalogPot::try_from(index)
            .map(|pot| pot.parameter() == parameter)
            .unwrap_or(false)
    })
}

fn is_direct_control(parameter: Parameter) -> bool {
    matches!(
        parameter,
        Parameter::MasterVolume | Parameter::VintageSpread
    )
}

fn cycle_samples(sample_rate: f32, microseconds: u32) -> u32 {
    (libm::roundf(sample_rate.max(1.0) * microseconds as f32 / 1_000_000.0) as u32)
        .max(ANALOG_POT_COUNT as u32)
}

fn scan_spacing(cycle_samples: u32, scan_index: usize) -> u32 {
    let count = ANALOG_POT_COUNT as u32;
    let base = cycle_samples / count;
    let remainder = cycle_samples % count;
    base + u32::from((scan_index as u32) < remainder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_cycle_is_exactly_six_milliseconds() {
        let settings = Settings::default();
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(settings, 48_000.0);
        assert_eq!(scheduler.cycle_samples, 288);
        for _ in 0..288 {
            assert_eq!(scheduler.next(settings, 48_000.0), settings);
        }
        assert_eq!(scheduler.scan_index, 0);
    }

    #[test]
    fn changed_cycle_is_extended_and_settles_all_controls() {
        let initial = Settings::default();
        let mut target = initial;
        assert!(target.set(Parameter::FilterAttack as u32, 0.8));
        assert!(target.set(Parameter::OscillatorBDetune as u32, 0.1));
        assert!(target.set(Parameter::OscillatorSync as u32, 1.0));
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(initial, 48_000.0);
        scheduler.notify_change(48_000.0);
        assert_eq!(scheduler.cycle_samples, 528);
        for _ in 0..527 {
            let _ = scheduler.next(target, 48_000.0);
        }
        assert_ne!(scheduler.next(target, 48_000.0), initial);
        assert_eq!(scheduler.next(target, 48_000.0), target);
    }

    #[test]
    fn master_volume_is_a_direct_analog_control() {
        let initial = Settings::default();
        let mut target = initial;
        assert!(target.set(Parameter::MasterVolume as u32, 0.1));
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(initial, 48_000.0);
        assert_eq!(
            scheduler
                .next(target, 48_000.0)
                .get(Parameter::MasterVolume),
            0.1
        );
    }
}
