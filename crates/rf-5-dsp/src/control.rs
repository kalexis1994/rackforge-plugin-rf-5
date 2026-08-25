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
        CONTROL_LOOP_IDLE_MICROSECONDS, CONTROL_VOLTAGE_STROBE_ORDER,
        CONTROL_VOLTAGE_STROBE_SLOT_COUNT, PANEL_POT_CONFIRMING_STEPS, analog_pot_code,
    },
};

const CONTROL_SERVICE_STEP_COUNT: usize = ANALOG_POT_COUNT + CONTROL_VOLTAGE_STROBE_SLOT_COUNT;

#[derive(Clone, Copy, Debug)]
pub struct ControlTick {
    pub settings: Settings,
    pub cv_strobe: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlScheduler {
    applied: Settings,
    panel: PanelPotScanner,
    service_index: usize,
    samples_until_service: u32,
    cycle_samples: u32,
    changed_cycle: bool,
}

#[derive(Clone, Copy, Debug)]
struct PanelPotScanner {
    accepted: Settings,
    pending_direction: [i8; PARAMETER_COUNT],
}

impl PanelPotScanner {
    fn new(settings: Settings) -> Self {
        Self {
            accepted: settings,
            pending_direction: [0; PARAMETER_COUNT],
        }
    }

    fn synchronize(&mut self, settings: Settings) {
        self.accepted = settings;
        self.pending_direction = [0; PARAMETER_COUNT];
    }

    fn scan(&mut self, target: Settings, parameter: Parameter) -> f32 {
        debug_assert_eq!(PANEL_POT_CONFIRMING_STEPS, 2);
        let index = parameter as usize;
        let accepted_code = analog_pot_code(self.accepted.get(parameter));
        let target_code = analog_pot_code(target.get(parameter));
        let direction = target_code.cmp(&accepted_code) as i8;
        if direction == 0 {
            self.pending_direction[index] = 0;
        } else if self.pending_direction[index] == direction {
            let copied = self
                .accepted
                .set(parameter as u32, f64::from(target.get(parameter)));
            debug_assert!(copied);
            self.pending_direction[index] = 0;
        } else {
            self.pending_direction[index] = direction;
        }
        self.accepted.get(parameter)
    }
}

impl Default for ControlScheduler {
    fn default() -> Self {
        Self {
            applied: Settings::default(),
            panel: PanelPotScanner::new(Settings::default()),
            service_index: 0,
            samples_until_service: 1,
            cycle_samples: CONTROL_SERVICE_STEP_COUNT as u32,
            changed_cycle: false,
        }
    }
}

impl ControlScheduler {
    pub fn prepare(&mut self, target: Settings, sample_rate: f32) {
        self.applied = target;
        self.panel.synchronize(target);
        self.begin_cycle(target, sample_rate);
    }

    pub fn notify_change(&mut self, sample_rate: f32) {
        if self.changed_cycle {
            return;
        }
        let old_cycle = self.cycle_samples.max(1);
        let new_cycle = cycle_samples(sample_rate, CONTROL_LOOP_CHANGED_MICROSECONDS);
        self.samples_until_service = self
            .samples_until_service
            .saturating_mul(new_cycle)
            .div_ceil(old_cycle)
            .max(1);
        self.cycle_samples = new_cycle;
        self.changed_cycle = true;
    }

    /// Program/state recall replaces the stored table directly; it does not
    /// pretend that twenty-four physical knobs moved across the panel.
    pub fn notify_recall(&mut self, target: Settings, sample_rate: f32) {
        self.panel.synchronize(target);
        self.notify_change(sample_rate);
    }

    pub fn next(&mut self, target: Settings, sample_rate: f32) -> ControlTick {
        if self.samples_until_service > 1 {
            self.samples_until_service -= 1;
            return ControlTick {
                settings: self.with_direct_controls(target),
                cv_strobe: None,
            };
        }

        let cv_strobe = if self.service_index < ANALOG_POT_COUNT {
            let pot = AnalogPot::try_from(self.service_index as u8).expect("valid scan position");
            self.scan_panel_parameter(target, pot.parameter());
            if let Some(scale_parameter) = pot.scale_parameter() {
                self.scan_panel_parameter(target, scale_parameter);
            }
            None
        } else {
            if self.service_index == ANALOG_POT_COUNT {
                self.copy_switch_latches(target);
            }
            let strobe_slot = self.service_index - ANALOG_POT_COUNT;
            CONTROL_VOLTAGE_STROBE_ORDER[strobe_slot].map(|destination| destination as usize)
        };
        self.service_index += 1;

        if self.service_index == CONTROL_SERVICE_STEP_COUNT {
            self.begin_cycle(target, sample_rate);
        } else {
            self.samples_until_service = service_spacing(self.cycle_samples, self.service_index);
        }
        ControlTick {
            settings: self.with_direct_controls(target),
            cv_strobe,
        }
    }

    pub fn current(&self, target: Settings) -> Settings {
        self.with_direct_controls(target)
    }

    fn begin_cycle(&mut self, target: Settings, sample_rate: f32) {
        self.service_index = 0;
        self.changed_cycle = target != self.applied;
        let microseconds = if self.changed_cycle {
            CONTROL_LOOP_CHANGED_MICROSECONDS
        } else {
            CONTROL_LOOP_IDLE_MICROSECONDS
        };
        self.cycle_samples = cycle_samples(sample_rate, microseconds);
        self.samples_until_service = service_spacing(self.cycle_samples, 0);
    }

    fn copy_switch_latches(&mut self, target: Settings) {
        for index in 0..PARAMETER_COUNT as u32 {
            let parameter = Parameter::try_from(index).expect("contiguous parameter contract");
            if !is_scanned_pot(parameter) && !is_direct_control(parameter) {
                copy_parameter(&mut self.applied, target, parameter);
            }
        }
    }

    fn scan_panel_parameter(&mut self, target: Settings, parameter: Parameter) {
        let accepted = self.panel.scan(target, parameter);
        let copied = self.applied.set(parameter as u32, f64::from(accepted));
        debug_assert!(copied);
    }

    fn with_direct_controls(&self, target: Settings) -> Settings {
        let mut result = self.applied;
        for parameter in [
            Parameter::MasterVolume,
            Parameter::MasterTune,
            Parameter::VintageSpread,
        ] {
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
        AnalogPot::try_from(index).is_ok_and(|pot| {
            pot.parameter() == parameter || pot.scale_parameter() == Some(parameter)
        })
    })
}

fn is_direct_control(parameter: Parameter) -> bool {
    matches!(
        parameter,
        Parameter::MasterVolume | Parameter::MasterTune | Parameter::VintageSpread
    )
}

fn cycle_samples(sample_rate: f32, microseconds: u32) -> u32 {
    (libm::roundf(sample_rate.max(1.0) * microseconds as f32 / 1_000_000.0) as u32)
        .max(CONTROL_SERVICE_STEP_COUNT as u32)
}

fn service_spacing(cycle_samples: u32, service_index: usize) -> u32 {
    let count = CONTROL_SERVICE_STEP_COUNT as u32;
    let base = cycle_samples / count;
    let remainder = cycle_samples % count;
    base + u32::from((service_index as u32) < remainder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rf_5_contract::hardware::CONTROL_VOLTAGE_DESTINATION_COUNT;

    #[test]
    fn unchanged_cycle_is_exactly_six_milliseconds() {
        let settings = Settings::default();
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(settings, 48_000.0);
        assert_eq!(scheduler.cycle_samples, 288);
        for _ in 0..288 {
            assert_eq!(scheduler.next(settings, 48_000.0).settings, settings);
        }
        assert_eq!(scheduler.service_index, 0);
    }

    #[test]
    fn changed_cycle_is_extended_and_settles_all_controls() {
        let initial = Settings::default();
        let mut target = initial;
        assert!(target.set(Parameter::FilterAttack as u32, 0.8));
        assert!(target.set(Parameter::OscillatorBDetune as u32, 0.1));
        assert!(target.set(Parameter::OscillatorSync as u32, 1.0));
        assert!(target.set(Parameter::ScaleE as u32, 0.25));
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(initial, 48_000.0);
        scheduler.notify_change(48_000.0);
        assert_eq!(scheduler.cycle_samples, 528);
        for _ in 0..528 {
            let _ = scheduler.next(target, 48_000.0);
        }
        assert_eq!(
            scheduler.current(target).get(Parameter::FilterAttack),
            initial.get(Parameter::FilterAttack)
        );
        for _ in 0..528 {
            let _ = scheduler.next(target, 48_000.0);
        }
        assert_eq!(scheduler.next(target, 48_000.0).settings, target);
    }

    #[test]
    fn panel_pot_requires_two_consecutive_scans_in_one_direction() {
        let initial = Settings::default();
        let mut higher = initial;
        let start = analog_pot_code(initial.get(Parameter::FilterAttack));
        assert!(higher.set(Parameter::FilterAttack as u32, f64::from(start + 1) / 127.0));
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(initial, 48_000.0);
        scheduler.notify_change(48_000.0);

        for _ in 0..528 {
            let _ = scheduler.next(higher, 48_000.0);
        }
        assert_eq!(
            scheduler.current(higher).get(Parameter::FilterAttack),
            initial.get(Parameter::FilterAttack)
        );
        for _ in 0..528 {
            let _ = scheduler.next(higher, 48_000.0);
        }
        assert_eq!(
            scheduler.current(higher).get(Parameter::FilterAttack),
            higher.get(Parameter::FilterAttack)
        );
    }

    #[test]
    fn reversing_a_panel_move_restarts_direction_confirmation() {
        let initial = Settings::default();
        let start = analog_pot_code(initial.get(Parameter::FilterAttack));
        let mut higher = initial;
        let mut lower = initial;
        assert!(higher.set(Parameter::FilterAttack as u32, f64::from(start + 1) / 127.0));
        assert!(lower.set(Parameter::FilterAttack as u32, 0.0));
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(initial, 48_000.0);
        scheduler.notify_change(48_000.0);
        for target in [higher, lower] {
            for _ in 0..528 {
                let _ = scheduler.next(target, 48_000.0);
            }
        }
        assert_eq!(
            scheduler.current(lower).get(Parameter::FilterAttack),
            initial.get(Parameter::FilterAttack)
        );
        for _ in 0..528 {
            let _ = scheduler.next(lower, 48_000.0);
        }
        assert_eq!(
            scheduler.current(lower).get(Parameter::FilterAttack),
            lower.get(Parameter::FilterAttack)
        );
    }

    #[test]
    fn program_recall_bypasses_physical_pot_confirmation() {
        let initial = Settings::default();
        let mut recalled = initial;
        assert!(recalled.set(Parameter::FilterAttack as u32, 0.8));
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(initial, 48_000.0);
        scheduler.notify_recall(recalled, 48_000.0);
        for _ in 0..528 {
            let _ = scheduler.next(recalled, 48_000.0);
        }
        assert_eq!(
            scheduler.current(recalled).get(Parameter::FilterAttack),
            recalled.get(Parameter::FilterAttack)
        );
    }

    #[test]
    fn non_programmable_knobs_are_direct_analog_controls() {
        let initial = Settings::default();
        let mut target = initial;
        assert!(target.set(Parameter::MasterVolume as u32, 0.1));
        assert!(target.set(Parameter::MasterTune as u32, 0.9));
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(initial, 48_000.0);
        let applied = scheduler.next(target, 48_000.0).settings;
        assert_eq!(applied.get(Parameter::MasterVolume), 0.1);
        assert_eq!(applied.get(Parameter::MasterTune), 0.9);
    }

    #[test]
    fn one_cpu_cycle_contains_all_pot_reads_and_cv_writes() {
        let settings = Settings::default();
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(settings, 48_000.0);
        let mut strobed = [false; CONTROL_VOLTAGE_DESTINATION_COUNT];
        for _ in 0..288 {
            if let Some(destination) = scheduler.next(settings, 48_000.0).cv_strobe {
                assert!(!strobed[destination]);
                strobed[destination] = true;
            }
        }
        assert!(strobed.into_iter().all(|value| value));
    }

    #[test]
    fn cv_writes_follow_the_v81_physical_strobe_sequence() {
        let settings = Settings::default();
        let mut scheduler = ControlScheduler::default();
        scheduler.prepare(settings, 48_000.0);
        let mut observed = [None; CONTROL_VOLTAGE_STROBE_SLOT_COUNT];
        let mut strobe_slot = 0;
        for _ in 0..288 {
            let service_due = scheduler.samples_until_service == 1;
            let before = scheduler.service_index;
            let tick = scheduler.next(settings, 48_000.0);
            if service_due && before >= ANALOG_POT_COUNT {
                observed[strobe_slot] = tick.cv_strobe;
                strobe_slot += 1;
            }
        }
        assert_eq!(strobe_slot, CONTROL_VOLTAGE_STROBE_SLOT_COUNT);
        assert_eq!(
            observed,
            CONTROL_VOLTAGE_STROBE_ORDER.map(|destination| destination.map(|value| value as usize))
        );
    }
}
