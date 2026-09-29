#[cfg(any(target_arch = "wasm32", test))]
mod light;
#[cfg(any(target_arch = "wasm32", test))]
mod panel;
#[cfg(any(target_arch = "wasm32", test))]
#[cfg(test)]
mod texture;

#[cfg(any(target_arch = "wasm32", test))]
use serde::Deserialize;

#[cfg(any(target_arch = "wasm32", test))]
const PROTOCOL: &str = "rackforge.plugin.web@1";

#[cfg(any(target_arch = "wasm32", test))]
fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(any(target_arch = "wasm32", test))]
/// A legend's rule, the section outline's straight, thinner kin: a line at
/// height `y` from `start` to `end`, left open from `label_start` to
/// `label_end` for the legend's name ("---- DESTINATION ----").
fn legend_rule_path(
    start: f64,
    end: f64,
    label_start: f64,
    label_end: f64,
    y: f64,
) -> Option<String> {
    if ![start, end, label_start, label_end, y]
        .iter()
        .all(|value| value.is_finite())
        || label_start <= start
        || end <= label_end
    {
        return None;
    }
    Some(format!(
        "M {start:.2} {y:.2} H {label_start:.2} M {label_end:.2} {y:.2} H {end:.2}"
    ))
}

#[cfg(any(target_arch = "wasm32", test))]
fn section_outline_path(
    width: f64,
    height: f64,
    label_start: f64,
    label_end: f64,
    requested_radius: f64,
) -> Option<String> {
    if !width.is_finite()
        || !height.is_finite()
        || !label_start.is_finite()
        || !label_end.is_finite()
        || !requested_radius.is_finite()
        || width <= 4.0
        || height <= 4.0
        || label_end <= label_start
    {
        return None;
    }

    let inset = 1.0;
    let left = inset;
    let top = inset;
    let right = width - inset;
    let bottom = height - inset;
    let radius = requested_radius
        .max(2.0)
        .min((width - inset * 2.0) * 0.25)
        .min((height - inset * 2.0) * 0.5);
    let gap_start = label_start.clamp(left + radius + 2.0, right - radius - 4.0);
    let gap_end = label_end.clamp(gap_start + 4.0, right - radius - 2.0);

    Some(format!(
        "M {gap_start:.2} {top:.2} H {:.2} A {radius:.2} {radius:.2} 0 0 0 {left:.2} {:.2} V {:.2} A {radius:.2} {radius:.2} 0 0 0 {:.2} {bottom:.2} H {:.2} A {radius:.2} {radius:.2} 0 0 0 {right:.2} {:.2} V {:.2} A {radius:.2} {radius:.2} 0 0 0 {:.2} {top:.2} H {gap_end:.2}",
        left + radius,
        top + radius,
        bottom - radius,
        left + radius,
        right - radius,
        bottom - radius,
        top + radius,
        right - radius,
    ))
}

/// The LED on a switch's deck: the black bezel and the lens that nearly
/// fills it.
#[cfg(any(target_arch = "wasm32", test))]
const LED_RIM_RADIUS: f64 = 6.0;
#[cfg(any(target_arch = "wasm32", test))]
const LED_LENS_RADIUS: f64 = 5.1;

/// The panel round a switch that its dust image covers, in switch units: a
/// little past the cut-out on every side, far enough for the dust to fade.
#[cfg(any(target_arch = "wasm32", test))]
const WELL_DUST_ORIGIN: (f64, f64) = (-14.0, -20.0);
#[cfg(any(target_arch = "wasm32", test))]
const WELL_DUST_SPAN: (f64, f64) = (80.0, 98.0);

/// The generated finishes (the panel's relief and the dust) are made offline
/// by `texture.rs` and shipped as images, so the page only loads them; its
/// tests check the images still match. Each dust comes in a few variants so
/// neighbouring controls are not soiled alike.
#[cfg(any(target_arch = "wasm32", test))]
const DUST_VARIANTS: usize = 3;
#[cfg(any(target_arch = "wasm32", test))]
const PANEL_TEXTURE_PATH: &str = "assets/finishes/panel.png";

#[cfg(any(target_arch = "wasm32", test))]
fn cap_dust_path(leds: usize, variant: usize) -> String {
    format!("assets/finishes/cap-{leds}-{variant}.png")
}

#[cfg(any(target_arch = "wasm32", test))]
fn well_dust_path(variant: usize) -> String {
    format!("assets/finishes/well-{variant}.png")
}

#[cfg(any(target_arch = "wasm32", test))]
fn knob_dust_path(variant: usize) -> String {
    format!("assets/finishes/knob-{variant}.png")
}

/// Every finish image the page is drawn with.
#[cfg(any(target_arch = "wasm32", test))]
fn finish_paths() -> Vec<String> {
    let mut paths = vec![PANEL_TEXTURE_PATH.to_owned()];
    for variant in 0..DUST_VARIANTS {
        paths.extend([
            cap_dust_path(1, variant),
            cap_dust_path(2, variant),
            well_dust_path(variant),
            knob_dust_path(variant),
        ]);
    }
    paths
}

#[cfg(any(target_arch = "wasm32", test))]
/// The dust on a switch's cap, laid over the whole cap so it moves with it
/// when pressed.
fn switch_dust_layer(index: u32, two_leds: bool) -> String {
    let path = cap_dust_path(1 + usize::from(two_leds), index as usize % DUST_VARIANTS);
    format!(
        "<image class=\"switch-dirt\" href=\"{path}\" x=\"0\" y=\"-6\" width=\"52\" height=\"70\" preserveAspectRatio=\"none\"></image>"
    )
}

#[cfg(any(target_arch = "wasm32", test))]
fn panel_dust_image(path: &str, (x, y): (f64, f64), (width, height): (f64, f64)) -> String {
    format!(
        "<image class=\"panel-dust\" href=\"{path}\" x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\" preserveAspectRatio=\"none\"></image>"
    )
}

#[cfg(any(target_arch = "wasm32", test))]
/// Dust from a switch's cut-out a little way out onto the panel, so it
/// stays put when the cap is pressed.
fn well_dust_layer(index: u32) -> String {
    panel_dust_image(
        &well_dust_path(index as usize % DUST_VARIANTS),
        WELL_DUST_ORIGIN,
        WELL_DUST_SPAN,
    )
}

#[cfg(any(target_arch = "wasm32", test))]
/// Dust round the foot of a knob, over its scale.
fn knob_dust_layer(index: u32) -> String {
    panel_dust_image(
        &knob_dust_path(index as usize % DUST_VARIANTS),
        (0.0, 0.0),
        (100.0, 100.0),
    )
}

#[cfg(any(target_arch = "wasm32", test))]
/// Where a lit LED's own light lands on the switch's top bevel, worked out
/// once when the switch is drawn: a gradient whose opacity CSS turns on with
/// the LED, so nothing is shaded while playing.
///
/// The LED is a point source `LED_HEIGHT` above the flat deck at (`led_x`,
/// `LED_Y`). The top bevel is the plane through the deck edge at y = 20 that
/// leans `BEVEL_LEAN_DEGREES` towards the viewer. With D the LED's distance to
/// that plane and rho the distance within it from the foot of the
/// perpendicular, irradiance is cos(theta) / d^2 = D / d^3, i.e.
/// E / E_max = (1 + rho^2 / D^2)^(-3/2). Seen from the front the plane is
/// foreshortened by cos(lean) in y, so the profile is an ellipse centred on
/// the foot's projection. The face and side bevels have the LED behind their
/// planes and receive nothing.
fn led_spill_gradient(id: &str, led_x: f64) -> String {
    const LED_Y: f64 = 10.0;
    const LED_HEIGHT: f64 = 1.5;
    const BEVEL_EDGE_Y: f64 = 20.0;
    const BEVEL_LEAN_DEGREES: f64 = 50.0;
    const PEAK_OPACITY: f64 = 0.6;
    let lean = BEVEL_LEAN_DEGREES.to_radians();
    let (normal_y, normal_z) = (-lean.sin(), lean.cos());
    let distance = normal_y * (LED_Y - BEVEL_EDGE_Y) + normal_z * LED_HEIGHT;
    let foot_y = LED_Y - distance * normal_y;
    let radius = 3.0 * distance;
    let stops: String = (0..=12)
        .map(|step| {
            let t = f64::from(step) / 12.0;
            let rho = t * radius;
            let irradiance = (1.0 + (rho / distance).powi(2)).powf(-1.5);
            format!(
                "<stop offset=\"{t:.4}\" stop-color=\"#ff3e39\" stop-opacity=\"{:.4}\"></stop>",
                PEAK_OPACITY * irradiance
            )
        })
        .collect();
    format!(
        "<radialGradient id=\"{id}\" gradientUnits=\"userSpaceOnUse\" cx=\"0\" cy=\"0\" r=\"1\" gradientTransform=\"translate({led_x:.2} {foot_y:.2}) scale({radius:.2} {:.2})\">{stops}</radialGradient>",
        radius * lean.cos()
    )
}

#[cfg(any(target_arch = "wasm32", test))]
fn prophet_switch_svg(
    index: u32,
    primary_active: bool,
    secondary_active: Option<bool>,
    light_button: bool,
) -> String {
    switch_svg(index, primary_active, secondary_active, light_button, None)
}

#[cfg(any(target_arch = "wasm32", test))]
/// What the Rev 3's own two-digit display shows for a program, for a
/// program display too narrow for its name: bank and program ("11"), a
/// decimal point after the bank for File 2 ("1.1") and after the program
/// for File 3 ("11."), as the Rev 3 marks its three files. The RF-5's own
/// programs show "U" and their place in the USER bank.
fn program_digits(name: &str, bank: Option<&str>, user_place: Option<usize>) -> String {
    if let Some(place) = user_place {
        return format!("U{place}");
    }
    let place = name.as_bytes();
    if place.len() < 3
        || !place[0].is_ascii_digit()
        || place[1] != b'-'
        || !place[2].is_ascii_digit()
    {
        return "--".to_owned();
    }
    let (bank_digit, program_digit) = (char::from(place[0]), char::from(place[2]));
    match bank.unwrap_or_default() {
        bank if bank.ends_with("file2") => format!("{bank_digit}.{program_digit}"),
        bank if bank.ends_with("file3") => format!("{bank_digit}{program_digit}."),
        _ => format!("{bank_digit}{program_digit}"),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
/// Which way a program step goes.
#[derive(Clone, Copy)]
enum Step {
    Previous,
    Next,
}

#[cfg(any(target_arch = "wasm32", test))]
/// A program step key: the panel's light switch with no LED, its name
/// printed on the panel under it in two lines.
fn step_key_svg(index: u32, step: Step) -> String {
    let name = match step {
        Step::Previous => "BACK<br>PROGRAM",
        Step::Next => "NEXT<br>PROGRAM",
    };
    format!(
        "{}<span class=\"control-label\" aria-hidden=\"true\">{name}</span>",
        switch_svg(index, false, None, true, Some(""))
    )
}

#[cfg(any(target_arch = "wasm32", test))]
/// A switch drawing; `deck_mark`, when given, is printed on the LED deck in
/// place of the LED.
fn switch_svg(
    index: u32,
    primary_active: bool,
    secondary_active: Option<bool>,
    light_button: bool,
    deck_mark: Option<&str>,
) -> String {
    let primary_x = if secondary_active.is_some() { 19 } else { 26 };
    let (light_x, light_y) = light::toward_light();
    let (glint_x, glint_y) = (light_x * 2.5, light_y * 2.5);
    let (led_rim, led_lens) = (LED_RIM_RADIUS, LED_LENS_RADIUS);
    let well_dust = well_dust_layer(index);
    // The block's face shades corner to corner; the rocker's face is turned a
    // few degrees further, as its bevel was drawn.
    let block_vector = light::gradient_attributes(0.0, core::f64::consts::FRAC_1_SQRT_2);
    let rocker_vector = light::gradient_attributes(5.6, 0.6466);
    // The switch sits in a narrow cut-out. The cap stands taller than the
    // gutter is wide, so it shades the gutter on its far side from the light;
    // only the near side keeps a trace of light. The panel's cut edge catches
    // light only where it faces the light.
    let well_vector = light::gradient_attributes(0.0, core::f64::consts::FRAC_1_SQRT_2);
    let (left, top, right, bottom) = (1.2, -5.8, 50.8, 61.8);
    let rim_edge = |outward: (f64, f64), path: &str| {
        let facing = outward.0 * light_x + outward.1 * light_y;
        if facing <= 0.0 {
            String::new()
        } else {
            format!(
                "<path class=\"switch-well-rim\" d=\"{path}\" stroke-opacity=\"{:.3}\"></path>",
                0.1 * facing
            )
        }
    };
    let well_rim = [
        rim_edge((0.0, -1.0), &format!("M{left} {top}H{right}")),
        rim_edge((-1.0, 0.0), &format!("M{left} {top}V{bottom}")),
        rim_edge((1.0, 0.0), &format!("M{right} {top}V{bottom}")),
        rim_edge((0.0, 1.0), &format!("M{left} {bottom}H{right}")),
    ]
    .concat();
    // Specular sheen, facet by facet. The cap's fine grain runs across the
    // switch, so it is rougher along x than down y; the bevels lean 50 degrees
    // off the face. Each facet's normal meets the panel light in Ward's term.
    // Every facet is flat, so under a distant light each is lit evenly: the
    // flat face and the LED deck included, which a lobe would make look domed.
    let (alpha_across, alpha_down) = (0.75, 0.55);
    let across = [1.0, 0.0, 0.0];
    let (lean_sin, lean_cos) = (50.0_f64.to_radians().sin(), 50.0_f64.to_radians().cos());
    let strength = if light_button { 0.22 } else { 0.16 };
    let sheen = |normal: [f64; 3]| {
        strength * light::ward_specular(normal, across, alpha_across, alpha_down)
    };
    let top_sheen = sheen([0.0, -lean_sin, lean_cos]);
    let left_sheen = sheen([-lean_sin, 0.0, lean_cos]);
    let right_sheen = sheen([lean_sin, 0.0, lean_cos]);
    let foot_sheen = sheen([0.0, lean_sin, lean_cos]);
    let flat_sheen = sheen([0.0, 0.0, 1.0]);
    let dust = switch_dust_layer(index, secondary_active.is_some());
    let spill_a = led_spill_gradient(&format!("led-spill-a-{index}"), f64::from(primary_x));
    let (spill_b, spill_b_path) = if secondary_active.is_some() {
        (
            led_spill_gradient(&format!("led-spill-b-{index}"), 33.0),
            format!(
                "<path class=\"led-spill led-spill-b\" d=\"M3 20H49L45 24H7Z\" fill=\"url(#led-spill-b-{index})\"></path>"
            ),
        )
    } else {
        (String::new(), String::new())
    };
    let secondary = secondary_active.map_or_else(String::new, |active| {
        format!(
            "<g class=\"led{}\" data-led=\"b\" transform=\"translate(33 10)\"><circle class=\"led-rim\" r=\"{led_rim}\"></circle><circle class=\"led-lens\" r=\"{led_lens}\"></circle><circle class=\"led-glint\" cx=\"{glint_x:.2}\" cy=\"{glint_y:.2}\" r=\"0.9\"></circle></g>",
            if active { " on" } else { "" }
        )
    });
    let primary = deck_mark.map_or_else(
        || {
            format!(
                "<g class=\"led{}\" data-led=\"a\" transform=\"translate({primary_x} 10)\"><circle class=\"led-rim\" r=\"{led_rim}\"></circle><circle class=\"led-lens\" r=\"{led_lens}\"></circle><circle class=\"led-glint\" cx=\"{glint_x:.2}\" cy=\"{glint_y:.2}\" r=\"0.9\"></circle></g>",
                if primary_active { " on" } else { "" }
            )
        },
        str::to_owned,
    );
    let block_stops = if light_button {
        "<stop offset=\"0\" stop-color=\"#8a9893\"></stop><stop offset=\"0.48\" stop-color=\"#77837f\"></stop><stop offset=\"1\" stop-color=\"#5c6562\"></stop>"
    } else {
        "<stop offset=\"0\" stop-color=\"#1d1e21\"></stop><stop offset=\"0.42\" stop-color=\"#0f1012\"></stop><stop offset=\"1\" stop-color=\"#050507\"></stop>"
    };
    let rocker_stops = if light_button {
        "<stop offset=\"0\" stop-color=\"#889591\"></stop><stop offset=\"0.42\" stop-color=\"#77837f\"></stop><stop offset=\"1\" stop-color=\"#59625f\"></stop>"
    } else {
        "<stop offset=\"0\" stop-color=\"#1e1f22\"></stop><stop offset=\"0.42\" stop-color=\"#121316\"></stop><stop offset=\"1\" stop-color=\"#07080a\"></stop>"
    };
    format!(
        "<svg class=\"prophet-switch\" viewBox=\"0 -6 52 70\" aria-hidden=\"true\"><defs><linearGradient id=\"switch-well-{index}\" {well_vector}><stop offset=\"0\" stop-color=\"#0c0d10\"></stop><stop offset=\"0.45\" stop-color=\"#040405\"></stop><stop offset=\"1\" stop-color=\"#000000\"></stop></linearGradient><linearGradient id=\"switch-block-{index}\" {block_vector}>{block_stops}</linearGradient><linearGradient id=\"switch-rocker-{index}\" {rocker_vector}>{rocker_stops}</linearGradient>{spill_a}{spill_b}</defs><rect class=\"switch-well\" x=\"1.2\" y=\"-5.8\" width=\"49.6\" height=\"67.6\" rx=\"1.2\" fill=\"url(#switch-well-{index})\"></rect>{well_rim}{well_dust}<g class=\"switch-block\"><rect class=\"switch-base\" x=\"3\" y=\"-4\" width=\"46\" height=\"64\" rx=\"1\" fill=\"url(#switch-block-{index})\"></rect><path class=\"switch-deck\" d=\"M3 -4H49V20H3Z\"></path><path class=\"switch-sheen\" d=\"M3 -4H49V20H3Z\" fill-opacity=\"{flat_sheen:.3}\"></path><path class=\"switch-deck-seam\" d=\"M3 20H49\"></path>{primary}{secondary}<g class=\"switch-rocker\"><path class=\"switch-rocker-left\" d=\"M3 20L7 24V58L3 60Z\"></path><path class=\"switch-rocker-right\" d=\"M49 20L45 24V58L49 60Z\"></path><path class=\"switch-rocker-face\" d=\"M7 24H45V58H7Z\" fill=\"url(#switch-rocker-{index})\"></path><path class=\"switch-rocker-top\" d=\"M3 20H49L45 24H7Z\"></path><path class=\"switch-rocker-highlight\" d=\"M8 25H9V56H8Z\"></path><path class=\"switch-rocker-foot\" d=\"M7 58H45L49 60H3Z\"></path><path class=\"switch-sheen\" d=\"M7 24H45V58H7Z\" fill-opacity=\"{flat_sheen:.3}\"></path><path class=\"switch-sheen\" d=\"M3 20H49L45 24H7Z\" fill-opacity=\"{top_sheen:.3}\"></path><path class=\"switch-sheen\" d=\"M3 20L7 24V58L3 60Z\" fill-opacity=\"{left_sheen:.3}\"></path><path class=\"switch-sheen\" d=\"M49 20L45 24V58L49 60Z\" fill-opacity=\"{right_sheen:.3}\"></path><path class=\"switch-sheen\" d=\"M7 58H45L49 60H3Z\" fill-opacity=\"{foot_sheen:.3}\"></path><path class=\"led-spill led-spill-a\" d=\"M3 20H49L45 24H7Z\" fill=\"url(#led-spill-a-{index})\"></path>{spill_b_path}</g><path class=\"switch-block-highlight\" d=\"M4 -3H48M4 -3V19\"></path>{dust}</g></svg>"
    )
}

#[cfg(any(target_arch = "wasm32", test))]
/// The nameplate: one polished metal plate with engraved matte-black lettering.
///
/// Its two highlights are Ward's anisotropic specular lobe on a flat plate:
/// with the light and the eye at a distance D, tan(theta_h) grows as r / 2D,
/// so exp(-tan^2(theta_h) (cos^2(phi) / ax^2 + sin^2(phi) / ay^2)) is an
/// elliptical Gaussian. The stops bake that Gaussian (t = 1 is three sigma);
/// the lobes sit towards and away from the panel light, and the engraving
/// catches it on the side facing away.
fn identity_plaque_svg() -> String {
    let (light_x, light_y) = light::toward_light();
    let (shadow_x, shadow_y) = light::shadow_direction();
    let metal = light::gradient_attributes(0.0, 0.5);
    let (glint_x, glint_y) = (210.0 + light_x * 33.94, 52.0 + light_y * 25.46);
    let (low_x, low_y) = (210.0 + shadow_x * 127.3, 52.0 + shadow_y * 48.1);
    let (edge_x, edge_y) = (shadow_x * 0.8, shadow_y * 0.8);
    let main_stops = light::ward_lobe_stops(0.62, 16);
    let low_stops = light::ward_lobe_stops(0.22, 16);
    // The plate's edge rolls away in a soft bevel: each side of it catches
    // the light in proportion to how far it faces it, so the ring is drawn
    // twice, across and down, each graded from lit to shaded by that
    // component of the light, then softened and kept inside the plate.
    let bevel = |component: f64, (x1, y1, x2, y2): (u8, u8, u8, u8)| {
        let (lit, shaded) = (0.3 * component.abs(), 0.22 * component.abs());
        let (x1, y1, x2, y2) = if component < 0.0 {
            (x1, y1, x2, y2)
        } else {
            (x2, y2, x1, y1)
        };
        format!(
            "x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\"><stop offset=\"0\" stop-color=\"#ffffff\" stop-opacity=\"{lit:.3}\"></stop><stop offset=\"0.5\" stop-color=\"#ffffff\" stop-opacity=\"0\"></stop><stop offset=\"0.5\" stop-color=\"#26282c\" stop-opacity=\"0\"></stop><stop offset=\"1\" stop-color=\"#26282c\" stop-opacity=\"{shaded:.3}\"></stop>"
        )
    };
    let bevel_across = bevel(light_x, (0, 0, 1, 0));
    let bevel_down = bevel(light_y, (0, 0, 0, 1));
    format!(
        "<svg class=\"identity-plaque\" viewBox=\"0 0 420 104\" role=\"img\" aria-label=\"RackForge Instruments RF-5, five-voice programmable polyphonic synthesizer\"><defs><linearGradient id=\"plaque-metal\" {metal}><stop offset=\"0\" stop-color=\"#d8dadd\"></stop><stop offset=\"0.3\" stop-color=\"#c9ccd0\"></stop><stop offset=\"0.55\" stop-color=\"#cdd0d3\"></stop><stop offset=\"0.8\" stop-color=\"#b3b6ba\"></stop><stop offset=\"1\" stop-color=\"#bdc0c4\"></stop></linearGradient><radialGradient id=\"plaque-glint\" gradientUnits=\"userSpaceOnUse\" cx=\"0\" cy=\"0\" r=\"1\" gradientTransform=\"translate({glint_x:.2} {glint_y:.2}) rotate(-3) scale(250 50)\">{main_stops}</radialGradient><radialGradient id=\"plaque-glint-low\" gradientUnits=\"userSpaceOnUse\" cx=\"0\" cy=\"0\" r=\"1\" gradientTransform=\"translate({low_x:.2} {low_y:.2}) rotate(-3) scale(230 40)\">{low_stops}</radialGradient><linearGradient id=\"plaque-bevel-across\" {bevel_across}</linearGradient><linearGradient id=\"plaque-bevel-down\" {bevel_down}</linearGradient><clipPath id=\"plaque-outline\"><rect x=\"2\" y=\"2\" width=\"416\" height=\"100\" rx=\"16\"></rect></clipPath><filter id=\"plaque-bevel-soft\" x=\"-5%\" y=\"-10%\" width=\"110%\" height=\"120%\"><feGaussianBlur stdDeviation=\"1.2\"></feGaussianBlur></filter></defs><rect class=\"plaque-shadow\" x=\"2\" y=\"2\" width=\"416\" height=\"100\" rx=\"16\"></rect><rect class=\"plaque-metal\" x=\"2\" y=\"2\" width=\"416\" height=\"100\" rx=\"16\"></rect><rect class=\"plaque-glint\" x=\"2\" y=\"2\" width=\"416\" height=\"100\" rx=\"16\"></rect><rect class=\"plaque-glint-low\" x=\"2\" y=\"2\" width=\"416\" height=\"100\" rx=\"16\"></rect><g class=\"plaque-bevel\" clip-path=\"url(#plaque-outline)\"><g filter=\"url(#plaque-bevel-soft)\"><rect x=\"3.5\" y=\"3.5\" width=\"413\" height=\"97\" rx=\"14.5\" stroke=\"url(#plaque-bevel-across)\"></rect><rect x=\"3.5\" y=\"3.5\" width=\"413\" height=\"97\" rx=\"14.5\" stroke=\"url(#plaque-bevel-down)\"></rect><rect class=\"plaque-bevel-foot\" x=\"2.6\" y=\"2.6\" width=\"414.8\" height=\"98.8\" rx=\"15.4\"></rect></g></g><g class=\"plaque-engraving-edge\" transform=\"translate({edge_x:.2} {edge_y:.2})\" aria-hidden=\"true\"><text class=\"plaque-brand\" x=\"210\" y=\"25\" text-anchor=\"middle\">RACKFORGE INSTRUMENTS</text><text class=\"plaque-model\" x=\"210\" y=\"67\" text-anchor=\"middle\">RF-5</text><text class=\"plaque-description\" x=\"210\" y=\"87\" text-anchor=\"middle\">FIVE-VOICE POLYPHONIC SYNTHESIZER</text></g><g class=\"plaque-engraving\"><text class=\"plaque-brand\" x=\"210\" y=\"25\" text-anchor=\"middle\">RACKFORGE INSTRUMENTS</text><text class=\"plaque-model\" x=\"210\" y=\"67\" text-anchor=\"middle\">RF-5</text><text class=\"plaque-description\" x=\"210\" y=\"87\" text-anchor=\"middle\">FIVE-VOICE POLYPHONIC SYNTHESIZER</text></g></svg>"
    )
}

#[cfg(any(target_arch = "wasm32", test))]
fn relative_knob_value(
    start_value: f64,
    delta_y: f64,
    minimum: f64,
    maximum: f64,
    step: f64,
) -> f64 {
    if !start_value.is_finite()
        || !delta_y.is_finite()
        || !minimum.is_finite()
        || !maximum.is_finite()
        || maximum <= minimum
    {
        return minimum;
    }
    let raw = start_value + delta_y / 180.0 * (maximum - minimum);
    if step.is_finite() && step > 0.0 {
        (minimum + ((raw - minimum) / step).round() * step).clamp(minimum, maximum)
    } else {
        raw.clamp(minimum, maximum)
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn keyboard_knob_value(
    current: f64,
    key: &str,
    minimum: f64,
    maximum: f64,
    step: f64,
) -> Option<f64> {
    if !current.is_finite() || !minimum.is_finite() || !maximum.is_finite() || maximum <= minimum {
        return None;
    }
    let increment = if step.is_finite() && step > 0.0 {
        step
    } else {
        (maximum - minimum) / 127.0
    };
    let value = match key {
        "ArrowUp" | "ArrowRight" => current + increment,
        "ArrowDown" | "ArrowLeft" => current - increment,
        "PageUp" => current + increment * 10.0,
        "PageDown" => current - increment * 10.0,
        "Home" => minimum,
        "End" => maximum,
        _ => return None,
    };
    Some(value.clamp(minimum, maximum))
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(untagged)]
enum ParameterDefault {
    Number(f64),
    Boolean(bool),
}

#[cfg(any(target_arch = "wasm32", test))]
impl ParameterDefault {
    const fn as_f64(self) -> f64 {
        match self {
            Self::Number(value) => value,
            Self::Boolean(value) => value as u8 as f64,
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use js_sys::{Object, Reflect};
    use serde::{Deserialize, Serialize};
    use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
    use web_sys::{
        Document, Element, Event, HtmlImageElement, KeyboardEvent, MessageEvent, MouseEvent,
        PointerEvent, Window,
    };

    type AppHandle = Rc<RefCell<App>>;
    type ResponseHandler = Box<dyn FnOnce(&AppHandle, Result<JsValue, String>)>;

    #[derive(Debug, Deserialize)]
    struct HostContext {
        instance: Instance,
    }

    /// The panel only follows which program is selected, to re-read its
    /// controls; RackForge's program selector lists and chooses them.
    #[derive(Debug, Deserialize)]
    struct Instance {
        selected_sound_id: String,
        #[serde(default)]
        sounds: Vec<Sound>,
    }

    /// A program as the context lists it: enough for its display digits.
    #[derive(Debug, Deserialize)]
    struct Sound {
        id: String,
        name: String,
        #[serde(default)]
        bank: Option<String>,
        #[serde(default)]
        editable: bool,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct ParameterSnapshot {
        schema: ParameterSchema,
        values: Vec<ParameterValue>,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct ParameterSchema {
        parameters: Vec<Parameter>,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct Parameter {
        index: u32,
        id: String,
        name: String,
        kind: ParameterKind,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct ParameterKind {
        #[serde(rename = "type")]
        kind: String,
        minimum: Option<f64>,
        maximum: Option<f64>,
        default: Option<ParameterDefault>,
        step: Option<f64>,
    }

    impl ParameterKind {
        fn default_value(&self) -> f64 {
            self.default.map(ParameterDefault::as_f64).unwrap_or(0.0)
        }
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    struct ParameterValue {
        index: u32,
        value: f64,
    }

    #[derive(Serialize)]
    struct Request<'a> {
        protocol: &'static str,
        kind: &'static str,
        request_id: &'a str,
        method: &'a str,
        params: serde_json::Value,
    }

    #[derive(Serialize)]
    struct Ready {
        protocol: &'static str,
        kind: &'static str,
    }

    /// Switch drawings number their gradients by index; the page keys take
    /// their own range, clear of the parameters'.
    const PAGE_SWITCH_INDEX: u32 = 1_000;
    const STEP_SWITCH_INDEX: u32 = 1_100;
    const RECORD_SWITCH_INDEX: u32 = 1_102;
    const SAVE_SWITCH_INDEX: u32 = 1_103;

    struct App {
        window: Window,
        document: Document,
        root: Element,
        /// RackForge's `<rf-program-select>`, made once and put back in the
        /// program memory bar after every render: the render replaces the
        /// whole page, and a new element each time would lose an open list.
        program_selector: Element,
        host_origin: String,
        context: Option<HostContext>,
        snapshot: Option<ParameterSnapshot>,
        parameter_values: BTreeMap<u32, f64>,
        pending: BTreeMap<String, ResponseHandler>,
        sequence: u64,
        refresh_generation: u64,
        active_section: String,
        active_parameter_drag: Option<u32>,
        render_after_drag: bool,
        bridge_error: String,
        /// The lettering and the walnut have loaded.
        assets_loaded: bool,
        /// The page has been faded in.
        revealed: bool,
        /// RECORD is lit: the program save dialog is open.
        record_armed: bool,
    }

    impl App {
        fn new() -> Result<AppHandle, JsValue> {
            let window = web_sys::window().ok_or_else(|| JsValue::from_str("missing window"))?;
            let document = window
                .document()
                .ok_or_else(|| JsValue::from_str("missing document"))?;
            let root = document
                .get_element_by_id("plugin-root")
                .ok_or_else(|| JsValue::from_str("missing #plugin-root"))?;
            let host_origin = window.location().origin()?;
            let program_selector = document.create_element("rf-program-select")?;
            for (name, value) in [
                ("id", "program-selector"),
                ("label", "Program"),
                ("placeholder", "Search RF-5 programs"),
                ("empty-label", "Waiting for programs…"),
            ] {
                program_selector.set_attribute(name, value)?;
            }
            // Its steps are the panel's light switches, named under them, in
            // the element's arrow slots; pressed like any switch of the panel.
            program_selector.set_inner_html(&format!(
                "<span slot=\"prev\" class=\"hardware-button tune-button program-step\">{}</span><span slot=\"next\" class=\"hardware-button tune-button program-step\">{}</span>",
                step_key_svg(STEP_SWITCH_INDEX, Step::Previous),
                step_key_svg(STEP_SWITCH_INDEX + 1, Step::Next),
            ));
            // RackForge's program save dialog, opened by RECORD. It lives
            // outside the page the panel redraws, so it keeps an open dialog
            // and a half-typed name; its buttons are the panel's switches.
            let program_save = document.create_element("rf-program-save")?;
            for (name, value) in [
                ("id", "program-save"),
                ("heading", "RECORD PROGRAM"),
                ("label", "NAME"),
                ("placeholder", "PROGRAM NAME"),
                ("current-label", "CURRENT PROGRAM:"),
                ("busy-label", "RECORDING…"),
                ("default-name", "NEW PROGRAM"),
            ] {
                program_save.set_attribute(name, value)?;
            }
            program_save.set_inner_html(&format!(
                "<span slot=\"cancel\" class=\"hardware-button program-step\">{}<span class=\"control-label\">CANCEL</span></span><span slot=\"replace\" class=\"hardware-button tune-button program-step\">{}<span class=\"control-label\">REPLACE</span></span><span slot=\"save\" class=\"hardware-button record-button program-step\">{}<span class=\"control-label\">SAVE NEW</span></span>",
                switch_svg(SAVE_SWITCH_INDEX, false, None, false, Some("")),
                switch_svg(SAVE_SWITCH_INDEX + 1, false, None, true, Some("")),
                switch_svg(SAVE_SWITCH_INDEX + 2, false, None, false, Some("")),
            ));
            if let Some(body) = document.body() {
                body.append_child(&program_save)?;
            }
            Ok(Rc::new(RefCell::new(Self {
                window,
                document,
                root,
                program_selector,
                host_origin,
                context: None,
                snapshot: None,
                parameter_values: BTreeMap::new(),
                pending: BTreeMap::new(),
                sequence: 0,
                refresh_generation: 0,
                active_section: "modulation".to_owned(),
                active_parameter_drag: None,
                render_after_drag: false,
                bridge_error: String::new(),
                assets_loaded: false,
                revealed: false,
                record_armed: false,
            })))
        }

        fn value(&self, parameter: &Parameter) -> f64 {
            self.parameter_values
                .get(&parameter.index)
                .copied()
                .unwrap_or_else(|| parameter.kind.default_value())
        }

        fn render(&self) {
            // The head: the nameplate on its walnut rail. Then the panel
            // face, one sheet under the program memory, the page keys and the
            // controls. Under it only a thin strip of the case shows.
            let mut html = String::from("<div class=\"rf5-frame\">");
            html.push_str("<div class=\"wood-rail wood-rail-plaque\">");
            html.push_str(&identity_plaque_svg());
            html.push_str("</div>");
            html.push_str("<div class=\"panel-face\">");
            html.push_str(&self.render_program_memory());
            html.push_str(&self.render_tabs());
            html.push_str(&self.render_panel());
            html.push_str("</div>");
            html.push_str("<div class=\"wood-rail wood-rail-bottom\" aria-hidden=\"true\"></div>");
            html.push_str("</div>");
            self.root.set_inner_html(&html);
            if let Some(slot) = self.document.get_element_by_id("program-selector-slot") {
                let _ = slot.append_child(&self.program_selector);
            }
            // The digits a narrow display shows in place of the name.
            let _ = self.program_selector.set_attribute(
                "style",
                &format!("--rf5-digits: \"{}\"", self.program_digits()),
            );
            layout_group_outlines(&self.root);
        }

        /// The panel's pages, chosen with a row of the panel's own switches:
        /// the lit LED marks the page shown, and a PAGE legend runs under
        /// the row.
        /// The selected program's display digits (see `program_digits`).
        fn program_digits(&self) -> String {
            let Some(instance) = self.context.as_ref().map(|context| &context.instance) else {
                return "--".to_owned();
            };
            let Some(sound) = instance
                .sounds
                .iter()
                .find(|sound| sound.id == instance.selected_sound_id)
            else {
                return "--".to_owned();
            };
            let user_place = sound.editable.then(|| {
                instance
                    .sounds
                    .iter()
                    .filter(|other| other.editable)
                    .position(|other| other.id == sound.id)
                    .map_or(0, |place| place + 1)
            });
            program_digits(&sound.name, sound.bank.as_deref(), user_place)
        }

        fn render_tabs(&self) -> String {
            let mut keys = String::new();
            for (position, section) in panel::SECTIONS.iter().enumerate() {
                let active = section.id == self.active_section;
                let switch =
                    prophet_switch_svg(PAGE_SWITCH_INDEX + position as u32, active, None, false);
                keys.push_str(&format!(
                    "<div class=\"page-key\"><span class=\"control-label\"><span class=\"label-full\">{label}</span><span class=\"label-short\">{short}</span></span><button type=\"button\" class=\"hardware-button page-button{}\" data-action=\"section\" data-section=\"{}\" aria-pressed=\"{active}\" aria-label=\"{label}: {caption}\" title=\"{caption}\">{switch}</button></div>",
                    if active { " active" } else { "" },
                    section.id,
                    label = section.label,
                    short = section.short,
                    caption = section.caption,
                ));
            }
            format!(
                "<nav class=\"panel-tabs control-legend\" aria-label=\"RF-5 panel sections\"><div class=\"legend-controls\">{keys}</div><div class=\"legend-line\"><svg class=\"legend-rule\" aria-hidden=\"true\" preserveAspectRatio=\"none\"><path></path></svg><span>PAGE</span></div></nav>"
            )
        }

        fn render_panel(&self) -> String {
            let Some(snapshot) = self.snapshot.as_ref() else {
                let message = if self.bridge_error.is_empty() {
                    "Reading the front-panel state…"
                } else {
                    &self.bridge_error
                };
                return format!(
                    "<section class=\"hardware-panel loading\"><p>{}</p><button type=\"button\" data-action=\"retry\">RETRY PANEL</button></section>",
                    escape_html(message)
                );
            };
            let section = panel::section(&self.active_section);
            let mut groups = String::new();
            for group in section.groups {
                let render_ids = |ids: &[&str]| {
                    let mut html = String::new();
                    for id in ids {
                        if let Some(parameter) = snapshot
                            .schema
                            .parameters
                            .iter()
                            .find(|parameter| parameter.id == *id)
                        {
                            html.push_str(&self.render_control(parameter));
                        }
                    }
                    html
                };
                let mut controls = String::new();
                let mut ids = group.parameter_ids;
                while let Some(id) = ids.first() {
                    if let Some(legend) = group
                        .legends
                        .iter()
                        .find(|legend| legend.parameter_ids.first() == Some(id))
                    {
                        let controls_html = render_ids(legend.parameter_ids);
                        if legend.label.is_empty() {
                            controls.push_str(&format!(
                                "<div class=\"control-legend control-cluster\"><div class=\"legend-controls\">{controls_html}</div></div>"
                            ));
                            ids = &ids[legend.parameter_ids.len()..];
                            continue;
                        }
                        controls.push_str(&format!(
                            "<div class=\"control-legend legend-{} legend-{}\" style=\"--legend-span:{}\"><div class=\"legend-controls\">{}</div><div class=\"legend-line\"><svg class=\"legend-rule\" aria-hidden=\"true\" preserveAspectRatio=\"none\"><path></path></svg><span>{}</span></div></div>",
                            legend.parameter_ids.len(),
                            legend.label.to_ascii_lowercase().replace(' ', "-"),
                            legend.parameter_ids.len(),
                            controls_html,
                            legend.label
                        ));
                        ids = &ids[legend.parameter_ids.len()..];
                    } else {
                        controls.push_str(&render_ids(&ids[..1]));
                        ids = &ids[1..];
                    }
                }
                groups.push_str(&format!(
                    "<section class=\"control-group group-{}\"><svg class=\"section-outline\" aria-hidden=\"true\" preserveAspectRatio=\"none\"><path></path></svg><h2><span>{}</span></h2><div class=\"control-grid\">{controls}</div></section>",
                    group.id,
                    group.title
                ));
            }
            let error = if self.bridge_error.is_empty() {
                String::new()
            } else {
                format!(
                    "<p class=\"bridge-error\">{}</p>",
                    escape_html(&self.bridge_error)
                )
            };
            format!(
                "<main class=\"hardware-panel section-{}\"><div class=\"panel-surface\">{groups}</div>{error}</main>",
                section.id
            )
        }

        fn render_control(&self, parameter: &Parameter) -> String {
            let value = self.value(parameter);
            if parameter.kind.kind == "boolean" {
                self.render_button(parameter, value)
            } else {
                self.render_knob(parameter, value)
            }
        }

        fn render_button(&self, parameter: &Parameter, value: f64) -> String {
            let active = value >= 0.5;
            let symbol = waveform_symbol(&parameter.id);
            // With no figure over it, the name sits right on the switch; the
            // wave shapes, as on the original, go by their figures alone.
            let bare = if symbol.is_empty() { " bare" } else { "" };
            let label = if !symbol.is_empty() {
                String::new()
            } else {
                panel_label(&parameter.id, &parameter.name)
            };
            let switch = prophet_switch_svg(parameter.index, active, None, parameter.id == "tune");
            let action = if parameter.id == "tune" {
                "momentary"
            } else {
                "toggle"
            };
            format!(
                "<div class=\"parameter-control button-control{bare} parameter-{}\"><span class=\"control-label\">{}</span><span class=\"wave-symbol-slot\" aria-hidden=\"true\">{symbol}</span><button type=\"button\" class=\"hardware-button{}{}\" data-action=\"{action}\" data-index=\"{}\" data-rackforge-parameter-index=\"{}\" aria-label=\"{}\" aria-pressed=\"{}\">{switch}</button><output data-output-index=\"{}\">{}</output></div>",
                parameter.id,
                label,
                if active { " active" } else { "" },
                if parameter.id == "tune" {
                    " tune-button"
                } else {
                    ""
                },
                parameter.index,
                parameter.index,
                escape_html(&parameter.name),
                active,
                parameter.index,
                if active { "ON" } else { "OFF" }
            )
        }

        fn render_knob(&self, parameter: &Parameter, value: f64) -> String {
            let minimum = parameter.kind.minimum.unwrap_or(0.0);
            let maximum = parameter.kind.maximum.unwrap_or(1.0);
            let step = parameter.kind.step.unwrap_or(0.0);
            let normalized = normalized(value, minimum, maximum);
            let angle = -135.0 + normalized * 270.0;
            let ticks = knob_ticks(parameter.id == "wheel-mod-source-mix");
            let dust = knob_dust_layer(parameter.index);
            format!(
                "<div class=\"parameter-control knob-control parameter-{}\"><span class=\"control-label\">{}</span><div class=\"knob-shell\" data-knob-index=\"{}\" data-rackforge-parameter-index=\"{}\" style=\"--knob-turn:{angle:.3}deg\"><svg class=\"knob-scale\" viewBox=\"0 0 100 100\" aria-hidden=\"true\">{ticks}{dust}</svg><span class=\"knob-shadow\" aria-hidden=\"true\"></span><span class=\"knob-cap\" aria-hidden=\"true\"><span class=\"knob-top\"></span><span class=\"knob-marker\"></span></span><input class=\"knob-input\" type=\"range\" data-action=\"parameter\" data-index=\"{}\" min=\"{minimum}\" max=\"{maximum}\" step=\"{step}\" value=\"{value}\" aria-label=\"{}\"></div><output data-output-index=\"{}\">{}</output></div>",
                parameter.id,
                panel_label(&parameter.id, &parameter.name),
                parameter.index,
                parameter.index,
                parameter.index,
                escape_html(&parameter.name),
                parameter.index,
                format_value(parameter, value)
            )
        }

        /// The program memory, under the nameplate: its maker and RackForge's
        /// program selector, put in its slot after the page is drawn. The
        /// selector's name carries the bank and number ("1-2 Low Strings").
        /// The program memory, under the nameplate: the RECORD switch, lit
        /// while a program is being recorded, then RackForge's program
        /// selector, put in its slot after the page is drawn.
        fn render_program_memory(&self) -> String {
            let armed = self.record_armed;
            format!(
                "<section class=\"program-memory\"><button type=\"button\" class=\"hardware-button program-step record-button\" data-action=\"record\" aria-pressed=\"{armed}\" aria-label=\"Record program\">{}<span class=\"control-label\" aria-hidden=\"true\">RECORD</span></button><div class=\"program-selector-slot\" id=\"program-selector-slot\"></div></section>",
                switch_svg(RECORD_SWITCH_INDEX, armed, None, false, None)
            )
        }
    }

    fn layout_group_outlines(root: &Element) {
        const LABEL_PADDING: f64 = 7.0;
        const CORNER_RADIUS: f64 = 15.0;

        for section in panel::SECTIONS {
            for group in section.groups {
                let selector = format!(".control-group.group-{}", group.id);
                let Ok(Some(container)) = root.query_selector(&selector) else {
                    continue;
                };
                let Ok(Some(label)) = container.query_selector("h2 span") else {
                    continue;
                };
                let Ok(Some(outline)) = container.query_selector(".section-outline") else {
                    continue;
                };
                let Ok(Some(path)) = outline.query_selector("path") else {
                    continue;
                };

                let container_rect = container.get_bounding_client_rect();
                let label_rect = label.get_bounding_client_rect();
                let width = container_rect.width();
                let height = container_rect.height();
                let label_start = label_rect.left() - container_rect.left() - LABEL_PADDING;
                let label_end = label_rect.right() - container_rect.left() + LABEL_PADDING;
                let Some(path_data) =
                    section_outline_path(width, height, label_start, label_end, CORNER_RADIUS)
                else {
                    continue;
                };

                let _ = outline.set_attribute("viewBox", &format!("0 0 {width:.2} {height:.2}"));
                let _ = path.set_attribute("d", &path_data);
            }
        }
        layout_legend_rules(root);
    }

    /// Draws each legend's rule from the middle of its first control (a
    /// switch or a knob) to the middle of its last, broken round the
    /// legend's name, which it centres between them.
    fn layout_legend_rules(root: &Element) {
        const LABEL_PADDING: f64 = 6.0;

        let Ok(legends) = root.query_selector_all(".control-legend") else {
            return;
        };
        for index in 0..legends.length() {
            let Some(legend) = legends
                .item(index)
                .and_then(|node| node.dyn_into::<Element>().ok())
            else {
                continue;
            };
            let (Ok(Some(line)), Ok(Some(label)), Ok(Some(rule)), Ok(Some(path)), Ok(switches)) = (
                legend.query_selector(".legend-line"),
                legend.query_selector(".legend-line span"),
                legend.query_selector(".legend-rule"),
                legend.query_selector(".legend-rule path"),
                legend.query_selector_all(".hardware-button, .knob-shell"),
            ) else {
                continue;
            };
            let centre = |position: u32| {
                switches
                    .item(position)
                    .and_then(|node| node.dyn_into::<Element>().ok())
                    .map(|switch| {
                        let rect = switch.get_bounding_client_rect();
                        rect.left() + rect.width() / 2.0
                    })
            };
            let (Some(first), Some(last)) =
                (centre(0), centre(switches.length().saturating_sub(1)))
            else {
                continue;
            };
            let line_rect = line.get_bounding_client_rect();
            let label_width = label.get_bounding_client_rect().width();
            let (width, height) = (line_rect.width(), line_rect.height());
            // The name sits halfway between the first control and the last,
            // however unevenly their columns share the line.
            let middle = (first + last) / 2.0 - line_rect.left();
            let _ = label.set_attribute("style", &format!("left:{:.2}px", middle - width / 2.0));
            let Some(path_data) = legend_rule_path(
                first - line_rect.left(),
                last - line_rect.left(),
                middle - label_width / 2.0 - LABEL_PADDING,
                middle + label_width / 2.0 + LABEL_PADDING,
                height / 2.0,
            ) else {
                continue;
            };
            let _ = rule.set_attribute("viewBox", &format!("0 0 {width:.2} {height:.2}"));
            let _ = path.set_attribute("d", &path_data);
        }
    }

    fn normalized(value: f64, minimum: f64, maximum: f64) -> f64 {
        if maximum > minimum {
            ((value - minimum) / (maximum - minimum)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// The dial: eleven ticks over the knob's 270-degree throw, each running
    /// in to the knob's edge (radius 31 of 100 for its 52 px on the 86 px
    /// scale), and a number 0-10 just outside each, on the tick's bearing.
    /// A mix knob (Wheel-Mod's source mix) reads 5-0-5 instead, from LFO at
    /// its left end to NOISE at its right, as on the Prophet-5.
    fn knob_ticks(mix: bool) -> String {
        const NUMBER_RADIUS: f64 = 55.0;
        let mut ticks = String::new();
        if mix {
            ticks.push_str(
                "<text class=\"knob-end\" x=\"3\" y=\"98\" text-anchor=\"end\">LFO</text><text class=\"knob-end\" x=\"97\" y=\"98\" text-anchor=\"start\">NOISE</text>",
            );
        }
        for index in 0..=10_i32 {
            let number: i32 = if mix { (index - 5).abs() } else { index };
            let angle = -135.0 + index as f64 * 27.0;
            let class = if matches!(index, 0 | 5 | 10) {
                " major"
            } else {
                ""
            };
            let radians = f64::to_radians(angle);
            let (x, y) = (
                50.0 + NUMBER_RADIUS * radians.sin(),
                50.0 - NUMBER_RADIUS * radians.cos(),
            );
            ticks.push_str(&format!(
                "<line class=\"knob-tick{class}\" x1=\"50\" y1=\"3\" x2=\"50\" y2=\"19\" transform=\"rotate({angle} 50 50)\"></line><text class=\"knob-number\" x=\"{x:.1}\" y=\"{y:.1}\">{number}</text>"
            ));
        }
        ticks
    }

    fn panel_label(id: &str, fallback: &str) -> String {
        let label = match id {
            "poly-mod-filter-envelope-amount" => "FILTER ENV",
            "poly-mod-oscillator-b-amount" => "OSC B",
            "poly-mod-oscillator-a-frequency" => "FREQ A",
            "poly-mod-oscillator-a-pulse-width" => "PW A",
            "wheel-mod-source-mix" => "SOURCE MIX",
            "wheel-mod-oscillator-a-frequency" => "FREQ A",
            "wheel-mod-oscillator-b-frequency" => "FREQ B",
            "wheel-mod-oscillator-a-pulse-width" => "PW A",
            "wheel-mod-oscillator-b-pulse-width" => "PW B",
            "wheel-mod-filter" => "FILTER",
            "oscillator-a-frequency" | "oscillator-b-frequency" | "lfo-frequency" => "FREQUENCY",
            "oscillator-b-detune" => "FINE",
            "oscillator-a-pulse-width" | "oscillator-b-pulse-width" => "PULSE WIDTH",
            "oscillator-b-low-frequency" => "LO FREQ",
            "oscillator-b-keyboard" | "filter-keyboard" => "KEYBOARD",
            "oscillator-a-level" => "OSC A",
            "oscillator-b-level" => "OSC B",
            "filter-envelope-amount" => "ENV AMOUNT",
            "master-volume" => "VOLUME",
            "vintage-spread" => "VOICE SPREAD",
            "release-enable" => "RELEASE",
            "program-change-mutes-tails" => "CUT TAILS",
            _ if id.ends_with("-saw") => "SAW",
            _ if id.ends_with("-triangle") => "TRIANGLE",
            _ if id.ends_with("-square") => "SQUARE",
            _ if id.ends_with("-pulse") => "PULSE",
            _ => fallback,
        };
        escape_html(label)
    }

    fn waveform_symbol(id: &str) -> &'static str {
        if id.ends_with("-saw") {
            "<svg class=\"wave-symbol\" viewBox=\"0 0 42 32\" aria-hidden=\"true\"><path d=\"M5 29L35 3V29\"></path></svg>"
        } else if id.ends_with("-triangle") {
            "<svg class=\"wave-symbol\" viewBox=\"0 0 42 32\" aria-hidden=\"true\"><path d=\"M3 29L21 3L39 29\"></path></svg>"
        } else if id.ends_with("-square") || id.ends_with("-pulse") {
            "<svg class=\"wave-symbol\" viewBox=\"0 0 42 32\" aria-hidden=\"true\"><path d=\"M1 29H8V3H34V29H41\"></path></svg>"
        } else {
            ""
        }
    }

    fn format_value(parameter: &Parameter, value: f64) -> String {
        if parameter.id.starts_with("scale-") {
            let cents = (value * 127.0 - 64.0) * 100.0 / 128.0;
            return format!("{cents:+.1}¢");
        }
        if parameter.kind.kind == "boolean" {
            if parameter.id == "tune" {
                return if value >= 0.5 { "TUNING" } else { "READY" }.to_owned();
            }
            return if value >= 0.5 { "ON" } else { "OFF" }.to_owned();
        }
        format!("{:.1}", value * 10.0)
    }

    fn request(
        app: &AppHandle,
        method: &str,
        params: serde_json::Value,
        handler: impl FnOnce(&AppHandle, Result<JsValue, String>) + 'static,
    ) {
        let (id, window, origin) = {
            let mut state = app.borrow_mut();
            state.sequence += 1;
            let id = format!("rf-5-ui-{}", state.sequence);
            state.pending.insert(id.clone(), Box::new(handler));
            (id, state.window.clone(), state.host_origin.clone())
        };
        let message = Request {
            protocol: PROTOCOL,
            kind: "request",
            request_id: &id,
            method,
            params,
        };
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        let message = match message.serialize(&serializer) {
            Ok(message) => message,
            Err(error) => {
                resolve(app, &id, Err(error.to_string()));
                return;
            }
        };
        match window.parent().ok().flatten() {
            Some(parent) => {
                if let Err(error) = parent.post_message(&message, &origin) {
                    resolve(app, &id, Err(format!("postMessage failed: {error:?}")));
                    return;
                }
            }
            None => {
                resolve(
                    app,
                    &id,
                    Err("RackForge parent window is missing.".to_owned()),
                );
                return;
            }
        }
        let weak = Rc::downgrade(app);
        let timeout_id = id.clone();
        let timeout = Closure::once_into_js(move || {
            if let Some(app) = weak.upgrade() {
                resolve(
                    &app,
                    &timeout_id,
                    Err("RackForge did not answer in time.".to_owned()),
                );
            }
        });
        let _ = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(timeout.unchecked_ref(), 4_000);
    }

    fn resolve(app: &AppHandle, id: &str, result: Result<JsValue, String>) {
        let handler = app.borrow_mut().pending.remove(id);
        if let Some(handler) = handler {
            handler(app, result);
        }
    }

    fn refresh_parameters(app: &AppHandle) {
        if app.borrow().context.is_none() {
            return;
        }
        let generation = {
            let mut state = app.borrow_mut();
            state.refresh_generation += 1;
            state.refresh_generation
        };
        request(
            app,
            "plugin.parameters",
            serde_json::json!({}),
            move |app, result| {
                if app.borrow().refresh_generation != generation {
                    return;
                }
                match result.and_then(|value| {
                    serde_wasm_bindgen::from_value::<ParameterSnapshot>(value)
                        .map_err(|error| error.to_string())
                }) {
                    Ok(snapshot) => {
                        let values = snapshot
                            .values
                            .iter()
                            .map(|value| (value.index, value.value))
                            .collect();
                        let mut state = app.borrow_mut();
                        state.snapshot = Some(snapshot);
                        state.parameter_values = values;
                        state.bridge_error.clear();
                    }
                    Err(error) => app.borrow_mut().bridge_error = error,
                }
                if app.borrow().active_parameter_drag.is_some() {
                    app.borrow_mut().render_after_drag = true;
                } else {
                    app.borrow().render();
                }
                maybe_reveal(app);
            },
        );
    }

    /// RECORD opens RackForge's program save dialog, when the host has
    /// given the page its plugin kit.
    fn open_program_save(app: &AppHandle) {
        let document = app.borrow().document.clone();
        let Some(dialog) = document.get_element_by_id("program-save") else {
            return;
        };
        if let Ok(open) = Reflect::get(&dialog, &JsValue::from_str("open"))
            && let Some(open) = open.dyn_ref::<js_sys::Function>()
        {
            let _ = open.call0(&dialog);
        }
    }

    /// RECORD's LED follows the dialog: lit while it is open.
    fn follow_program_save(app: &AppHandle) -> Result<(), JsValue> {
        let document = app.borrow().document.clone();
        let Some(dialog) = document.get_element_by_id("program-save") else {
            return Ok(());
        };
        for (event_name, armed) in [
            ("rf-program-save-open", true),
            ("rf-program-save-close", false),
        ] {
            let lamp_app = app.clone();
            let follow = Closure::<dyn FnMut(Event)>::new(move |_: Event| {
                if lamp_app.borrow().record_armed == armed {
                    return;
                }
                lamp_app.borrow_mut().record_armed = armed;
                lamp_app.borrow().render();
            });
            dialog.add_event_listener_with_callback(event_name, follow.as_ref().unchecked_ref())?;
            follow.forget();
        }
        Ok(())
    }

    /// A key is pressed and the whole page is drawn anew, the key with it:
    /// draw the new key still down, then let it come back up at the switch's
    /// own pace, as a released switch does.
    fn release_key(app: &AppHandle, selector: &str) {
        let (document, window) = {
            let state = app.borrow();
            (state.document.clone(), state.window.clone())
        };
        let Ok(Some(key)) = document.query_selector(selector) else {
            return;
        };
        let _ = key.class_list().add_1("releasing");
        let release = Closure::once_into_js(move || {
            let _ = key.class_list().remove_1("releasing");
        });
        let _ = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(release.unchecked_ref(), 30);
    }

    /// The page stays hidden until everything it is drawn with is ready: the
    /// finish images, the lettering, the walnut and the panel's state
    /// (or the error saying why there is none). Then it fades in whole,
    /// never half-dressed.
    fn maybe_reveal(app: &AppHandle) {
        let ready = {
            let state = app.borrow();
            state.assets_loaded && (state.snapshot.is_some() || !state.bridge_error.is_empty())
        };
        if ready {
            reveal(app);
        }
    }

    fn reveal(app: &AppHandle) {
        let (root, window) = {
            let mut state = app.borrow_mut();
            if state.revealed {
                return;
            }
            state.revealed = true;
            (state.root.clone(), state.window.clone())
        };
        // A moment for the last render's images to decode before the fade.
        let show = Closure::once_into_js(move || {
            let _ = root.class_list().add_1("ready");
        });
        let _ =
            window.set_timeout_with_callback_and_timeout_and_arguments_0(show.unchecked_ref(), 60);
    }

    /// Loads the lettering, the walnut and the finish images the page is
    /// drawn with, so it is not shown before them. A failure counts as done:
    /// the page falls back rather than staying hidden.
    fn load_assets(app: &AppHandle) {
        let document = app.borrow().document.clone();
        let loads = js_sys::Array::new();
        let fonts = document.fonts();
        for font in ["700 12px Arimo", "400 12px Arimo", "12px Segment14"] {
            loads.push(&fonts.load(font));
        }
        let images = finish_paths().into_iter().chain([
            "assets/walnut-satin.jpg".to_owned(),
            "assets/walnut-satin-tall.jpg".to_owned(),
        ]);
        for path in images {
            if let Some(loaded) = image_loaded(&document, &path) {
                loads.push(&loaded);
            }
        }
        let settled = |app: AppHandle| {
            Closure::once(move |_: JsValue| {
                app.borrow_mut().assets_loaded = true;
                maybe_reveal(&app);
            })
        };
        let (loaded, failed) = (settled(app.clone()), settled(app.clone()));
        let _ = js_sys::Promise::all(&loads).then2(&loaded, &failed);
        loaded.forget();
        failed.forget();
    }

    /// Settles once the image at `path` has loaded, by its load event, not
    /// decode(): a page not on screen may never decode an image it does not
    /// show. The browser keeps it for the page's own use of the same path.
    fn image_loaded(document: &Document, path: &str) -> Option<js_sys::Promise> {
        let image = document
            .create_element("img")
            .ok()?
            .dyn_into::<HtmlImageElement>()
            .ok()?;
        let loaded = js_sys::Promise::new(&mut |resolve, reject| {
            image.set_onload(Some(&resolve));
            image.set_onerror(Some(&reject));
        });
        image.set_src(path);
        Some(loaded)
    }

    fn send_parameter(app: &AppHandle, index: u32, value: f64) {
        app.borrow_mut().parameter_values.insert(index, value);
        request(
            app,
            "plugin.set_parameter",
            serde_json::json!({ "parameter_index": index, "value": value }),
            move |app, result| {
                if let Err(error) = result {
                    app.borrow_mut().bridge_error = error;
                    refresh_parameters(app);
                }
            },
        );
    }

    fn update_parameter_dom(app: &AppHandle, index: u32) {
        let (document, parameter, value) = {
            let state = app.borrow();
            let Some(parameter) = state
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.schema.parameters.iter().find(|p| p.index == index))
                .cloned()
            else {
                return;
            };
            (
                state.document.clone(),
                parameter.clone(),
                state.value(&parameter),
            )
        };
        if let Ok(Some(input)) =
            document.query_selector(&format!("[data-action=parameter][data-index='{index}']"))
        {
            let _ = Reflect::set(
                input.as_ref(),
                &JsValue::from_str("value"),
                &JsValue::from_str(&value.to_string()),
            );
        }
        if let Ok(Some(output)) = document.query_selector(&format!("[data-output-index='{index}']"))
        {
            output.set_text_content(Some(&format_value(&parameter, value)));
        }
        if let Ok(Some(knob)) = document.query_selector(&format!("[data-knob-index='{index}']")) {
            let minimum = parameter.kind.minimum.unwrap_or(0.0);
            let maximum = parameter.kind.maximum.unwrap_or(1.0);
            let angle = -135.0 + normalized(value, minimum, maximum) * 270.0;
            let _ = knob.set_attribute("style", &format!("--knob-turn:{angle:.3}deg"));
        }
        if let Ok(Some(button)) =
            document.query_selector(&format!(".hardware-button[data-index='{index}']"))
        {
            let active = value >= 0.5;
            let _ = button.set_attribute("aria-pressed", if active { "true" } else { "false" });
            let _ = button.class_list().toggle_with_force("active", active);
            if let Ok(Some(control)) = button.closest(".button-control")
                && let Ok(Some(led)) = control.query_selector(".led")
            {
                let _ = led.set_attribute("class", if active { "led on" } else { "led" });
            }
        }
    }

    fn element_from_event(event: &Event) -> Option<Element> {
        event
            .target()?
            .dyn_into::<Element>()
            .ok()?
            .closest("[data-action]")
            .ok()
            .flatten()
    }

    fn knob_from_event(event: &PointerEvent) -> Option<(Element, Element)> {
        let surface = event
            .target()?
            .dyn_into::<Element>()
            .ok()?
            .closest("[data-knob-index]")
            .ok()
            .flatten()?;
        let input = surface.query_selector(".knob-input").ok().flatten()?;
        Some((input, surface))
    }

    fn numeric_value(element: &Element) -> Option<f64> {
        Reflect::get(element.as_ref(), &JsValue::from_str("value"))
            .ok()
            .and_then(|value| value.as_string())
            .and_then(|value| value.parse().ok())
    }

    fn update_knob_drag(
        app: &AppHandle,
        input: &Element,
        start_y: f64,
        current_y: f64,
        start_value: f64,
    ) {
        let minimum = input
            .get_attribute("min")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0.0);
        let maximum = input
            .get_attribute("max")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1.0);
        let step = input
            .get_attribute("step")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0.0);
        let value = relative_knob_value(start_value, start_y - current_y, minimum, maximum, step);
        let _ = Reflect::set(
            input.as_ref(),
            &JsValue::from_str("value"),
            &JsValue::from_str(&value.to_string()),
        );
        if let Some(index) = input
            .get_attribute("data-index")
            .and_then(|value| value.parse().ok())
        {
            send_parameter(app, index, value);
            update_parameter_dom(app, index);
        }
    }

    fn finish_drag(app: &AppHandle) {
        let rerender = {
            let mut state = app.borrow_mut();
            state.active_parameter_drag = None;
            std::mem::take(&mut state.render_after_drag)
        };
        if rerender {
            app.borrow().render();
        }
    }

    fn install_events(app: &AppHandle) -> Result<(), JsValue> {
        let click_app = app.clone();
        let click = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
            if event.button() != 0 {
                return;
            }
            let Some(element) = element_from_event(&event) else {
                return;
            };
            match element.get_attribute("data-action").as_deref() {
                Some("section") => {
                    if let Some(section) = element.get_attribute("data-section")
                        && panel::SECTIONS
                            .iter()
                            .any(|candidate| candidate.id == section)
                    {
                        click_app.borrow_mut().active_section = section.clone();
                        click_app.borrow().render();
                        release_key(
                            &click_app,
                            &format!(".page-button[data-section='{section}']"),
                        );
                    }
                }
                Some("record") => {
                    open_program_save(&click_app);
                    release_key(&click_app, "button.record-button");
                }
                Some("retry") => refresh_parameters(&click_app),
                Some("toggle") => {
                    if let Some(index) = element
                        .get_attribute("data-index")
                        .and_then(|value| value.parse().ok())
                    {
                        let value = if click_app
                            .borrow()
                            .parameter_values
                            .get(&index)
                            .copied()
                            .unwrap_or(0.0)
                            >= 0.5
                        {
                            0.0
                        } else {
                            1.0
                        };
                        send_parameter(&click_app, index, value);
                        update_parameter_dom(&click_app, index);
                    }
                }
                Some("momentary") => {
                    if let Some(index) = element
                        .get_attribute("data-index")
                        .and_then(|value| value.parse().ok())
                    {
                        send_parameter(&click_app, index, 1.0);
                        update_parameter_dom(&click_app, index);
                        let weak = Rc::downgrade(&click_app);
                        let refresh = Closure::once_into_js(move || {
                            if let Some(app) = weak.upgrade() {
                                refresh_parameters(&app);
                            }
                        });
                        let _ = click_app
                            .borrow()
                            .window
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                refresh.unchecked_ref(),
                                8_100,
                            );
                    }
                }
                _ => {}
            }
        });
        app.borrow()
            .root
            .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())?;
        click.forget();

        let input_app = app.clone();
        let input = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(element) = element_from_event(&event) else {
                return;
            };
            if element.get_attribute("data-action").as_deref() == Some("parameter")
                && let Some(index) = element
                    .get_attribute("data-index")
                    .and_then(|value| value.parse().ok())
                && let Some(value) = numeric_value(&element)
            {
                send_parameter(&input_app, index, value);
                update_parameter_dom(&input_app, index);
            }
        });
        app.borrow()
            .root
            .add_event_listener_with_callback("input", input.as_ref().unchecked_ref())?;
        app.borrow()
            .root
            .add_event_listener_with_callback("change", input.as_ref().unchecked_ref())?;
        input.forget();

        let keyboard_app = app.clone();
        let keyboard = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
            let Some(element) = element_from_event(&event) else {
                return;
            };
            if element.get_attribute("data-action").as_deref() != Some("parameter") {
                return;
            }
            let minimum = element
                .get_attribute("min")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0);
            let maximum = element
                .get_attribute("max")
                .and_then(|value| value.parse().ok())
                .unwrap_or(1.0);
            let step = element
                .get_attribute("step")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0);
            let Some(value) = keyboard_knob_value(
                numeric_value(&element).unwrap_or(minimum),
                &event.key(),
                minimum,
                maximum,
                step,
            ) else {
                return;
            };
            event.prevent_default();
            let _ = Reflect::set(
                element.as_ref(),
                &JsValue::from_str("value"),
                &JsValue::from_str(&value.to_string()),
            );
            if let Some(index) = element
                .get_attribute("data-index")
                .and_then(|value| value.parse().ok())
            {
                send_parameter(&keyboard_app, index, value);
                update_parameter_dom(&keyboard_app, index);
            }
        });
        app.borrow()
            .root
            .add_event_listener_with_callback("keydown", keyboard.as_ref().unchecked_ref())?;
        keyboard.forget();

        let drag_state = Rc::new(RefCell::new(None::<(i32, Element, Element, f64, f64)>));
        for event_name in [
            "pointerdown",
            "pointermove",
            "pointerup",
            "pointercancel",
            "lostpointercapture",
        ] {
            let drag_app = app.clone();
            let active = drag_state.clone();
            let drag = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
                let pointer_id = event.pointer_id();
                match event_name {
                    "pointerdown" => {
                        if !event.is_primary() || event.button() != 0 {
                            return;
                        }
                        let Some((input, surface)) = knob_from_event(&event) else {
                            return;
                        };
                        event.prevent_default();
                        let start_value = numeric_value(&input).unwrap_or(0.0);
                        let start_y = f64::from(event.client_y());
                        let index = input
                            .get_attribute("data-index")
                            .and_then(|value| value.parse().ok());
                        let _ = surface.set_pointer_capture(pointer_id);
                        let _ = input
                            .clone()
                            .dyn_into::<web_sys::HtmlElement>()
                            .map(|element| element.focus());
                        *active.borrow_mut() =
                            Some((pointer_id, input, surface, start_y, start_value));
                        if let Some(index) = index {
                            drag_app.borrow_mut().active_parameter_drag = Some(index);
                        }
                    }
                    "pointermove" => {
                        let drag = active.borrow().as_ref().and_then(
                            |(id, input, _, start_y, start_value)| {
                                (*id == pointer_id).then(|| (input.clone(), *start_y, *start_value))
                            },
                        );
                        if let Some((input, start_y, start_value)) = drag {
                            event.prevent_default();
                            update_knob_drag(
                                &drag_app,
                                &input,
                                start_y,
                                f64::from(event.client_y()),
                                start_value,
                            );
                        }
                    }
                    "pointerup" => {
                        let drag = active.borrow().as_ref().and_then(
                            |(id, input, surface, start_y, start_value)| {
                                (*id == pointer_id).then(|| {
                                    (input.clone(), surface.clone(), *start_y, *start_value)
                                })
                            },
                        );
                        if let Some((input, surface, start_y, start_value)) = drag {
                            event.prevent_default();
                            update_knob_drag(
                                &drag_app,
                                &input,
                                start_y,
                                f64::from(event.client_y()),
                                start_value,
                            );
                            let _ = surface.release_pointer_capture(pointer_id);
                            *active.borrow_mut() = None;
                            finish_drag(&drag_app);
                        }
                    }
                    "pointercancel" | "lostpointercapture"
                        if active
                            .borrow()
                            .as_ref()
                            .is_some_and(|(id, ..)| *id == pointer_id) =>
                    {
                        *active.borrow_mut() = None;
                        finish_drag(&drag_app);
                    }
                    _ => {}
                }
            });
            app.borrow()
                .root
                .add_event_listener_with_callback(event_name, drag.as_ref().unchecked_ref())?;
            drag.forget();
        }

        let secondary = Rc::new(RefCell::new(None::<Element>));
        for event_name in [
            "pointerdown",
            "pointerup",
            "pointercancel",
            "lostpointercapture",
        ] {
            let pressed = secondary.clone();
            let guard =
                Closure::<dyn FnMut(PointerEvent)>::new(
                    move |event: PointerEvent| match event_name {
                        "pointerdown" if event.pointer_type() == "mouse" && event.button() != 0 => {
                            if let Some(element) = element_from_event(&event) {
                                if let Some(previous) =
                                    pressed.borrow_mut().replace(element.clone())
                                {
                                    let _ =
                                        previous.class_list().remove_1("rackforge-context-press");
                                }
                                let _ = element.class_list().add_1("rackforge-context-press");
                                event.prevent_default();
                                event.stop_immediate_propagation();
                            }
                        }
                        "pointerup" | "pointercancel" | "lostpointercapture" => {
                            if let Some(element) = pressed.borrow_mut().take() {
                                let _ = element.class_list().remove_1("rackforge-context-press");
                                event.prevent_default();
                                event.stop_immediate_propagation();
                            }
                        }
                        _ => {}
                    },
                );
            app.borrow()
                .root
                .add_event_listener_with_callback_and_bool(
                    event_name,
                    guard.as_ref().unchecked_ref(),
                    true,
                )?;
            guard.forget();
        }

        let resize_app = app.clone();
        let resize = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            layout_group_outlines(&resize_app.borrow().root);
        });
        app.borrow()
            .window
            .add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref())?;
        resize.forget();

        let message_app = app.clone();
        let message = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            let source_is_parent = message_app
                .borrow()
                .window
                .parent()
                .ok()
                .flatten()
                .zip(event.source())
                .is_some_and(|(parent, source)| Object::is(parent.as_ref(), source.as_ref()));
            if !source_is_parent || event.origin() != message_app.borrow().host_origin {
                return;
            }
            let data = event.data();
            if Reflect::get(&data, &JsValue::from_str("protocol"))
                .ok()
                .and_then(|value| value.as_string())
                .as_deref()
                != Some(PROTOCOL)
            {
                return;
            }
            match Reflect::get(&data, &JsValue::from_str("kind"))
                .ok()
                .and_then(|value| value.as_string())
                .as_deref()
            {
                Some("context") => {
                    if let Ok(context) = serde_wasm_bindgen::from_value::<HostContext>(data) {
                        let changed = message_app
                            .borrow()
                            .context
                            .as_ref()
                            .map(|old| old.instance.selected_sound_id.as_str())
                            != Some(context.instance.selected_sound_id.as_str());
                        message_app.borrow_mut().context = Some(context);
                        message_app.borrow().render();
                        if changed || message_app.borrow().snapshot.is_none() {
                            refresh_parameters(&message_app);
                        }
                    }
                }
                Some("parameter_changed") => {
                    let index = Reflect::get(&data, &JsValue::from_str("parameter_index"))
                        .ok()
                        .and_then(|value| value.as_f64())
                        .filter(|value| value.is_finite() && value.fract() == 0.0)
                        .map(|value| value as u32);
                    let value = Reflect::get(&data, &JsValue::from_str("value"))
                        .ok()
                        .and_then(|value| value.as_f64())
                        .filter(|value| value.is_finite());
                    if let (Some(index), Some(value)) = (index, value) {
                        message_app
                            .borrow_mut()
                            .parameter_values
                            .insert(index, value);
                        update_parameter_dom(&message_app, index);
                    }
                }
                Some("response") => {
                    if let Some(request_id) = Reflect::get(&data, &JsValue::from_str("request_id"))
                        .ok()
                        .and_then(|value| value.as_string())
                    {
                        let ok = Reflect::get(&data, &JsValue::from_str("ok"))
                            .ok()
                            .and_then(|value| value.as_bool())
                            .unwrap_or(false);
                        let result = if ok {
                            Ok(Reflect::get(&data, &JsValue::from_str("result"))
                                .unwrap_or(JsValue::UNDEFINED))
                        } else {
                            Err(Reflect::get(&data, &JsValue::from_str("error"))
                                .ok()
                                .and_then(|value| value.as_string())
                                .unwrap_or_else(|| "RackForge rejected this request.".to_owned()))
                        };
                        resolve(&message_app, &request_id, result);
                    }
                }
                _ => {}
            }
        });
        app.borrow()
            .window
            .add_event_listener_with_callback("message", message.as_ref().unchecked_ref())?;
        message.forget();
        Ok(())
    }

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        let app = App::new()?;
        // Every shadow and lighting gradient in the stylesheet reads the one
        // panel light from these root variables.
        if let Some(root) = app.borrow().document.document_element() {
            let existing = root.get_attribute("style").unwrap_or_default();
            root.set_attribute("style", &format!("{existing}{}", light::css_variables()))?;
        }
        install_events(&app)?;
        follow_program_save(&app)?;
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        let ready = Ready {
            protocol: PROTOCOL,
            kind: "ready",
        }
        .serialize(&serializer)?;
        let (parent, origin) = {
            let app = app.borrow();
            (
                app.window
                    .parent()?
                    .ok_or_else(|| JsValue::from_str("missing parent"))?,
                app.host_origin.clone(),
            )
        };
        parent.post_message(&ready, &origin)?;
        load_assets(&app);
        // Should anything never arrive, the page shows regardless.
        let fallback = app.clone();
        let show_anyway = Closure::once_into_js(move || reveal(&fallback));
        app.borrow()
            .window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                show_anyway.unchecked_ref(),
                5_000,
            )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_escaped_before_rendering() {
        assert_eq!(
            escape_html("<RF & \"5\">"),
            "&lt;RF &amp; &quot;5&quot;&gt;"
        );
    }

    #[test]
    fn legend_rule_runs_between_the_switches_round_its_name() {
        assert_eq!(
            legend_rule_path(29.0, 157.0, 55.0, 131.0, 6.0).as_deref(),
            Some("M 29.00 6.00 H 55.00 M 131.00 6.00 H 157.00")
        );
        // A name wider than the run leaves no rule to draw.
        assert!(legend_rule_path(29.0, 90.0, 20.0, 100.0, 6.0).is_none());
    }

    #[test]
    fn section_outline_is_one_continuous_path_with_a_label_gap() {
        let path = section_outline_path(320.0, 150.0, 22.0, 91.0, 15.0).unwrap();
        assert_eq!(path.matches("M ").count(), 1);
        assert_eq!(path.matches(" A ").count(), 4);
        assert!(path.starts_with("M 22.00 1.00"));
        assert!(path.ends_with("H 91.00"));
        assert!(!path.contains('Z'));
    }

    #[test]
    fn knob_drag_is_relative_quantized_and_clamped() {
        assert_eq!(
            relative_knob_value(0.5, 0.0, 0.0, 1.0, 1.0 / 127.0),
            64.0 / 127.0
        );
        assert_eq!(relative_knob_value(0.5, 500.0, 0.0, 1.0, 1.0 / 127.0), 1.0);
        assert_eq!(relative_knob_value(0.5, -500.0, 0.0, 1.0, 1.0 / 127.0), 0.0);
    }

    #[test]
    fn knob_keyboard_controls_are_explicit_and_bounded() {
        let step = 1.0 / 127.0;
        assert_eq!(
            keyboard_knob_value(0.0, "ArrowRight", 0.0, 1.0, step),
            Some(step)
        );
        assert_eq!(
            keyboard_knob_value(1.0, "ArrowUp", 0.0, 1.0, step),
            Some(1.0)
        );
        assert_eq!(keyboard_knob_value(0.5, "Home", 0.0, 1.0, step), Some(0.0));
        assert_eq!(keyboard_knob_value(0.5, "End", 0.0, 1.0, step), Some(1.0));
        assert_eq!(keyboard_knob_value(0.5, "Escape", 0.0, 1.0, step), None);
    }

    #[test]
    fn led_spill_follows_inverse_square_on_the_bevel_plane() {
        let gradient = led_spill_gradient("g", 26.0);
        // D = 10 sin 50 + 1.5 cos 50 = 8.62; the foot of the perpendicular
        // sits above the bevel, and the profile falls to (1 + 9)^-1.5 at 3D.
        assert!(gradient.contains("translate(26.00 16.61)"));
        assert!(gradient.contains("scale(25.87 16.63)"));
        assert!(gradient.contains("stop-opacity=\"0.6000\""));
        assert!(gradient.ends_with("stop-opacity=\"0.0190\"></stop></radialGradient>"));
    }

    #[test]
    fn host_defaults_accept_numbers_and_booleans() {
        let number: ParameterDefault = serde_json::from_str("0.625").unwrap();
        let enabled: ParameterDefault = serde_json::from_str("true").unwrap();
        assert_eq!(number.as_f64(), 0.625);
        assert_eq!(enabled.as_f64(), 1.0);
    }

    #[test]
    fn prophet_switch_svg_supports_one_or_two_independent_leds() {
        let single = prophet_switch_svg(4, true, None, false);
        assert_eq!(single.matches("<g class=\"led").count(), 1);
        assert!(single.contains("class=\"led on\""));
        assert!(single.contains("url(#switch-rocker-4)"));
        assert!(single.contains("class=\"switch-block\""));
        // Sunk into a thin cut-out, lit facet by facet.
        assert!(single.contains("class=\"switch-well\""));
        assert_eq!(single.matches("class=\"switch-well-rim\"").count(), 2);
        assert!(single.contains("d=\"M1.2 -5.8H50.8\""));
        assert!(single.contains("d=\"M1.2 -5.8V61.8\""));
        assert!(!single.contains("switch-frame"));
        // Four bevels, the flat face and the LED deck, each lit evenly.
        assert_eq!(single.matches("class=\"switch-sheen\"").count(), 6);
        assert!(!single.contains("glint-4"));

        let dual = prophet_switch_svg(9, false, Some(true), false);
        assert_eq!(dual.matches("<g class=\"led").count(), 2);
        assert_eq!(dual.matches("class=\"led on\"").count(), 1);
        assert!(dual.contains("translate(19 10)"));
        assert!(dual.contains("url(#led-spill-a-9)") && dual.contains("url(#led-spill-b-9)"));
        assert!(single.contains("url(#led-spill-a-4)") && !single.contains("led-spill-b"));
        assert!(dual.contains("translate(33 10)"));

        assert!(single.contains("class=\"switch-dirt\" href=\"assets/finishes/cap-1-1.png\""));
        assert!(dual.contains("href=\"assets/finishes/cap-2-0.png\""));
        assert!(single.contains(
            "class=\"panel-dust\" href=\"assets/finishes/well-1.png\" x=\"-14\" y=\"-20\""
        ));
        assert!(single.find("well-1.png") < single.find("class=\"switch-block\""));
        assert!(knob_dust_layer(7).contains("href=\"assets/finishes/knob-1.png\""));
        assert_eq!(finish_paths().len(), 13);

        let step = step_key_svg(12, Step::Next);
        assert!(!step.contains("data-led=") && step.contains(">NEXT<br>PROGRAM</span>"));
        assert!(step_key_svg(13, Step::Previous).contains(">BACK<br>PROGRAM</span>"));

        let light = prophet_switch_svg(11, false, None, true);
        assert!(light.contains("stop-color=\"#77837f\""));
    }

    #[test]
    fn identity_plaque_is_engraved_metal_lit_by_the_panel_light() {
        let plaque = identity_plaque_svg();
        assert!(plaque.contains("class=\"identity-plaque\""));
        assert!(plaque.contains(">RACKFORGE INSTRUMENTS</text>"));
        assert!(plaque.contains(">RF-5</text>"));
        assert!(plaque.contains(">FIVE-VOICE POLYPHONIC SYNTHESIZER</text>"));
        assert!(!plaque.contains("plaque-fastener"));
        // Upper-left light: the main lobe sits up and left of centre, the
        // engraving edge falls down and right.
        assert!(plaque.contains("translate(186.00 34.00)"));
        assert!(plaque.contains("plaque-engraving-edge\" transform=\"translate(0.57 0.57)\""));
    }

    /// Programs are chosen with RackForge's selector, which the panel makes
    /// once and puts back in the program memory after every render.
    #[test]
    fn the_program_memory_carries_the_rackforge_program_selector() {
        let source = include_str!("lib.rs");
        assert!(source.contains("create_element(\"rf-program-select\")"));
        assert!(source.contains("id=\\\"program-selector-slot\\\""));
        // The maker is engraved on the nameplate, not repeated in the bar.
        assert!(!source.contains(&["program", "-identity"].concat()));
        assert!(!source.contains(&["program", "-grid"].concat()));
        // The nameplate heads the panel, before the program memory.
        let plaque = source.find("wood-rail-plaque").expect("the nameplate rail");
        let memory = source
            .find("render_program_memory());")
            .expect("the program memory");
        assert!(plaque < memory, "the nameplate comes first");
    }

    #[test]
    fn every_page_key_has_a_short_name_for_a_narrow_row() {
        for section in panel::SECTIONS {
            assert!(
                !section.short.is_empty() && section.short.len() <= 6,
                "{}",
                section.id
            );
        }
    }

    #[test]
    fn narrow_displays_show_the_rev_3s_digits() {
        let original = Some("factory.rf5.original");
        assert_eq!(program_digits("1-1 Brass", original, None), "11");
        assert_eq!(program_digits("5-8 Cat", original, None), "58");
        assert_eq!(
            program_digits("2-4 Dog", Some("factory.rf5.file2"), None),
            "2.4"
        );
        assert_eq!(
            program_digits("3-7 Slide Guitar", Some("factory.rf5.file3"), None),
            "37."
        );
        assert_eq!(program_digits("My Horns", Some("user"), Some(7)), "U7");
        assert_eq!(program_digits("Init", None, None), "--");
    }

    #[test]
    fn protocol_identity_is_stable() {
        assert_eq!(PROTOCOL, "rackforge.plugin.web@1");
    }
}
