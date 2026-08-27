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
sections. The wide layout follows the Rev 3 control hierarchy rather than a
generic parameter grid:

1. Modulation: Poly Mod, common LFO and Wheel Mod;
2. Oscillators: oscillator A, oscillator B and the three-input mixer;
3. Filter + Envelopes: cutoff, resonance, envelope amount, keyboard switching
   and the filter ADSR share one outlined FILTER block; the amplifier ADSR
   remains a second block;
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

## Physical material system

The panel is generated from reusable CSS geometry and does not use a scanned
front-panel image. Its black painted-metal surface reuses the broad
Gaussian-like anisotropic highlight profile proven by RF-106, with a fixed
104-degree light direction and a very fine vertical grain. Control movement
never rotates this lighting: only the white indicator rotates, so the
highlight remains tied to the virtual light source.

The standard potentiometer combines a fluted black skirt, a recessed top face,
fixed anisotropic reflections, an SVG graduation ring and a high-contrast white
indicator. Master Tune and Volume use the same component with a brushed
aluminium top-face variant. Toggle controls combine a separate red LED, a dark
switch well and a nearly flat rectangular cap. Wood rails and end cheeks are
procedural gradients, not bundled textures.

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

The engine receives pitch bend and modulation wheel through MIDI. Automatic
tune is already an explicit momentary public operation on the Voice page.
On-screen pitch and modulation wheel positions remain non-patch performance
state rather than stored parameters.

Those controls must not be disguised as stored patch parameters. The current
surface can evolve independently because all existing synthesis controls bind
by stable parameter index.

## Independence

The surface uses original RF-5 lettering, generated geometry and CSS materials.
Reference photographs inform proportions and ergonomic grouping only. It does
not ship manufacturer names, product marks, scanned panel artwork, copied wood
textures or third-party interface assets.
