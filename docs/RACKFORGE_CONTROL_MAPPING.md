# RackForge control mapping

RF-5 publishes RackForge Control Profile v1 roles as automatic controller
defaults. These declarations are hints owned by the host: a user's explicit
MIDI Link always takes priority, and every public RF-5 control remains manually
linkable through its `data-rackforge-parameter-index` attribute.

## Published roles

| RackForge v1 role | RF-5 parameter | Rationale |
| --- | --- | --- |
| `synth.oscillator.pulse_width` | Oscillator A Pulse Width | Oscillator A is the primary oscillator when v1 offers only one unqualified pulse-width role. |
| `synth.oscillator.noise.level` | Noise | Direct oscillator-mixer noise level. |
| `synth.filter.cutoff` | Cutoff | Direct low-pass cutoff control. |
| `synth.filter.resonance` | Resonance | Direct resonance-current control. |
| `synth.filter.envelope.amount` | Filter Envelope Amount | Direct filter-envelope depth. |
| `synth.filter.key_tracking` | Keyboard Tracking | The original two-state keyboard tracking control; continuous controller input is intentionally thresholded by RackForge's boolean mapping. |
| `synth.envelope.amp.attack` | Amplifier Attack | Direct amplifier CEM3310 timing control. |
| `synth.envelope.amp.decay` | Amplifier Decay | Direct amplifier CEM3310 timing control. |
| `synth.envelope.amp.sustain` | Amplifier Sustain | Direct amplifier CEM3310 level control. |
| `synth.envelope.amp.release` | Amplifier Release | Direct amplifier CEM3310 timing control. |
| `synth.lfo.rate` | LFO Frequency | Direct common-LFO rate control. |

On the bundled Arturia KeyLab Essential mk3 this gives useful defaults to
encoder 1 (Oscillator A Pulse Width), encoder 3 (Noise), encoder 4 (Filter
Envelope Amount), encoder 6 (Keyboard Tracking), and faders 1-7 (amplifier
ADSR, cutoff, resonance and LFO rate). Fader 9 remains the RackForge-owned
global master level and does not duplicate RF-5's volume.

## Deliberately unbound v1 roles

| RackForge v1 role | Why RF-5 does not publish it |
| --- | --- |
| `synth.oscillator.sub.level` | RF-5 has no sub oscillator. |
| `synth.filter.lfo.amount` | Wheel Mod Filter is a destination switch; wheel position supplies the amount. Mapping a continuous amount role to that switch would be misleading. |
| `synth.lfo.depth` | RF-5 has no stored LFO-depth parameter. The performance wheel supplies live depth. |
| `synth.lfo.delay` | The modeled instrument has no LFO-delay control. |
| `synth.amplifier.level` | RackForge owns the automatic master-volume mapping; RF-5's physical Master Volume remains available through the UI and explicit MIDI Link only. |
| `plugin.output.level` | Deliberately omitted so a controller's master control cannot compete with RackForge's global master level. |
| `mixer.channel.level` | v1 has one unqualified mixer role, while RF-5 has distinct oscillator A and B levels. Choosing one would make the other inaccessible by convention. |
| `mixer.channel.pan` | RF-5 is mono before RackForge's stereo host boundary and has no channel pan. |
| `performance.modulation` | Modulation is incoming performance MIDI, not persistent plugin state. |
| `performance.expression` | RF-5 does not currently model an expression-pedal circuit or public expression parameter. |
| `performance.sustain` | Sustain is handled as performance MIDI CC64, not as the programmable Release Enable switch. |
| `rackforge.master.level` / `rackforge.master.pan` | Both are reserved exclusively for the RackForge host mix and must never target RF-5 parameters. |

Control Profile v1 does not define qualified oscillator A/B roles, waveform or
sync roles, filter-envelope ADSR roles, Poly Mod roles, Unison, Glide, Master
Tune, Scale Mode or automatic Tune. Those RF-5 controls remain fully available
to MIDI Learn and explicit links. They should gain automatic defaults only
through future official, unambiguous RackForge roles rather than RF-5-specific
semantic names that existing controllers do not publish.
