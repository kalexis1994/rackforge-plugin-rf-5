//! The CONFIG surface: the Rev 3's cassette interface, for passing programs
//! on. SAVE TO TAPE writes the programs marked in the RF-5's memory to a
//! file, as a tape recording, a Rev 4 SysEx dump or RackForge program
//! documents; LOAD FROM TAPE reads any of them back, to listen to and to
//! install in the USER bank.
//!
//! RackForge holds the programs, not the page. A program's memory is only
//! reached through a program draft: begun for a program and read off the
//! context, then cancelled, to save it out; begun empty, replaced by the tape's
//! program and saved, to install one. Listening is a draft held open with a
//! tape's program in it, which RackForge plays. Each of those is a run of
//! [`Step`]s the page takes one at a time, as RackForge answers.
//!
//! What is here is the state and its drawing, free of the browser; the page
//! drives it from `browser::config`.

use std::collections::{BTreeSet, VecDeque};

use serde::Deserialize;
use serde_json::Value;

use crate::transfer::{
    self, Medium, Memory, Reading, clean_name, document, sysex,
    tape::{self, FILE_PROGRAMS},
};
use crate::{escape_html, prophet_switch_svg, switch_svg};

/// How many of its own programs the RF-5 keeps.
pub const USER_PROGRAMS: usize = 64;
/// The largest file LOAD FROM TAPE reads: minutes of a stereo recording.
pub const MAX_FILE_BYTES: usize = 160 * 1024 * 1024;
/// Switch drawings number their gradients; the cassette's take their own
/// range.
const KEY_INDEX: u32 = 1_200;

/// A program in the instance's catalog, as the context lists it.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Sound {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub bank: Option<String>,
    /// One of RackForge's CUSTOM programs, the RF-5's USER bank.
    #[serde(default)]
    pub editable: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Bank {
    pub id: String,
    pub name: String,
}

/// A file read by LOAD FROM TAPE.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadedTape {
    pub file_name: String,
    pub reading: Reading,
    pub marked: Vec<bool>,
}

impl LoadedTape {
    /// All its programs marked to install, unless the reading could not
    /// vouch for them.
    pub fn new(file_name: &str, reading: Reading) -> Self {
        let trusted = reading.medium != Medium::Tape
            || reading.warning.is_none()
            || reading
                .warning
                .as_deref()
                .is_some_and(|warning| warning.ends_with("CHECKSUM GOOD"));
        Self {
            file_name: file_name.to_owned(),
            marked: vec![trusted; reading.programs.len()],
            reading,
        }
    }

    /// The name a program is installed under: its own, or the file's with
    /// its place on the tape ("FACT1R3 2-4").
    pub fn name(&self, index: usize) -> String {
        let program = &self.reading.programs[index];
        program
            .name
            .clone()
            .or_else(|| clean_name(&format!("{} {}", file_stem(&self.file_name), program.place)))
            .unwrap_or_else(|| format!("TAPE {}", program.place))
    }

    pub fn marked_indices(&self) -> Vec<usize> {
        (0..self.marked.len())
            .filter(|index| self.marked[*index])
            .collect()
    }
}

/// A file's name without its extension, upper case as the panel prints.
pub fn file_stem(file_name: &str) -> String {
    let base = file_name.rsplit(['/', '\\']).next().unwrap_or(file_name);
    let stem = base
        .split('.')
        .next()
        .filter(|stem| !stem.is_empty())
        .unwrap_or(base);
    stem.to_uppercase()
}

/// What the tape keys' LEDs show.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lamp {
    Off,
    /// SAVE TO TAPE lit: programs are being written out.
    Saving,
    /// LOAD FROM TAPE lit: a file is being read or its programs installed.
    Loading,
    /// LOAD FROM TAPE blinking, as the Prophet shows a failed load.
    Error,
}

/// The red display and the line printed under it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Status {
    pub display: String,
    pub note: String,
    pub lamp: Lamp,
}

impl Status {
    pub fn ready() -> Self {
        Self::new("READY", "", Lamp::Off)
    }

    pub fn new(display: &str, note: &str, lamp: Lamp) -> Self {
        Self {
            display: display.to_owned(),
            note: note.to_owned(),
            lamp,
        }
    }

    pub fn error(display: &str, note: &str) -> Self {
        Self::new(display, note, Lamp::Error)
    }
}

/// One exchange with RackForge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Step {
    /// Begin a draft: of a program, or empty (`None`) for a new one.
    Begin(Option<String>),
    /// Wait for the draft to reach the page in the context.
    AwaitDraft,
    /// Keep the draft's document: the program being saved out.
    Capture,
    /// Put the tape's program at this index into the draft.
    Replace(usize),
    Save,
    Cancel,
    /// Wait for the draft to leave the context.
    AwaitClear,
    /// Keep the draft open, sounding: listening to a tape's program.
    Hold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Task {
    Export { medium: Medium, total: usize },
    Install { total: usize },
    Listen(usize),
    StopListening,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    pub task: Task,
    pub steps: VecDeque<Step>,
    /// The draft this job opened, and its latest document.
    pub draft: Option<(u64, Value)>,
    /// The documents captured for SAVE TO TAPE.
    pub captured: Vec<Value>,
    pub installed: usize,
    /// A request is out; the next step waits for its answer.
    pub waiting: bool,
    /// Counts the steps taken, so a watchdog knows whether the job moved.
    pub generation: u64,
}

impl Job {
    pub fn new(task: Task, steps: VecDeque<Step>) -> Self {
        Self {
            task,
            steps,
            draft: None,
            captured: Vec::new(),
            installed: 0,
            waiting: false,
            generation: 0,
        }
    }

    /// Holding a tape's program open to listen to it.
    pub fn listening(&self) -> Option<usize> {
        match (self.task, self.steps.front()) {
            (Task::Listen(index), Some(Step::Hold)) => Some(index),
            _ => None,
        }
    }

    pub fn advance(&mut self) {
        self.steps.pop_front();
        self.waiting = false;
        self.generation += 1;
    }

    /// The display's count while it runs: "SAVING 12/40".
    pub fn progress(&self) -> Option<String> {
        match self.task {
            Task::Export { total, .. } => Some(format!(
                "SAVING {}/{total}",
                (self.captured.len() + 1).min(total)
            )),
            Task::Install { total } => Some(format!(
                "LOADING {}/{total}",
                (self.installed + 1).min(total)
            )),
            Task::Listen(_) | Task::StopListening => None,
        }
    }
}

/// The steps to close a draft held open for listening, first.
fn closing(listening: bool) -> impl Iterator<Item = Step> {
    listening
        .then_some([Step::Cancel, Step::AwaitClear])
        .into_iter()
        .flatten()
}

/// Reads each program's document through a draft of it, cancelled.
pub fn export_steps(ids: &[String], listening: bool) -> VecDeque<Step> {
    closing(listening)
        .chain(ids.iter().flat_map(|id| {
            [
                Step::Begin(Some(id.clone())),
                Step::AwaitDraft,
                Step::Capture,
                Step::Cancel,
                Step::AwaitClear,
            ]
        }))
        .collect()
}

/// Installs each tape program as a new program, saved from an empty draft.
pub fn install_steps(indices: &[usize], listening: bool) -> VecDeque<Step> {
    closing(listening)
        .chain(indices.iter().flat_map(|index| {
            [
                Step::Begin(None),
                Step::AwaitDraft,
                Step::Replace(*index),
                Step::Save,
                Step::AwaitClear,
            ]
        }))
        .collect()
}

/// Opens a draft, or reuses the one held, with the tape's program in it.
pub fn listen_steps(index: usize, holding: bool) -> VecDeque<Step> {
    let open = (!holding).then_some([Step::Begin(None), Step::AwaitDraft]);
    open.into_iter()
        .flatten()
        .chain([Step::Replace(index), Step::Hold])
        .collect()
}

pub fn stop_steps() -> VecDeque<Step> {
    [Step::Cancel, Step::AwaitClear].into()
}

/// The CONFIG surface's state.
#[derive(Clone, Debug, PartialEq)]
pub struct Cassette {
    /// What SAVE TO TAPE writes.
    pub medium: Medium,
    /// Programs marked in the RF-5's memory, by id, for SAVE TO TAPE.
    pub marked: BTreeSet<String>,
    pub tape: Option<LoadedTape>,
    pub job: Option<Job>,
    pub status: Status,
    /// A file is being dragged over the page.
    pub dropping: bool,
}

impl Default for Cassette {
    fn default() -> Self {
        Self {
            medium: Medium::Tape,
            marked: BTreeSet::new(),
            tape: None,
            job: None,
            status: Status::ready(),
            dropping: false,
        }
    }
}

impl Cassette {
    pub fn listening(&self) -> Option<usize> {
        self.job.as_ref().and_then(Job::listening)
    }

    /// A job is running; the keys wait for it.
    pub fn busy(&self) -> bool {
        self.job.is_some() && self.listening().is_none()
    }

    /// Marked programs still in the catalog, in its order.
    pub fn marked_ids(&self, sounds: &[Sound]) -> Vec<String> {
        sounds
            .iter()
            .filter(|sound| self.marked.contains(&sound.id))
            .map(|sound| sound.id.clone())
            .collect()
    }

    /// How many programs one save can write in the chosen medium.
    pub fn capacity(&self) -> usize {
        match self.medium {
            Medium::Tape => FILE_PROGRAMS,
            Medium::Sysex => sysex::MAX_PROGRAMS,
            Medium::Document => USER_PROGRAMS + FILE_PROGRAMS,
        }
    }
}

/// Places left in the USER bank.
pub fn free_places(sounds: &[Sound]) -> usize {
    USER_PROGRAMS.saturating_sub(sounds.iter().filter(|sound| sound.editable).count())
}

/// A file name the host's file system accepts.
fn safe_file_name(name: &str) -> String {
    let name: String = name
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || " -_().,".contains(character) {
                character
            } else {
                '_'
            }
        })
        .collect();
    let name = name.trim().to_owned();
    if name.is_empty() {
        "RF-5".to_owned()
    } else {
        name
    }
}

/// A saved file: its name, contents and media type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedFile {
    pub name: String,
    pub bytes: Vec<u8>,
    pub media_type: &'static str,
}

/// Writes the captured documents out in the chosen medium.
pub fn saved_file(medium: Medium, documents: &[Value]) -> Option<SavedFile> {
    let programs: Vec<(String, Memory)> = documents.iter().filter_map(document::program).collect();
    let [first, ..] = programs.as_slice() else {
        return None;
    };
    let title = if programs.len() == 1 {
        safe_file_name(&first.0)
    } else {
        format!("RF-5 {} programs", programs.len())
    };
    Some(match medium {
        Medium::Tape => {
            let memories: Vec<Memory> = programs.iter().map(|(_, memory)| *memory).collect();
            SavedFile {
                name: format!("{title}.wav"),
                bytes: tape::write(&memories),
                media_type: "audio/wav",
            }
        }
        Medium::Sysex => SavedFile {
            name: format!("{title}.syx"),
            bytes: sysex::write(
                programs
                    .iter()
                    .map(|(name, memory)| (name.as_str(), memory)),
            ),
            media_type: "application/octet-stream",
        },
        Medium::Document => SavedFile {
            name: if documents.len() == 1 {
                format!("{title}.rackforge-program.json")
            } else {
                format!("{title}.rf5-programs.json")
            },
            bytes: document::write(documents),
            media_type: "application/json",
        },
    })
}

/// What LOAD FROM TAPE reports on a file it read.
pub fn loaded_status(tape: &LoadedTape) -> Status {
    let count = tape.reading.programs.len();
    let what = match (tape.reading.medium, count) {
        (Medium::Tape, 8) => "PROPHET-5 TAPE, ONE BANK".to_owned(),
        (Medium::Tape, _) => "PROPHET-5 TAPE, ONE PROGRAM FILE".to_owned(),
        (Medium::Sysex, _) => "PROPHET-5 REV 4 SYSEX".to_owned(),
        (Medium::Document, _) => "RF-5 PROGRAM FILE".to_owned(),
    };
    let checked = match (tape.reading.medium, &tape.reading.warning) {
        (_, Some(warning)) => warning.clone(),
        (Medium::Tape, None) => "CHECKSUM GOOD".to_owned(),
        _ => String::new(),
    };
    let note = [what, checked]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    let lamp = if tape.marked.iter().any(|marked| *marked) {
        Lamp::Off
    } else {
        Lamp::Error
    };
    Status::new(
        &format!("{count} PROGRAM{}", if count == 1 { "" } else { "S" }),
        &note,
        lamp,
    )
}

/// Reads a file LOAD FROM TAPE was given.
pub fn load(file_name: &str, bytes: &[u8]) -> Result<LoadedTape, String> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err("THE FILE IS TOO LARGE FOR A TAPE".to_owned());
    }
    transfer::read(bytes)
        .map(|reading| LoadedTape::new(file_name, reading))
        .map_err(|error| error.0)
}

/// A catalog name split into its place and name: "1-1 Brass" is "1-1",
/// "Brass".
fn place_and_name(name: &str) -> (&str, &str) {
    match name.split_once(' ') {
        Some((place, rest))
            if place.len() == 3
                && place.as_bytes()[1] == b'-'
                && place.as_bytes()[0].is_ascii_digit()
                && place.as_bytes()[2].is_ascii_digit() =>
        {
            (place, rest)
        }
        _ => ("", name),
    }
}

/// A key: the panel's switch with its LED, its name printed over it.
fn key(
    index: u32,
    label: &str,
    action: &str,
    lit: bool,
    classes: &str,
    attributes: &str,
    disabled: bool,
) -> String {
    let light = classes.contains("tune-button");
    format!(
        "<div class=\"page-key cassette-key\"><span class=\"control-label\">{label}</span><button type=\"button\" class=\"hardware-button {classes}{}\" data-action=\"{action}\" {attributes} aria-pressed=\"{lit}\" aria-label=\"{}\"{}>{}</button></div>",
        if lit { " active" } else { "" },
        label.replace("<br>", " "),
        if disabled { " disabled" } else { "" },
        switch_svg(KEY_INDEX + index, lit, None, light, None),
    )
}

fn outline(class: &str, title: &str, body: &str) -> String {
    format!(
        "<section class=\"control-group {class}\"><svg class=\"section-outline\" aria-hidden=\"true\" preserveAspectRatio=\"none\"><path></path></svg><h2><span>{title}</span></h2>{body}</section>"
    )
}

fn legend(class: &str, span: usize, controls: &str, label: &str) -> String {
    format!(
        "<div class=\"control-legend {class}\" style=\"--legend-span:{span}\"><div class=\"legend-controls\">{controls}</div><div class=\"legend-line\"><svg class=\"legend-rule\" aria-hidden=\"true\" preserveAspectRatio=\"none\"><path></path></svg><span>{label}</span></div></div>"
    )
}

fn render_deck(state: &Cassette) -> String {
    let busy = state.busy();
    let saving = state.status.lamp == Lamp::Saving;
    let loading = state.status.lamp == Lamp::Loading;
    let blinking = state.status.lamp == Lamp::Error;
    let keys = format!(
        "{}{}",
        key(0, "SAVE<br>TO TAPE", "tape-save", saving, "", "", busy),
        key(
            1,
            "LOAD<br>FROM TAPE",
            "tape-load",
            loading || blinking,
            &format!("tune-button{}", if blinking { " blinking" } else { "" }),
            "",
            busy
        ),
    );
    let mut formats = String::new();
    for (position, medium) in [Medium::Tape, Medium::Sysex, Medium::Document]
        .into_iter()
        .enumerate()
    {
        let active = state.medium == medium;
        formats.push_str(&format!(
            "<div class=\"page-key\"><span class=\"control-label\">{label}</span><button type=\"button\" class=\"hardware-button format-button{}\" data-action=\"format\" data-medium=\"{label}\" aria-pressed=\"{active}\" aria-label=\"Save as {label}\"{}>{}</button></div>",
            if active { " active" } else { "" },
            if busy { " disabled" } else { "" },
            prophet_switch_svg(KEY_INDEX + 10 + position as u32, active, None, false),
            label = medium.label(),
        ));
    }
    let window = format!(
        "<div class=\"tape-window\"><div class=\"led-display\" role=\"status\" aria-live=\"polite\"><span>{}</span></div><p class=\"tape-note\">{}</p></div>",
        escape_html(&state.status.display),
        escape_html(&state.status.note),
    );
    outline(
        "group-cassette",
        "CASSETTE INTERFACE",
        &format!(
            "<div class=\"cassette-deck\"><div class=\"cassette-keys\">{keys}</div>{window}{}</div>",
            legend("legend-format", 3, &formats, "FORMAT")
        ),
    )
}

/// A program's row: its LED marks it. A tape's program with no name of
/// its own shows the one it is installed under, greyed.
fn program_row(
    action: &str,
    data: &str,
    (place, name, named): (&str, &str, bool),
    marked: bool,
    disabled: bool,
) -> String {
    format!(
        "<button type=\"button\" class=\"program-row{}\" data-action=\"{action}\" {data} aria-pressed=\"{marked}\"{}><span class=\"row-led\" aria-hidden=\"true\"></span><span class=\"row-place\">{}</span><span class=\"row-name{}\">{}</span></button>",
        if marked { " marked" } else { "" },
        if disabled { " disabled" } else { "" },
        escape_html(place),
        if named { "" } else { " unnamed" },
        escape_html(name),
    )
}

fn bank_name(bank: Option<&str>, banks: &[Bank], editable: bool) -> String {
    if editable {
        return "USER".to_owned();
    }
    bank.and_then(|id| banks.iter().find(|candidate| candidate.id == id))
        .map_or_else(|| "FACTORY".to_owned(), |bank| bank.name.to_uppercase())
}

fn render_memory(state: &Cassette, sounds: &[Sound], banks: &[Bank]) -> String {
    let busy = state.busy();
    // Banks in catalog order, the USER bank always shown, last.
    let mut groups: Vec<(String, Vec<&Sound>)> = Vec::new();
    for sound in sounds.iter().filter(|sound| !sound.editable) {
        let name = bank_name(sound.bank.as_deref(), banks, false);
        match groups.iter_mut().find(|(group, _)| *group == name) {
            Some((_, members)) => members.push(sound),
            None => groups.push((name, vec![sound])),
        }
    }
    groups.push((
        "USER".to_owned(),
        sounds.iter().filter(|sound| sound.editable).collect(),
    ));
    let mut body = String::new();
    for (position, (name, members)) in groups.iter().enumerate() {
        let mut rows = String::new();
        for sound in members {
            let (place, program) = place_and_name(&sound.name);
            rows.push_str(&program_row(
                "mark-memory",
                &format!("data-id=\"{}\"", escape_html(&sound.id)),
                (place, program, true),
                state.marked.contains(&sound.id),
                busy,
            ));
        }
        if members.is_empty() {
            rows.push_str("<p class=\"rows-empty\">NO PROGRAMS OF YOUR OWN YET: RECORD ONE ON THE PANEL, OR LOAD A TAPE AND INSTALL IT</p>");
        }
        let count = members
            .iter()
            .filter(|sound| state.marked.contains(&sound.id))
            .count();
        body.push_str(&format!(
            "<div class=\"program-bank\"><div class=\"bank-head\"><span class=\"bank-name\">{}</span><span class=\"bank-count\">{count}/{}</span><span class=\"bank-rule\" aria-hidden=\"true\"></span><button type=\"button\" class=\"text-key\" data-action=\"mark-bank\" data-bank=\"{position}\" data-mark=\"all\"{disabled}>ALL</button><button type=\"button\" class=\"text-key\" data-action=\"mark-bank\" data-bank=\"{position}\" data-mark=\"none\"{disabled}>NONE</button></div><div class=\"program-rows\">{rows}</div></div>",
            escape_html(name),
            members.len(),
            disabled = if busy || members.is_empty() { " disabled" } else { "" },
        ));
    }
    let marked = state.marked_ids(sounds).len();
    let capacity = state.capacity();
    let foot = if marked > capacity {
        format!(
            "{marked} MARKED: A {} HOLDS {capacity}",
            match state.medium {
                Medium::Tape => "TAPE",
                Medium::Sysex => "SYSEX DUMP",
                Medium::Document => "FILE",
            }
        )
    } else {
        format!("{marked} MARKED TO SAVE TO TAPE")
    };
    outline(
        "group-memory",
        "PROGRAM MEMORY",
        &format!(
            "{body}<p class=\"section-foot{}\">{foot}</p>",
            if marked > capacity { " over" } else { "" }
        ),
    )
}

fn render_tape(state: &Cassette, sounds: &[Sound]) -> String {
    let busy = state.busy();
    let Some(tape) = state.tape.as_ref() else {
        return outline(
            "group-tape empty",
            "TAPE",
            "<div class=\"tape-empty\"><p class=\"tape-empty-lead\">PRESS LOAD FROM TAPE, OR DROP A FILE HERE</p><p>PROPHET-5 REV 3 CASSETTE RECORDINGS (.WAV)</p><p>PROPHET-5 REV 4 SYSEX DUMPS (.SYX)</p><p>RF-5 PROGRAM FILES (.JSON)</p></div>",
        );
    };
    let listening = state.listening();
    let mut rows = String::new();
    for (index, program) in tape.reading.programs.iter().enumerate() {
        let name = tape.name(index);
        let listen = listening == Some(index);
        rows.push_str(&format!(
            "<div class=\"tape-row{}\">{}<button type=\"button\" class=\"listen-key{}\" data-action=\"listen\" data-index=\"{index}\" aria-pressed=\"{listen}\" aria-label=\"Listen to {}\"{}><svg viewBox=\"0 0 12 12\" aria-hidden=\"true\"><path d=\"{}\"></path></svg></button></div>",
            if listen { " listening" } else { "" },
            program_row(
                "mark-tape",
                &format!("data-index=\"{index}\""),
                (&program.place, &name, program.name.is_some()),
                tape.marked[index],
                busy,
            ),
            if listen { " active" } else { "" },
            escape_html(&name),
            if busy { " disabled" } else { "" },
            if listen {
                "M3 3h6v6H3z"
            } else {
                "M3.5 2.2 10 6l-6.5 3.8z"
            },
        ));
    }
    let marked = tape.marked_indices().len();
    let free = free_places(sounds);
    let foot = if marked > free {
        format!("{marked} MARKED, {free} PLACES FREE IN THE USER BANK")
    } else {
        format!("{marked} MARKED · USER BANK: {free} OF {USER_PROGRAMS} FREE")
    };
    let install = key(
        2,
        "INSTALL<br>MARKED",
        "install",
        false,
        "record-button",
        "",
        busy || marked == 0 || marked > free,
    );
    let warning = tape
        .reading
        .warning
        .as_deref()
        .filter(|warning| !warning.ends_with("CHECKSUM GOOD"))
        .map(|warning| format!("<p class=\"tape-warning\">{}</p>", escape_html(warning)))
        .unwrap_or_default();
    outline(
        "group-tape",
        "TAPE",
        &format!(
            "<div class=\"tape-head\"><span class=\"tape-file\">{}</span><button type=\"button\" class=\"text-key\" data-action=\"mark-tape-all\" data-mark=\"all\"{disabled}>ALL</button><button type=\"button\" class=\"text-key\" data-action=\"mark-tape-all\" data-mark=\"none\"{disabled}>NONE</button><button type=\"button\" class=\"text-key\" data-action=\"eject\"{disabled}>EJECT</button></div>{warning}<div class=\"program-rows\">{rows}</div><div class=\"tape-foot\"><p class=\"section-foot{}\">{foot}</p>{install}</div>",
            escape_html(&tape.file_name),
            if marked > free { " over" } else { "" },
            disabled = if busy { " disabled" } else { "" },
        ),
    )
}

/// The CONFIG page's panel.
pub fn render(state: &Cassette, sounds: &[Sound], banks: &[Bank]) -> String {
    format!(
        "<main class=\"hardware-panel config-panel{}\"><div class=\"panel-surface config-surface\">{}{}{}</div></main>",
        if state.dropping { " dropping" } else { "" },
        render_deck(state),
        render_memory(state, sounds, banks),
        render_tape(state, sounds),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transfer::{INIT_PROGRAM, Program};
    use serde_json::json;

    fn sound(id: &str, name: &str, editable: bool) -> Sound {
        Sound {
            id: id.to_owned(),
            name: name.to_owned(),
            bank: Some(
                if editable {
                    "user"
                } else {
                    "factory.rf5.original"
                }
                .to_owned(),
            ),
            editable,
        }
    }

    fn tape_reading(count: usize, warning: Option<&str>) -> Reading {
        Reading {
            medium: Medium::Tape,
            programs: (0..count)
                .map(|index| Program {
                    place: transfer::bank_place(index),
                    name: None,
                    memory: INIT_PROGRAM,
                })
                .collect(),
            warning: warning.map(str::to_owned),
        }
    }

    #[test]
    fn tape_programs_are_named_after_their_file_and_place() {
        let tape = LoadedTape::new("tapes/FACT1R3.WAV", tape_reading(40, None));
        assert_eq!(tape.name(9), "FACT1R3 2-2");
        assert!(tape.marked.iter().all(|marked| *marked));
        assert_eq!(file_stem("my.tape.wav"), "MY");
    }

    #[test]
    fn a_tape_that_failed_its_checksum_is_not_marked() {
        let tape = LoadedTape::new("x.wav", tape_reading(40, Some("CHECKSUM ERROR: X")));
        assert!(tape.marked.iter().all(|marked| !*marked));
        assert_eq!(loaded_status(&tape).lamp, Lamp::Error);
        let tape = LoadedTape::new(
            "x.wav",
            tape_reading(8, Some("2 UNCLEAR BITS, CHECKSUM GOOD")),
        );
        assert!(tape.marked.iter().all(|marked| *marked));
        assert_eq!(loaded_status(&tape).display, "8 PROGRAMS");
    }

    #[test]
    fn saving_reads_each_program_through_a_cancelled_draft() {
        let steps = export_steps(&["a".to_owned(), "b".to_owned()], true);
        assert_eq!(steps.len(), 12);
        assert_eq!(steps[0], Step::Cancel);
        assert_eq!(steps[2], Step::Begin(Some("a".to_owned())));
        assert_eq!(steps[4], Step::Capture);
        let steps = install_steps(&[3], false);
        assert_eq!(
            steps,
            VecDeque::from([
                Step::Begin(None),
                Step::AwaitDraft,
                Step::Replace(3),
                Step::Save,
                Step::AwaitClear
            ])
        );
        assert_eq!(
            listen_steps(2, true),
            VecDeque::from([Step::Replace(2), Step::Hold])
        );
    }

    #[test]
    fn a_held_draft_is_listening() {
        let mut job = Job::new(Task::Listen(4), listen_steps(4, false));
        assert_eq!(job.listening(), None);
        job.advance();
        job.advance();
        job.advance();
        assert_eq!(job.listening(), Some(4));
        let state = Cassette {
            job: Some(job),
            ..Cassette::default()
        };
        assert!(!state.busy());
    }

    #[test]
    fn saved_files_take_the_medium_and_the_programs_names() {
        let documents: Vec<Value> = ["Brass", "Strings"]
            .iter()
            .map(|name| {
                json!({
                    "plugin_id": document::PLUGIN_ID,
                    "name": name,
                    "payload": { "memory": document::to_hex(&INIT_PROGRAM) },
                })
            })
            .collect();
        let tape = saved_file(Medium::Tape, &documents).unwrap();
        assert_eq!(tape.name, "RF-5 2 programs.wav");
        assert_eq!(transfer::read(&tape.bytes).unwrap().programs.len(), 8);
        let one = saved_file(Medium::Document, &documents[..1]).unwrap();
        assert_eq!(one.name, "Brass.rackforge-program.json");
        let dump = saved_file(Medium::Sysex, &documents).unwrap();
        assert_eq!(
            transfer::read(&dump.bytes).unwrap().programs[1]
                .name
                .as_deref(),
            Some("STRINGS")
        );
        assert_eq!(saved_file(Medium::Tape, &[]), None);
        assert_eq!(safe_file_name("a/b:c"), "a_b_c");
    }

    #[test]
    fn the_panel_lists_the_memory_by_bank_and_the_tape() {
        let sounds = [
            sound("original-11-brass", "1-1 Brass", false),
            sound("custom.user-01", "My <Horns>", true),
        ];
        let mut state = Cassette::default();
        state.marked.insert("custom.user-01".to_owned());
        state.tape = Some(LoadedTape::new("FACT1R3.WAV", tape_reading(40, None)));
        let html = render(&state, &sounds, &[]);
        assert!(html.contains("CASSETTE INTERFACE"));
        assert!(
            html.contains(
                "<span class=\"row-place\">1-1</span><span class=\"row-name\">Brass</span>"
            )
        );
        assert!(html.contains("My &lt;Horns&gt;"));
        assert!(html.contains("1 MARKED TO SAVE TO TAPE"));
        assert!(html.contains("40 MARKED · USER BANK: 63 OF 64 FREE"));
        assert_eq!(html.matches("data-action=\"listen\"").count(), 40);
        assert_eq!(place_and_name("Organ 2"), ("", "Organ 2"));
    }

    #[test]
    fn installing_more_than_the_user_bank_holds_is_refused() {
        let sounds: Vec<Sound> = (0..60)
            .map(|index| sound(&format!("custom.user-{index}"), "x", true))
            .collect();
        let state = Cassette {
            tape: Some(LoadedTape::new("FACT1R3.WAV", tape_reading(40, None))),
            ..Cassette::default()
        };
        let html = render(&state, &sounds, &[]);
        assert!(html.contains("40 MARKED, 4 PLACES FREE IN THE USER BANK"));
        assert!(html.contains(
            "data-action=\"install\"  aria-pressed=\"false\" aria-label=\"INSTALL MARKED\" disabled"
        ));
    }
}
