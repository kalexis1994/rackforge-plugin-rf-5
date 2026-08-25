# RF-5 control-surface architecture

## Boundary

The RF-5 graphical panel is a Rust WebAssembly client of RackForge's
`rackforge.plugin.web@1` bridge. `play.html` only loads the generated module;
rendering, state, program selection and interaction logic remain in Rust.

The first surface deliberately reuses RF-106's proven host boundary rather
than its instrument-specific layout:

- strict parent-window origin and protocol validation;
- request IDs, response matching and bounded request timeouts;
- context-driven program catalogs and explicit `plugin.select_sound` calls;
- complete parameter snapshots followed by targeted `parameter_changed`
  updates;
- `data-rackforge-parameter-index` anchors for host-owned context menus and
  MIDI Link;
- primary-pointer capture and relative vertical knob movement on mouse and
  touch;
- suppression of secondary-button hardware activation.

## Panel map

The source-backed front-panel inventory is grouped into five responsive
sections:

1. Modulation: Poly Mod, common LFO and Wheel Mod;
2. Oscillators: oscillator A, oscillator B and the three-input mixer;
3. Filter + Envelopes: filter, filter envelope and amplifier envelope;
4. Voice: Glide, Unison, Release, master volume and the RF-5 voice-population
   control;
5. Scale Mode: the twelve patch-independent V8.1 chromatic offsets.

The map is a static Rust data structure and its test compares the set against
the packaged parameter schema. Every public parameter must occur exactly once;
adding or removing a DSP parameter without placing it on the panel therefore
fails the normal test suite.

Scale Mode and Voice Spread remain visibly separated from the normal synthesis
path. Scale Mode is an original hidden operating mode that reuses panel pots,
while Voice Spread is an RF-5 machine-character control rather than a claim
about a dedicated historical knob.

## Responsive behavior

The UI changes layout instead of scaling the complete panel bitmap:

- wide surfaces show three physical groups on one row;
- laptop widths preserve each connected group and stack the groups;
- phones use two or three touchable controls per row;
- Scale Mode reduces from twelve to six, four and then three columns;
- the full program catalog remains below the active hardware section at every
  width.

Knobs expose a native range input for keyboard accessibility but pointer
movement belongs to the surrounding hardware surface. This avoids Chromium's
axis and initial-jump problems while retaining focus and arrow-key operation.

## Remaining physical controls

The engine already receives pitch bend and modulation wheel through MIDI, and
owns an explicit oscillator-tune operation. They are not yet public parameters
or web-bridge commands. A complete front panel still needs an append-only,
non-patch control boundary for:

- the momentary automatic-tune trigger;
- visible/on-screen pitch and modulation wheel positions.

Those controls must not be disguised as stored patch parameters. The current
surface can evolve independently because all existing synthesis controls bind
by stable parameter index.

## Independence

The surface uses original RF-5 lettering, generated geometry and CSS materials.
Reference photographs inform proportions and ergonomic grouping only. It does
not ship manufacturer names, product marks, scanned panel artwork, copied wood
textures or third-party interface assets.
