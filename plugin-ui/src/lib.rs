#[cfg(any(target_arch = "wasm32", test))]
mod panel;

#[cfg(any(target_arch = "wasm32", test))]
use serde::Deserialize;

#[cfg(any(target_arch = "wasm32", test))]
const PROTOCOL: &str = "rackforge.plugin.web@1";

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Debug, Deserialize)]
struct Sound {
    id: String,
    name: String,
    bank: String,
}

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
    use web_sys::{Document, Element, Event, MessageEvent, MouseEvent, PointerEvent, Window};

    type AppHandle = Rc<RefCell<App>>;
    type ResponseHandler = Box<dyn FnOnce(&AppHandle, Result<JsValue, String>)>;

    #[derive(Debug, Deserialize)]
    struct HostContext {
        instance: Instance,
    }

    #[derive(Debug, Deserialize)]
    struct Instance {
        selected_sound_id: String,
        sounds: Vec<Sound>,
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

    struct App {
        window: Window,
        document: Document,
        root: Element,
        host_origin: String,
        context: Option<HostContext>,
        snapshot: Option<ParameterSnapshot>,
        parameter_values: BTreeMap<u32, f64>,
        pending: BTreeMap<String, ResponseHandler>,
        sequence: u64,
        refresh_generation: u64,
        active_section: String,
        pending_sound_id: Option<String>,
        active_parameter_drag: Option<u32>,
        render_after_drag: bool,
        bridge_error: String,
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
            Ok(Rc::new(RefCell::new(Self {
                window,
                document,
                root,
                host_origin,
                context: None,
                snapshot: None,
                parameter_values: BTreeMap::new(),
                pending: BTreeMap::new(),
                sequence: 0,
                refresh_generation: 0,
                active_section: "modulation".to_owned(),
                pending_sound_id: None,
                active_parameter_drag: None,
                render_after_drag: false,
                bridge_error: String::new(),
            })))
        }

        fn value(&self, parameter: &Parameter) -> f64 {
            self.parameter_values
                .get(&parameter.index)
                .copied()
                .unwrap_or_else(|| parameter.kind.default_value())
        }

        fn selected_sound_id(&self) -> Option<&str> {
            self.pending_sound_id.as_deref().or_else(|| {
                self.context
                    .as_ref()
                    .map(|context| context.instance.selected_sound_id.as_str())
            })
        }

        fn selected_sound(&self) -> Option<&Sound> {
            let id = self.selected_sound_id()?;
            self.context
                .as_ref()?
                .instance
                .sounds
                .iter()
                .find(|sound| sound.id == id)
        }

        fn render(&self) {
            let mut html = String::from("<div class=\"rf5-frame\">");
            html.push_str(&self.render_header());
            html.push_str(&self.render_tabs());
            html.push_str(&self.render_panel());
            html.push_str(&self.render_programs());
            html.push_str("</div>");
            self.root.set_inner_html(&html);
        }

        fn render_header(&self) -> String {
            let program = self
                .selected_sound()
                .map(|sound| escape_html(&sound.name))
                .unwrap_or_else(|| "Waiting for RackForge".to_owned());
            format!(
                "<header class=\"instrument-header\"><div class=\"identity\"><strong>RF-5</strong><span>FIVE-VOICE PROGRAMMABLE POLYPHONIC SYNTHESIZER</span></div><div class=\"current-program\"><small>CURRENT PROGRAM</small><strong>{program}</strong></div></header>"
            )
        }

        fn render_tabs(&self) -> String {
            let mut html =
                String::from("<nav class=\"panel-tabs\" aria-label=\"RF-5 panel sections\">");
            for section in panel::SECTIONS {
                let active = section.id == self.active_section;
                html.push_str(&format!(
                    "<button type=\"button\" class=\"panel-tab{}\" data-action=\"section\" data-section=\"{}\" aria-pressed=\"{}\"><strong>{}</strong><span>{}</span></button>",
                    if active { " active" } else { "" },
                    section.id,
                    active,
                    section.label,
                    section.caption
                ));
            }
            html.push_str("</nav>");
            html
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
                let mut controls = String::new();
                for id in group.parameter_ids {
                    if let Some(parameter) = snapshot
                        .schema
                        .parameters
                        .iter()
                        .find(|parameter| parameter.id == *id)
                    {
                        controls.push_str(&self.render_control(parameter));
                    }
                }
                groups.push_str(&format!(
                    "<section class=\"control-group group-{}\"><h2>{}</h2><div class=\"control-grid\">{controls}</div></section>",
                    section.id,
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
            let action = if parameter.id == "tune" {
                "momentary"
            } else {
                "toggle"
            };
            format!(
                "<div class=\"parameter-control button-control\"><span class=\"control-label\">{}</span>{symbol}<span class=\"led{}\" aria-hidden=\"true\"></span><button type=\"button\" class=\"hardware-button{}\" data-action=\"{action}\" data-index=\"{}\" data-rackforge-parameter-index=\"{}\" aria-label=\"{}\" aria-pressed=\"{}\"></button><output data-output-index=\"{}\">{}</output></div>",
                panel_label(&parameter.id, &parameter.name),
                if active { " on" } else { "" },
                if active { " active" } else { "" },
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
            let ticks = knob_ticks();
            format!(
                "<div class=\"parameter-control knob-control\"><span class=\"control-label\">{}</span><div class=\"knob-shell\" data-knob-index=\"{}\" data-rackforge-parameter-index=\"{}\" style=\"--knob-turn:{angle:.3}deg\"><svg class=\"knob-scale\" viewBox=\"0 0 100 100\" aria-hidden=\"true\">{ticks}</svg><span class=\"knob-cap\"><span class=\"knob-marker\"></span></span><input class=\"knob-input\" type=\"range\" data-action=\"parameter\" data-index=\"{}\" min=\"{minimum}\" max=\"{maximum}\" step=\"{step}\" value=\"{value}\" aria-label=\"{}\"></div><output data-output-index=\"{}\">{}</output></div>",
                panel_label(&parameter.id, &parameter.name),
                parameter.index,
                parameter.index,
                parameter.index,
                escape_html(&parameter.name),
                parameter.index,
                format_value(parameter, value)
            )
        }

        fn render_programs(&self) -> String {
            let Some(context) = self.context.as_ref() else {
                return "<section class=\"program-library waiting\">Waiting for the RackForge program catalog…</section>".to_owned();
            };
            let selected = self.selected_sound_id().unwrap_or_default();
            let mut cards = String::new();
            let mut current_bank = "";
            for sound in &context.instance.sounds {
                if sound.bank != current_bank {
                    if !current_bank.is_empty() {
                        cards.push_str("</div>");
                    }
                    current_bank = &sound.bank;
                    cards.push_str(&format!(
                        "<h3>{}</h3><div class=\"program-grid\">",
                        bank_label(current_bank)
                    ));
                }
                let active = sound.id == selected;
                cards.push_str(&format!(
                    "<button type=\"button\" class=\"program-button{}\" data-action=\"sound\" data-sound-id=\"{}\" aria-pressed=\"{}\"><span>{}</span><strong>{}</strong></button>",
                    if active { " active" } else { "" },
                    escape_html(&sound.id),
                    active,
                    if active { "●" } else { "○" },
                    escape_html(&sound.name)
                ));
            }
            if !current_bank.is_empty() {
                cards.push_str("</div>");
            }
            format!(
                "<section class=\"program-library\"><header><div><small>PROGRAM MEMORY</small><h2>RF-5 Programs</h2></div><span>{} programs</span></header>{cards}</section>",
                context.instance.sounds.len()
            )
        }
    }

    fn normalized(value: f64, minimum: f64, maximum: f64) -> f64 {
        if maximum > minimum {
            ((value - minimum) / (maximum - minimum)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn knob_ticks() -> String {
        let mut ticks = String::new();
        for index in 0..=10 {
            let angle = -135.0 + index as f64 * 27.0;
            let class = if matches!(index, 0 | 5 | 10) {
                " major"
            } else {
                ""
            };
            ticks.push_str(&format!(
                "<line class=\"knob-tick{class}\" x1=\"50\" y1=\"3\" x2=\"50\" y2=\"11\" transform=\"rotate({angle} 50 50)\"></line>"
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
            "wheel-mod-source-mix" => "LFO / NOISE",
            "wheel-mod-oscillator-a-frequency" => "FREQ A",
            "wheel-mod-oscillator-b-frequency" => "FREQ B",
            "wheel-mod-oscillator-a-pulse-width" => "PW A",
            "wheel-mod-oscillator-b-pulse-width" => "PW B",
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
            "<svg class=\"wave-symbol\" viewBox=\"0 0 42 22\" aria-hidden=\"true\"><path d=\"M4 18L35 4V18\"></path></svg>"
        } else if id.ends_with("-triangle") {
            "<svg class=\"wave-symbol\" viewBox=\"0 0 42 22\" aria-hidden=\"true\"><path d=\"M3 18L21 4L39 18\"></path></svg>"
        } else if id.ends_with("-square") || id.ends_with("-pulse") {
            "<svg class=\"wave-symbol\" viewBox=\"0 0 42 22\" aria-hidden=\"true\"><path d=\"M3 18V4H21V18H39V4\"></path></svg>"
        } else {
            ""
        }
    }

    fn bank_label(bank: &str) -> String {
        match bank {
            "factory.rf5.baseline" => "BASELINE PROGRAMS".to_owned(),
            "factory.rf5.audition" => "AUDITION PROGRAMS".to_owned(),
            _ => escape_html(bank),
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
            },
        );
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
            document.query_selector(&format!("[data-action=toggle][data-index='{index}']"))
        {
            let active = value >= 0.5;
            let _ = button.set_attribute("aria-pressed", if active { "true" } else { "false" });
            button.set_class_name(if active {
                "hardware-button active"
            } else {
                "hardware-button"
            });
            if let Some(parent) = button.parent_element()
                && let Ok(Some(led)) = parent.query_selector(".led")
            {
                led.set_class_name(if active { "led on" } else { "led" });
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
                        click_app.borrow_mut().active_section = section;
                        click_app.borrow().render();
                    }
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
                Some("sound") => {
                    if let Some(sound_id) = element.get_attribute("data-sound-id") {
                        click_app.borrow_mut().pending_sound_id = Some(sound_id.clone());
                        click_app.borrow().render();
                        let selected = sound_id.clone();
                        request(
                            &click_app,
                            "plugin.select_sound",
                            serde_json::json!({"sound_id": sound_id}),
                            move |app, result| match result {
                                Ok(_) => {
                                    let mut state = app.borrow_mut();
                                    if let Some(context) = state.context.as_mut() {
                                        context.instance.selected_sound_id = selected;
                                    }
                                    state.pending_sound_id = None;
                                    state.bridge_error.clear();
                                    drop(state);
                                    refresh_parameters(app);
                                }
                                Err(error) => {
                                    let mut state = app.borrow_mut();
                                    state.pending_sound_id = None;
                                    state.bridge_error = error;
                                    drop(state);
                                    app.borrow().render();
                                }
                            },
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
                    "pointercancel" | "lostpointercapture" => {
                        if active
                            .borrow()
                            .as_ref()
                            .is_some_and(|(id, ..)| *id == pointer_id)
                        {
                            *active.borrow_mut() = None;
                            finish_drag(&drag_app);
                        }
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
        install_events(&app)?;
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
    fn knob_drag_is_relative_quantized_and_clamped() {
        assert_eq!(
            relative_knob_value(0.5, 0.0, 0.0, 1.0, 1.0 / 127.0),
            64.0 / 127.0
        );
        assert_eq!(relative_knob_value(0.5, 500.0, 0.0, 1.0, 1.0 / 127.0), 1.0);
        assert_eq!(relative_knob_value(0.5, -500.0, 0.0, 1.0, 1.0 / 127.0), 0.0);
    }

    #[test]
    fn host_defaults_accept_numbers_and_booleans() {
        let number: ParameterDefault = serde_json::from_str("0.625").unwrap();
        let enabled: ParameterDefault = serde_json::from_str("true").unwrap();
        assert_eq!(number.as_f64(), 0.625);
        assert_eq!(enabled.as_f64(), 1.0);
    }

    #[test]
    fn protocol_and_catalog_identity_are_stable() {
        let sound = Sound {
            id: "baseline-init".to_owned(),
            name: "RF-5 Init".to_owned(),
            bank: "factory.rf5.baseline".to_owned(),
        };
        assert_eq!(PROTOCOL, "rackforge.plugin.web@1");
        assert_eq!(sound.bank, "factory.rf5.baseline");
        assert_eq!(sound.id, "baseline-init");
        assert_eq!(sound.name, "RF-5 Init");
    }
}
