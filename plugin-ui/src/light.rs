//! The one light the whole panel is drawn under.
//!
//! Every highlight, cast shadow and shading gradient takes its direction from
//! `LIGHT_SOURCE_BEARING_DEGREES`. The stylesheet reads it through the CSS
//! variables `css_variables` writes on the document root, and the SVG this
//! crate generates reads it directly. Moving the light is this one constant.

/// Where the light comes from as seen on the screen, clockwise from straight
/// up: 0 is overhead, 315 the upper left.
pub const LIGHT_SOURCE_BEARING_DEGREES: f64 = 315.0;

/// Unit vector from a surface towards the light, in screen coordinates
/// (x to the right, y down).
pub fn toward_light() -> (f64, f64) {
    let bearing = LIGHT_SOURCE_BEARING_DEGREES.to_radians();
    (bearing.sin(), -bearing.cos())
}

/// Unit vector along which shadows fall: away from the light.
pub fn shadow_direction() -> (f64, f64) {
    let (x, y) = toward_light();
    (-x, -y)
}

/// CSS gradient angle running from the lit side to the shaded side.
pub fn shading_angle_degrees() -> f64 {
    (LIGHT_SOURCE_BEARING_DEGREES + 180.0).rem_euclid(360.0)
}

/// Declarations for the document root, read by every shadow and lighting
/// gradient in the stylesheet.
pub fn css_variables() -> String {
    let (light_x, light_y) = toward_light();
    let (shadow_x, shadow_y) = shadow_direction();
    format!(
        "--light-x:{light_x:.4};--light-y:{light_y:.4};--shadow-x:{shadow_x:.4};--shadow-y:{shadow_y:.4};--light-angle:{:.2}deg;--knob-chamfer:{};--knob-silver-top:{}",
        shading_angle_degrees(),
        knob_chamfer_gradient(),
        knob_silver_top()
    )
}

/// `objectBoundingBox` endpoints of a gradient centred on its shape that runs
/// from the lit side to the shaded side, turned `offset_degrees` away from the
/// light's own axis and reaching `half_length` either side of the centre.
pub fn gradient_vector(offset_degrees: f64, half_length: f64) -> (f64, f64, f64, f64) {
    let angle = (shading_angle_degrees() + offset_degrees).to_radians();
    let (x, y) = (angle.sin() * half_length, -angle.cos() * half_length);
    (0.5 - x, 0.5 - y, 0.5 + x, 0.5 + y)
}

/// `x1`/`y1`/`x2`/`y2` attributes for `gradient_vector`.
pub fn gradient_attributes(offset_degrees: f64, half_length: f64) -> String {
    let (x1, y1, x2, y2) = gradient_vector(offset_degrees, half_length);
    format!("x1=\"{x1:.4}\" y1=\"{y1:.4}\" x2=\"{x2:.4}\" y2=\"{y2:.4}\"")
}

/// Height of the light above the panel, in degrees.
pub const LIGHT_ELEVATION_DEGREES: f64 = 40.0;

fn normalize([x, y, z]: [f64; 3]) -> [f64; 3] {
    let length = (x * x + y * y + z * z).sqrt();
    [x / length, y / length, z / length]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Unit vector towards the light in panel space: x right, y down, z out of
/// the panel towards the viewer.
pub fn light_vector() -> [f64; 3] {
    let (x, y) = toward_light();
    let elevation = LIGHT_ELEVATION_DEGREES.to_radians();
    [x * elevation.cos(), y * elevation.cos(), elevation.sin()]
}

/// Ward's (1992) anisotropic specular term for a facet with `normal`, rough
/// `alpha_x` along `tangent` and `alpha_y` across it, lit by the panel light
/// and seen from straight in front:
/// exp(-tan^2(theta_h) (cos^2(phi_h) / alpha_x^2 + sin^2(phi_h) / alpha_y^2)),
/// written with the half vector H in the facet's frame as
/// exp(-((H.t / H.n)^2 / alpha_x^2 + (H.b / H.n)^2 / alpha_y^2)).
/// A facet turned away from the light reflects nothing.
pub fn ward_specular(normal: [f64; 3], tangent: [f64; 3], alpha_x: f64, alpha_y: f64) -> f64 {
    let n = normalize(normal);
    let along = dot(tangent, n);
    let t = normalize([
        tangent[0] - n[0] * along,
        tangent[1] - n[1] * along,
        tangent[2] - n[2] * along,
    ]);
    let b = cross(n, t);
    let light = light_vector();
    let half = normalize([light[0], light[1], light[2] + 1.0]);
    let h_n = dot(half, n);
    if h_n <= 0.0 || dot(light, n) <= 0.0 {
        return 0.0;
    }
    let x = dot(half, t) / h_n;
    let y = dot(half, b) / h_n;
    (-(x * x / (alpha_x * alpha_x) + y * y / (alpha_y * alpha_y))).exp()
}

/// Gradient stops for Ward's lobe on a flat facet. With the light and the eye
/// at a distance D, tan(theta_h) grows as r / 2D across the facet, so the
/// term above becomes an elliptical Gaussian; `t = 1` is three sigma.
pub fn ward_lobe_stops(amplitude: f64, steps: u32) -> String {
    (0..=steps)
        .map(|step| {
            let t = f64::from(step) / f64::from(steps);
            format!(
                "<stop offset=\"{t}\" stop-color=\"#ffffff\" stop-opacity=\"{:.4}\"></stop>",
                amplitude * (-4.5 * t * t).exp()
            )
        })
        .collect()
}

/// The knobs' silver: one spun metal, lit by the panel light. Its brightness
/// on a surface with `normal` is ambient + diffuse n.L + Ward specular for a
/// spun finish, smooth along `around` (the circumference) and rough across it.
fn silver(normal: [f64; 3], around: [f64; 3]) -> String {
    const AMBIENT: f64 = 0.16;
    const DIFFUSE: f64 = 0.5;
    const SPECULAR: f64 = 0.55;
    const DARK: [f64; 3] = [70.0, 71.0, 75.0];
    const BRIGHT: [f64; 3] = [244.0, 244.0, 242.0];
    let brightness = (AMBIENT
        + DIFFUSE * dot(normal, light_vector()).max(0.0)
        + SPECULAR * ward_specular(normal, around, 0.25, 0.6))
    .clamp(0.0, 1.0);
    let [r, g, b] = core::array::from_fn(|i| DARK[i] + (BRIGHT[i] - DARK[i]) * brightness);
    format!("rgb({r:.0},{g:.0},{b:.0})")
}

/// The ring is a chamfer, /----\ in section: at each azimuth phi around the
/// knob its normal leans outwards by `CHAMFER_TILT_DEGREES`,
/// n(phi) = (sin(tilt) sin(phi), -sin(tilt) cos(phi), cos(tilt)) with phi
/// clockwise from straight up, as CSS conic angles run. Baked once into a
/// conic gradient.
const CHAMFER_TILT_DEGREES: f64 = 45.0;

pub fn knob_chamfer_gradient() -> String {
    let tilt = CHAMFER_TILT_DEGREES.to_radians();
    let stops: Vec<String> = (0..=36)
        .map(|step| {
            let degrees = f64::from(step) * 10.0;
            let phi = degrees.to_radians();
            let normal = [tilt.sin() * phi.sin(), -tilt.sin() * phi.cos(), tilt.cos()];
            let around = [phi.cos(), phi.sin(), 0.0];
            format!("{} {degrees:.0}deg", silver(normal, around))
        })
        .collect();
    format!("conic-gradient({})", stops.join(","))
}

/// The flat top of the all-silver knobs: the ring's metal on a face turned
/// straight up, so only the light tells the two apart.
pub fn knob_silver_top() -> String {
    silver([0.0, 0.0, 1.0], [1.0, 0.0, 0.0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1.0e-9
    }

    #[test]
    fn upper_left_light_casts_shadows_down_and_right() {
        let (x, y) = toward_light();
        assert!(x < 0.0 && y < 0.0 && close(x, y));
        let (x, y) = shadow_direction();
        assert!(x > 0.0 && y > 0.0 && close(x * x + y * y, 1.0));
        assert!(close(shading_angle_degrees(), 135.0));
    }

    #[test]
    fn shading_gradient_runs_corner_to_corner_under_an_upper_left_light() {
        let (x1, y1, x2, y2) = gradient_vector(0.0, core::f64::consts::FRAC_1_SQRT_2);
        assert!(close(x1, 0.0) && close(y1, 0.0) && close(x2, 1.0) && close(y2, 1.0));
    }

    #[test]
    fn ward_lights_the_facets_that_face_the_light() {
        let tilt = 35.0_f64.to_radians();
        let (s, c) = (tilt.sin(), tilt.cos());
        let across = [1.0, 0.0, 0.0];
        let down = [0.0, 1.0, 0.0];
        let face = ward_specular([0.0, 0.0, 1.0], across, 0.4, 0.4);
        let top = ward_specular([0.0, -s, c], across, 0.4, 0.4);
        let left = ward_specular([-s, 0.0, c], down, 0.4, 0.4);
        let right = ward_specular([s, 0.0, c], down, 0.4, 0.4);
        let foot = ward_specular([0.0, s, c], across, 0.4, 0.4);
        assert!(
            close(top, left),
            "an upper-left light treats both lit bevels alike"
        );
        assert!(top > face && face > right && face > foot);
        assert!(right < 0.01 && foot < 0.01);
    }

    #[test]
    fn ward_lobe_falls_to_three_sigma_at_its_edge() {
        let stops = ward_lobe_stops(0.5, 16);
        assert!(
            stops.starts_with("<stop offset=\"0\" stop-color=\"#ffffff\" stop-opacity=\"0.5000\">")
        );
        assert!(stops.contains("offset=\"0.0625\""));
        assert!(stops.ends_with(
            "<stop offset=\"1\" stop-color=\"#ffffff\" stop-opacity=\"0.0056\"></stop>"
        ));
    }

    #[test]
    fn knob_chamfer_is_brightest_facing_the_light_and_darkest_opposite() {
        let gradient = knob_chamfer_gradient();
        let level = |degrees: u32| -> u32 {
            let tag = format!(" {degrees}deg");
            let end = gradient.find(&tag).unwrap();
            let start = gradient[..end].rfind("rgb(").unwrap() + 4;
            gradient[start..end]
                .split(',')
                .next()
                .unwrap()
                .parse()
                .unwrap()
        };
        // The light's bearing is 315 degrees: the chamfer there faces it.
        assert!(level(310) > level(220) && level(310) > level(40));
        assert!(level(130) < level(220) && level(130) < level(40));
        assert!(gradient.starts_with("conic-gradient(rgb("));
        assert!(gradient.ends_with(" 360deg)"));
    }

    #[test]
    fn silver_top_is_the_ring_metal_facing_up() {
        // Same material: the top sits between the chamfer's lit and shaded
        // sides because it faces the light less squarely than the lit side.
        let top = knob_silver_top();
        assert!(top.starts_with("rgb("));
        assert!(css_variables().contains(&format!("--knob-silver-top:{top}")));
    }

    #[test]
    fn css_variables_carry_the_same_light() {
        let css = css_variables();
        assert!(css.contains("--light-x:-0.7071"));
        assert!(css.contains("--shadow-y:0.7071"));
        assert!(css.contains("--light-angle:135.00deg"));
    }
}
