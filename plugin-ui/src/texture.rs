//! The panel's generated finishes: its crinkle-paint specks and the dust
//! round its controls.
//!
//! Made offline, not when the panel starts: the test at the foot of this
//! file writes them to `web/assets/finishes` (with `RF5_WRITE_FINISHES=1`)
//! and otherwise checks the shipped images still match, so the page only
//! loads images and repeats the finish with CSS.
//!
//! The finish is the Prophet-5's crinkle paint seen at arm's length: flat
//! plains, and scattered specks rougher than them. A speck's top scatters
//! the light, so the panel's anisotropic sheen does not reach it: it is
//! drawn black with an alpha, taking the sheen away where the sheen is and
//! hardly showing where the panel is dark. The specks are soft Gaussian
//! bumps, one per jittered point, their positions bent by smooth noise so
//! they cluster irregularly, thinned in patches by slower noise, and cut by
//! a soft threshold so the plains between them stay untouched. Each speck
//! also keeps a faint relief of its own, lit by the panel light, so it reads
//! as a raised grain and not a stain.
//!
//! Everything is periodic on a `TILE` x `TILE` torus, so the tile repeats
//! without a seam.

use crate::{LED_LENS_RADIUS, LED_RIM_RADIUS, WELL_DUST_ORIGIN, WELL_DUST_SPAN, light};

/// Tile edge in pixels. It is drawn at half this size in CSS pixels, which
/// keeps it crisp on double-density screens and its repeat hard to spot.
pub const TILE: usize = 512;

/// Specks across the tile, how far each spreads (exp(-d^2 / spread), d in
/// cells) and their height range.
const SPECK_CELLS: usize = 90;
const SPECK_SPREAD: f64 = 0.18;
const SPECK_HEIGHT: (f64, f64) = (0.5, 1.0);
/// How far, in pixels, smooth noise bends the speck positions.
const WARP_PIXELS: f64 = 4.0;
/// Where a speck begins and where it is whole, once thinned in patches.
const SPECK_EDGE: (f64, f64) = (0.55, 0.85);
/// How much of the sheen a whole speck takes away.
const SPECK_MATTE: f64 = 0.44;
/// A speck's own relief, lit by the panel light, with a glint on the side
/// that faces it: slope per unit of speck per pixel, and opacity per unit of
/// brightness change.
const RELIEF: f64 = 1.6;
const RELIEF_CONTRAST: f64 = 0.288;
const SPECULAR: f64 = 0.5;
const ROUGHNESS: f64 = 0.35;

fn hash(ix: usize, iy: usize, seed: u32) -> f64 {
    // A small integer hash: stable, so the tile is the same on every start.
    let mut h = (ix as u32).wrapping_mul(0x27d4_eb2d)
        ^ (iy as u32).wrapping_mul(0x1656_67b1)
        ^ seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / f64::from(u32::MAX)
}

fn wrap(index: i64, count: usize) -> usize {
    index.rem_euclid(count as i64) as usize
}

fn value_noise(x: f64, y: f64, cells: usize, seed: u32) -> f64 {
    value_noise_on(x, y, cells, seed, TILE)
}

fn value_noise_on(x: f64, y: f64, cells: usize, seed: u32, tile: usize) -> f64 {
    let size = tile as f64 / cells as f64;
    let (fx, fy) = (x / size, y / size);
    let (ix, iy) = (fx.floor(), fy.floor());
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let (tx, ty) = (smooth(fx - ix), smooth(fy - iy));
    let (x0, y0) = (wrap(ix as i64, cells), wrap(iy as i64, cells));
    let (x1, y1) = ((x0 + 1) % cells, (y0 + 1) % cells);
    let top = hash(x0, y0, seed) * (1.0 - tx) + hash(x1, y0, seed) * tx;
    let bottom = hash(x0, y1, seed) * (1.0 - tx) + hash(x1, y1, seed) * tx;
    top * (1.0 - ty) + bottom * ty
}

fn warp_offset(x: f64, y: f64, seed: u32) -> f64 {
    let noise = 0.6 * value_noise(x, y, 8, seed) + 0.4 * value_noise(x, y, 16, seed + 1);
    WARP_PIXELS * (noise - 0.5) * 2.0
}

fn bumps(x: f64, y: f64) -> f64 {
    let tile = TILE as f64;
    let size = tile / SPECK_CELLS as f64;
    let px = (x + warp_offset(x, y, 101)).rem_euclid(tile);
    let py = (y + warp_offset(x, y, 211)).rem_euclid(tile);
    let (cx, cy) = ((px / size).floor() as i64, (py / size).floor() as i64);
    let mut sum = 0.0;
    for oy in -1..=1 {
        for ox in -1..=1 {
            let (gx, gy) = (cx + ox, cy + oy);
            let (wx, wy) = (wrap(gx, SPECK_CELLS), wrap(gy, SPECK_CELLS));
            let fx = (gx as f64 + hash(wx, wy, 7)) * size;
            let fy = (gy as f64 + hash(wx, wy, 13)) * size;
            let height = SPECK_HEIGHT.0 + (SPECK_HEIGHT.1 - SPECK_HEIGHT.0) * hash(wx, wy, 29);
            let distance2 = ((px - fx).powi(2) + (py - fy).powi(2)) / (size * size);
            sum += height * (-distance2 / SPECK_SPREAD).exp();
        }
    }
    sum
}

/// How much of a speck covers a tile pixel, 0 on the plains to 1 on a
/// whole speck; periodic with period `TILE`.
pub fn speck(x: usize, y: usize) -> f64 {
    let (x, y) = ((x % TILE) as f64, (y % TILE) as f64);
    let patches = 0.6 + 0.8 * (0.6 * value_noise(x, y, 16, 41) + 0.4 * value_noise(x, y, 32, 42));
    smoothstep(SPECK_EDGE.0, SPECK_EDGE.1, bumps(x, y) * patches)
}

/// RGBA bytes of the tile, row by row: the speck's matte darkening with its
/// faint relief on top, as white (lighter) or black (darker) with an alpha
/// against the flat panel.
pub fn panel_texture_rgba() -> Vec<u8> {
    let specks: Vec<f64> = (0..TILE * TILE)
        .map(|index| speck(index % TILE, index / TILE))
        .collect();
    let at = |x: usize, y: usize| specks[(y % TILE) * TILE + (x % TILE)];
    let light = light::light_vector();
    let across = [1.0, 0.0, 0.0];
    let flat =
        light[2] + SPECULAR * light::ward_specular([0.0, 0.0, 1.0], across, ROUGHNESS, ROUGHNESS);
    let mut rgba = Vec::with_capacity(TILE * TILE * 4);
    for y in 0..TILE {
        for x in 0..TILE {
            let dx = (at(x + 1, y) - at(x + TILE - 1, y)) * 0.5;
            let dy = (at(x, y + 1) - at(x, y + TILE - 1)) * 0.5;
            let (nx, ny) = (-RELIEF * dx, -RELIEF * dy);
            let length = (nx * nx + ny * ny + 1.0).sqrt();
            let normal = [nx / length, ny / length, 1.0 / length];
            let lit = (normal[0] * light[0] + normal[1] * light[1] + normal[2] * light[2]).max(0.0)
                + SPECULAR * light::ward_specular(normal, across, ROUGHNESS, ROUGHNESS);
            let change = (lit - flat) * RELIEF_CONTRAST - at(x, y) * SPECK_MATTE;
            let shade = if change > 0.0 { 255 } else { 0 };
            let alpha = change.abs().min(1.0);
            rgba.extend_from_slice(&[shade, shade, shade, (alpha * 255.0).round() as u8]);
        }
    }
    rgba
}

/// The dust image covers the switch drawing's whole viewBox (x 0..52,
/// y -6..64) at this many pixels per unit, about 2.3 per CSS pixel.
pub const DUST_SCALE: usize = 2;
pub const DUST_WIDTH: usize = 52 * DUST_SCALE;
pub const DUST_HEIGHT: usize = 70 * DUST_SCALE;
/// Fine dust on black plastic: a light warm grey, never more than a veil.
const DUST: [u8; 3] = [176, 168, 150];
const DUST_OPACITY: f64 = 0.2;
/// On the cap itself, black plastic shows dust more than the grained
/// panel does, so it carries less of it.
const CAP_DUST: f64 = 0.55;

fn smoothstep(low: f64, high: f64, value: f64) -> f64 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How much dust sits at a point of the switch cap (switch units), built the
/// way mask generators such as Substance's Dirt and Edge Dirt build theirs:
///
/// - a base per part, by how often it is touched: the flat LED deck, which
///   nobody presses, most; the pyramid's bevels less; its top least;
/// - occlusion: dust packs into concave creases, falling off as exp(-d/r)
///   with the distance d to the fold between deck and pyramid, to the ring
///   round each LED bezel and to the cap's edge against the gutter, joined
///   as 1 - product(1 - e_i);
/// - edge masking: the convex rim of the pyramid's top is rubbed clean;
/// - wear: the finger clears the middle of the top, not its corners;
/// - grunge: noise added before a soft threshold,
///   smoothstep(a, b, base + occlusion - edges + (noise - 0.5) k), so the dust
///   breaks into patches that follow the form instead of lying on it.
fn switch_dust(x: f64, y: f64, leds: &[f64], seed: u32) -> f64 {
    let (left, right, top, bottom) = (3.0, 49.0, -4.0, 60.0);
    if x < left || x > right || y < top || y > bottom {
        return 0.0;
    }
    let led_distance = leds
        .iter()
        .map(|&led_x| ((x - led_x).powi(2) + (y - 10.0).powi(2)).sqrt())
        .fold(f64::INFINITY, f64::min);
    if led_distance < LED_LENS_RADIUS + 0.2 {
        return 0.0; // the lens stays clear
    }
    let (face_left, face_right, face_top, face_bottom) = (7.0, 45.0, 24.0, 58.0);
    let on_face = (face_left..=face_right).contains(&x) && (face_top..=face_bottom).contains(&y);
    let base = if y < 20.0 {
        0.42
    } else if on_face {
        0.14
    } else {
        0.3
    };
    let crease = |distance: f64, reach: f64| (-distance.max(0.0) / reach).exp();
    let fold = if y < 24.0 {
        crease((y - 20.0).abs(), 2.5)
    } else {
        0.0
    };
    let bezel = crease((led_distance - LED_RIM_RADIUS).abs(), 1.5);
    let gutter = crease((x - left).min(right - x).min(y - top).min(bottom - y), 1.8);
    let occlusion = 1.0 - (1.0 - fold) * (1.0 - bezel) * (1.0 - gutter);
    let rim = if y >= 20.0 {
        let dx = (x - face_left).abs().min((face_right - x).abs());
        let dy = (y - face_top).abs().min((face_bottom - y).abs());
        let inside_x = (face_left..=face_right).contains(&x);
        let inside_y = (face_top..=face_bottom).contains(&y);
        let distance = match (inside_x, inside_y) {
            (true, true) => dx.min(dy),
            (true, false) => dy,
            (false, true) => dx,
            (false, false) => (dx * dx + dy * dy).sqrt(),
        };
        crease(distance, 1.0)
    } else {
        0.0
    };
    let grunge = grunge(
        x * DUST_SCALE as f64,
        (y + 6.0) * DUST_SCALE as f64,
        500 + seed * 10,
        DUST_WIDTH,
    );
    let mut dust = smoothstep(
        0.3,
        0.9,
        base + 0.55 * occlusion - 0.3 * rim + (grunge - 0.5) * 0.5,
    );
    if on_face {
        let reach = ((x - 26.0) / 19.0).powi(2) + ((y - 41.0) / 17.0).powi(2);
        let wear = 1.0 - smoothstep(0.3, 1.0, reach.sqrt());
        dust *= 1.0 - 0.95 * wear;
    }
    dust
}

/// Grunge: three octaves of periodic noise over a `tile`-pixel image.
fn grunge(px: f64, py: f64, seed: u32, tile: usize) -> f64 {
    [(6, 0.45), (12, 0.35), (26, 0.2)]
        .iter()
        .enumerate()
        .map(|(octave, &(cells, weight))| {
            weight * value_noise_on(px, py, cells, seed + octave as u32, tile)
        })
        .sum()
}

/// How far the rag leaves dust out onto the panel from an edge it cannot
/// get into (e-folding distance, in the control's units, about a pixel):
/// a short way past a switch's cut-out, further round a knob's skirt,
/// half of which lies in the knob's own shadow.
const WELL_DUST_REACH: f64 = 4.0;
const KNOB_DUST_REACH: f64 = 8.0;

/// Dust on the panel at `outside` units out from the foot of a control,
/// where a cloth wiping the panel cannot reach into the corner: packed at
/// the edge and gone within a few units, as exp(-d/r) with the same grunge
/// and soft threshold as on the switch caps.
fn edge_dust(outside: f64, reach: f64, grunge: f64) -> f64 {
    smoothstep(
        0.3,
        0.9,
        0.8 * (-outside.max(0.0) / reach).exp() + (grunge - 0.5) * 0.5,
    )
}

/// The dust round a switch spans `WELL_DUST_SPAN` from `WELL_DUST_ORIGIN`.
pub const WELL_DUST_WIDTH: usize = WELL_DUST_SPAN.0 as usize * DUST_SCALE;
pub const WELL_DUST_HEIGHT: usize = WELL_DUST_SPAN.1 as usize * DUST_SCALE;

/// Dust round a switch's cut-out (x 1.2..50.8, y -5.8..61.8, corners 1.2):
/// none in the gutter, which stays black, and from the cut edge a hair out
/// onto the panel.
fn well_dust(x: f64, y: f64, seed: u32) -> f64 {
    let (centre, half, corner) = ((26.0, 28.0), (24.8, 33.8), 1.2);
    let qx = (x - centre.0).abs() - half.0 + corner;
    let qy = (y - centre.1).abs() - half.1 + corner;
    let outside = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - corner;
    let grunge = grunge(
        (x - WELL_DUST_ORIGIN.0) * DUST_SCALE as f64,
        (y - WELL_DUST_ORIGIN.1) * DUST_SCALE as f64,
        700 + seed * 10,
        WELL_DUST_WIDTH,
    );
    if outside <= 0.0 {
        0.0
    } else {
        edge_dust(outside, WELL_DUST_REACH, grunge)
    }
}

/// The panel round a knob: its image spans the knob scale's 100 x 100 units.
pub const KNOB_DUST_SCALE: usize = 2;
pub const KNOB_DUST_SIZE: usize = 100 * KNOB_DUST_SCALE;
/// The knob's skirt in scale units: 26 px of radius on the 86 px scale.
const KNOB_RADIUS: f64 = 26.0 * 100.0 / 86.0;

/// Dust round the foot of a knob, where the cloth goes round it and not
/// into the corner under its skirt.
fn knob_dust(x: f64, y: f64, seed: u32) -> f64 {
    let outside = (x - 50.0).hypot(y - 50.0) - KNOB_RADIUS;
    if outside < 0.0 {
        return 0.0;
    }
    let grunge = grunge(
        x * KNOB_DUST_SCALE as f64,
        y * KNOB_DUST_SCALE as f64,
        900 + seed * 10,
        KNOB_DUST_SIZE,
    );
    edge_dust(outside, KNOB_DUST_REACH, grunge)
}

/// A dust image `width` x `height` pixels at `scale` pixels per unit, whose
/// top-left pixel sits at `origin` in the control's units.
fn dust_rgba(
    (width, height): (usize, usize),
    scale: usize,
    origin: (f64, f64),
    dust: impl Fn(f64, f64) -> f64,
) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(width * height * 4);
    for py in 0..height {
        for px in 0..width {
            let x = origin.0 + (px as f64 + 0.5) / scale as f64;
            let y = origin.1 + (py as f64 + 0.5) / scale as f64;
            let alpha = DUST_OPACITY * dust(x, y);
            rgba.extend_from_slice(&[DUST[0], DUST[1], DUST[2], (alpha * 255.0).round() as u8]);
        }
    }
    rgba
}

/// RGBA dust image for a switch whose LEDs sit at `leds` (x, switch units).
pub fn switch_dust_rgba(leds: &[f64], seed: u32) -> Vec<u8> {
    dust_rgba(
        (DUST_WIDTH, DUST_HEIGHT),
        DUST_SCALE,
        (0.0, -6.0),
        |x, y| CAP_DUST * switch_dust(x, y, leds, seed),
    )
}

/// RGBA dust image for the gutter and panel round a switch.
pub fn well_dust_rgba(seed: u32) -> Vec<u8> {
    dust_rgba(
        (WELL_DUST_WIDTH, WELL_DUST_HEIGHT),
        DUST_SCALE,
        WELL_DUST_ORIGIN,
        |x, y| well_dust(x, y, seed),
    )
}

/// RGBA dust image for the panel round a knob.
pub fn knob_dust_rgba(seed: u32) -> Vec<u8> {
    dust_rgba(
        (KNOB_DUST_SIZE, KNOB_DUST_SIZE),
        KNOB_DUST_SCALE,
        (0.0, 0.0),
        |x, y| knob_dust(x, y, seed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rag_leaves_dust_only_at_the_foot_of_each_control() {
        let mean = |f: &dyn Fn(f64) -> f64| (0..40).map(|i| f(i as f64)).sum::<f64>() / 40.0;
        // The gutter stays black; dust from the cut edge out, clean a few
        // units on.
        let gutter = mean(&|i| well_dust(2.1, i * 1.5, 1));
        let hair = mean(&|i| well_dust(0.6, i * 1.5, 1));
        let away = mean(&|i| well_dust(-12.0, i * 1.5, 1));
        assert_eq!(gutter, 0.0);
        assert!(hair > 0.1 && hair > away, "{hair} {away}");
        assert!(away < 0.02, "away {away}");
        // Round a knob: none under the cap, some at its foot, none by the
        // numbers.
        let ring = |radius: f64| {
            mean(&|i| {
                let a = i * 0.157;
                knob_dust(50.0 + radius * a.cos(), 50.0 + radius * a.sin(), 2)
            })
        };
        assert_eq!(ring(KNOB_RADIUS - 1.0), 0.0);
        assert!(ring(KNOB_RADIUS + 0.3) > 0.1 && ring(KNOB_RADIUS + 22.0) < 0.02);
        assert_eq!(knob_dust_rgba(0).len(), KNOB_DUST_SIZE * KNOB_DUST_SIZE * 4);
        assert_eq!(
            well_dust_rgba(0).len(),
            WELL_DUST_WIDTH * WELL_DUST_HEIGHT * 4
        );
    }

    #[test]
    fn dust_follows_the_form_of_the_switch() {
        let at = |x: f64, y: f64| switch_dust(x, y, &[26.0], 0);
        // Lens clear, pressed top nearly clean, crease dustier than open deck.
        assert_eq!(at(26.0, 10.0), 0.0);
        let pressed = (0..20)
            .map(|i| at(22.0 + i as f64 * 0.4, 41.0))
            .sum::<f64>()
            / 20.0;
        let fold = (0..20)
            .map(|i| at(10.0 + i as f64 * 1.5, 19.5))
            .sum::<f64>()
            / 20.0;
        let deck = (0..20).map(|i| at(10.0 + i as f64 * 1.5, 2.0)).sum::<f64>() / 20.0;
        assert!(pressed < 0.05, "pressed {pressed}");
        assert!(fold > deck && deck > pressed, "fold {fold} deck {deck}");
        // Only a veil: never more than the dust opacity.
        let rgba = switch_dust_rgba(&[26.0], 0);
        assert_eq!(rgba.len(), DUST_WIDTH * DUST_HEIGHT * 4);
        assert!(
            rgba.as_chunks::<4>()
                .0
                .iter()
                .all(|p| f64::from(p[3]) <= DUST_OPACITY * 255.0 + 0.5)
        );
    }

    #[test]
    fn the_finish_repeats_exactly_at_the_tile_edge() {
        for step in (0..TILE).step_by(7) {
            assert_eq!(speck(0, step), speck(TILE, step));
            assert_eq!(speck(step, 0), speck(step, TILE));
        }
    }

    #[test]
    fn the_finish_is_flat_plains_with_scattered_specks() {
        let samples: Vec<f64> = (0..TILE)
            .step_by(3)
            .flat_map(|y| (0..TILE).step_by(3).map(move |x| speck(x, y)))
            .collect();
        let share = |f: &dyn Fn(f64) -> bool| {
            samples.iter().filter(|&&s| f(s)).count() as f64 / samples.len() as f64
        };
        let plains = share(&|s| s == 0.0);
        let whole = share(&|s| s == 1.0);
        assert!(plains > 0.5, "plains {plains}");
        assert!(whole > 0.005 && whole < 0.2, "whole specks {whole}");
    }

    #[test]
    fn specks_mostly_take_light_away_with_a_faint_lit_edge() {
        let rgba = panel_texture_rgba();
        assert_eq!(rgba.len(), TILE * TILE * 4);
        let lighter: Vec<u8> = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[0] == 255)
            .map(|pixel| pixel[3])
            .collect();
        let darker = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[0] == 0 && pixel[3] > 0)
            .count();
        // Highlights on the specks' lit sides, never a glare.
        assert!(!lighter.is_empty() && darker > lighter.len());
        assert!(
            lighter.iter().all(|&alpha| alpha < 110),
            "{:?}",
            lighter.iter().max()
        );
        // The plains are left as the panel.
        let untouched = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[3] == 0)
            .count();
        assert!(untouched > TILE * TILE / 3, "untouched {untouched}");
    }
}

#[cfg(test)]
mod shipped {
    use super::*;
    use crate::{DUST_VARIANTS, PANEL_TEXTURE_PATH, cap_dust_path, knob_dust_path, well_dust_path};

    /// Each shipped finish image with its size and the generator it comes from.
    fn finishes() -> Vec<(String, (usize, usize), Vec<u8>)> {
        let mut finishes = vec![(
            PANEL_TEXTURE_PATH.to_owned(),
            (TILE, TILE),
            panel_texture_rgba(),
        )];
        for variant in 0..DUST_VARIANTS {
            let seed = variant as u32;
            finishes.extend([
                (
                    cap_dust_path(1, variant),
                    (DUST_WIDTH, DUST_HEIGHT),
                    switch_dust_rgba(&[26.0], seed),
                ),
                (
                    cap_dust_path(2, variant),
                    (DUST_WIDTH, DUST_HEIGHT),
                    switch_dust_rgba(&[19.0, 33.0], seed),
                ),
                (
                    well_dust_path(variant),
                    (WELL_DUST_WIDTH, WELL_DUST_HEIGHT),
                    well_dust_rgba(seed),
                ),
                (
                    knob_dust_path(variant),
                    (KNOB_DUST_SIZE, KNOB_DUST_SIZE),
                    knob_dust_rgba(seed),
                ),
            ]);
        }
        finishes
    }

    fn web_path(path: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../plugin/package/web")
            .join(path)
    }

    /// The shipped images are exactly what the generators make; run with
    /// `RF5_WRITE_FINISHES=1` to write them after changing a generator.
    #[test]
    fn shipped_finishes_match_their_generators() {
        let write = std::env::var_os("RF5_WRITE_FINISHES").is_some();
        for (path, (width, height), rgba) in finishes() {
            let file = web_path(&path);
            if write {
                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
                let mut encoder = png::Encoder::new(
                    std::io::BufWriter::new(std::fs::File::create(&file).unwrap()),
                    width as u32,
                    height as u32,
                );
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder.set_compression(png::Compression::Best);
                let mut writer = encoder.write_header().unwrap();
                writer.write_image_data(&rgba).unwrap();
                continue;
            }
            let decoder = png::Decoder::new(std::io::BufReader::new(
                std::fs::File::open(&file).unwrap_or_else(|error| panic!("{path}: {error}")),
            ));
            let mut reader = decoder.read_info().unwrap();
            let mut pixels = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut pixels).unwrap();
            assert_eq!(
                (info.width as usize, info.height as usize),
                (width, height),
                "{path}"
            );
            assert_eq!(info.color_type, png::ColorType::Rgba, "{path}");
            assert!(
                pixels[..info.buffer_size()] == rgba[..],
                "{path} differs from its generator"
            );
        }
    }
}
