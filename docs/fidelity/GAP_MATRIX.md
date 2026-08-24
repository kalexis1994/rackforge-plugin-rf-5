# Fidelity gap matrix

| Block | Active implementation | Evidence state | Release status |
| --- | --- | --- | --- |
| Voice allocation | Deterministic five-voice round robin | Architecture inventory | Baseline |
| VCO A/B | Band-unlimited dual saw baseline | Rev 3 CEM3340 topology and CV scale accepted | Replace |
| Hard sync | Not implemented | Rev 3 routing accepted; transition behaviour unmeasured | Open |
| Mixer/noise | Fixed oscillator mix, no noise | CA3280 topology accepted; gain and noise spectra unmeasured | Open |
| Poly Mod | Not implemented | Rev 3 sources, destinations and summing paths accepted | Open |
| Filter | One-pole feedback baseline | Rev 3 CEM3320 four-pole topology accepted; nonlinear response unmeasured | Replace |
| Filter envelope | Not implemented | CEM3310 topology and CV polarity accepted; time mapping unmeasured | Open |
| Amplifier envelope | Linear ADSR baseline | CEM3310 topology and CV polarity accepted; time mapping unmeasured | Replace |
| VCA/output | Fixed scaling | CA3280 topology accepted; gain staging unmeasured | Replace |
| Unison/glide | Not implemented | Rev 3 routing and OTA slew topology accepted | Open |
| Control scanning/CV | Host values are applied directly | 24-pot order, 7/14-bit resolution, loop timing and 38 S/H destinations accepted | Contract frozen; behaviour open |
| Performance controls | Note on/off and all notes off | MIDI contract | Partial |
| Factory programs | Four original baseline programs | Internal | Expand |
