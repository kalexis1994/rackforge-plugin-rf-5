//! The CONFIG page: the cassette interface (`crate::cassette`) driven
//! through RackForge's program drafts, with files read from the player's
//! disk and saved back to it.

use super::*;
use crate::cassette::{
    self, Job, Lamp, SavedFile, Status, Step, Task, export_steps, install_steps, listen_steps,
    loaded_status, stop_steps,
};
use crate::transfer::{Medium, document};
use web_sys::{Blob, BlobPropertyBag, DragEvent, File, HtmlAnchorElement, HtmlInputElement, Url};

/// How long a step waits for a draft to come or go in the context.
const DRAFT_WAIT_MS: i32 = 6_000;
const FILE_INPUT_ID: &str = "tape-file";

pub(super) fn render_html(app: &App) -> String {
    let (sounds, banks) = app.context.as_ref().map_or((&[][..], &[][..]), |context| {
        (&context.instance.sounds[..], &context.instance.banks[..])
    });
    cassette::render(&app.cassette, sounds, banks)
}

/// The draft RackForge shows the page: its id and document.
fn current_draft(app: &AppHandle) -> Option<(u64, serde_json::Value)> {
    let state = app.borrow();
    let draft = state.context.as_ref()?.program_draft.as_ref()?;
    let document = serde_json::from_str(draft.document_json.as_deref()?).ok()?;
    Some((draft.draft_id, document))
}

fn set_status(app: &AppHandle, status: Status) {
    app.borrow_mut().cassette.status = status;
}

fn redraw(app: &AppHandle) {
    app.borrow().render();
}

/// Starts a job; one held open for listening hands its draft on.
fn start(app: &AppHandle, task: Task, steps: std::collections::VecDeque<Step>, status: Status) {
    {
        let mut state = app.borrow_mut();
        let draft = state.cassette.job.take().and_then(|job| job.draft);
        let mut job = Job::new(task, steps);
        job.draft = draft;
        state.cassette.job = Some(job);
        state.cassette.status = status;
    }
    redraw(app);
    run(app);
}

/// Fails the job: says why, and lets go of a draft it opened.
fn fail(app: &AppHandle, error: &str) {
    let draft = app
        .borrow_mut()
        .cassette
        .job
        .take()
        .and_then(|job| job.draft.map(|(id, _)| id));
    if let Some(draft_id) = draft {
        request(
            app,
            "plugin.cancel_program",
            serde_json::json!({ "draft_id": draft_id }),
            |_, _| {},
        );
    }
    set_status(app, Status::error("ERROR", &error.to_uppercase()));
    redraw(app);
}

/// Sends a step's request; its answer takes the job on.
fn send(app: &AppHandle, method: &str, params: serde_json::Value) {
    let generation = {
        let mut state = app.borrow_mut();
        let Some(job) = state.cassette.job.as_mut() else {
            return;
        };
        job.waiting = true;
        job.generation
    };
    request(app, method, params, move |app, result| {
        let current = app
            .borrow()
            .cassette
            .job
            .as_ref()
            .is_some_and(|job| job.generation == generation && job.waiting);
        if !current {
            return;
        }
        match result {
            Ok(_) => {
                {
                    let mut state = app.borrow_mut();
                    if let Some(job) = state.cassette.job.as_mut() {
                        match job.steps.front() {
                            Some(Step::Save) => {
                                job.installed += 1;
                                job.draft = None;
                            }
                            Some(Step::Cancel) => job.draft = None,
                            _ => {}
                        }
                        job.advance();
                    }
                }
                run(app);
            }
            Err(error) => fail(app, &error),
        }
    });
}

/// Fails the job if it is still waiting on the same step when the time is
/// up: RackForge never showed the draft coming or going.
fn watch(app: &AppHandle) {
    let generation = match app.borrow().cassette.job.as_ref() {
        Some(job) => job.generation,
        None => return,
    };
    let weak = Rc::downgrade(app);
    let timeout = Closure::once_into_js(move || {
        let Some(app) = weak.upgrade() else {
            return;
        };
        let stuck = app
            .borrow()
            .cassette
            .job
            .as_ref()
            .is_some_and(|job| job.generation == generation && !job.waiting);
        if stuck {
            fail(&app, "RackForge did not open or close the program in time");
        }
    });
    let _ = app
        .borrow()
        .window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            timeout.unchecked_ref(),
            DRAFT_WAIT_MS,
        );
}

/// Takes the job as far as it can go without waiting on RackForge.
pub(super) fn run(app: &AppHandle) {
    loop {
        let (step, draft_id) = {
            let state = app.borrow();
            let Some(job) = state.cassette.job.as_ref() else {
                return;
            };
            if job.waiting {
                return;
            }
            (
                job.steps.front().cloned(),
                job.draft.as_ref().map(|(id, _)| *id),
            )
        };
        let Some(step) = step else {
            finish(app);
            return;
        };
        match step {
            Step::Begin(program_id) => {
                send(
                    app,
                    "plugin.begin_program_edit",
                    serde_json::json!({ "program_id": program_id }),
                );
                return;
            }
            Step::AwaitDraft => {
                let Some(draft) = current_draft(app) else {
                    watch(app);
                    return;
                };
                let mut state = app.borrow_mut();
                if let Some(job) = state.cassette.job.as_mut() {
                    job.draft = Some(draft);
                    job.advance();
                }
            }
            Step::Capture => {
                let mut state = app.borrow_mut();
                if let Some(job) = state.cassette.job.as_mut() {
                    if let Some((_, document)) = job.draft.clone() {
                        job.captured.push(document);
                    }
                    job.advance();
                    if let Some(progress) = job.progress() {
                        state.cassette.status.display = progress;
                    }
                }
                drop(state);
                redraw(app);
            }
            Step::Replace(index) => {
                // The draft as RackForge shows it now: after one replace,
                // the next builds on its latest document.
                let draft = current_draft(app)
                    .filter(|(id, _)| Some(*id) == draft_id)
                    .or_else(|| app.borrow().cassette.job.as_ref()?.draft.clone());
                let replaced = {
                    let state = app.borrow();
                    let tape = state.cassette.tape.as_ref();
                    draft.as_ref().zip(tape).and_then(|((_, document), tape)| {
                        let program = tape.reading.programs.get(index)?;
                        document::with_program(document, &tape.name(index), &program.memory)
                    })
                };
                let (Some((draft_id, _)), Some(replaced)) = (draft, replaced) else {
                    fail(app, "The tape's program could not be prepared");
                    return;
                };
                send(
                    app,
                    "plugin.replace_program_draft",
                    serde_json::json!({ "draft_id": draft_id, "document": replaced }),
                );
                return;
            }
            Step::Save | Step::Cancel => {
                let method = if step == Step::Save {
                    "plugin.save_program"
                } else {
                    "plugin.cancel_program"
                };
                let Some(draft_id) = draft_id.or_else(|| current_draft(app).map(|(id, _)| id))
                else {
                    // Nothing open to close.
                    if let Some(job) = app.borrow_mut().cassette.job.as_mut() {
                        job.advance();
                    }
                    continue;
                };
                send(app, method, serde_json::json!({ "draft_id": draft_id }));
                return;
            }
            Step::AwaitClear => {
                if current_draft(app).is_some() {
                    watch(app);
                    return;
                }
                let mut state = app.borrow_mut();
                if let Some(job) = state.cassette.job.as_mut() {
                    job.advance();
                    if let Some(progress) = job.progress() {
                        state.cassette.status.display = progress;
                    }
                }
                drop(state);
                redraw(app);
            }
            Step::Hold => {
                let status = {
                    let state = app.borrow();
                    let index = state.cassette.listening();
                    let tape = state.cassette.tape.as_ref();
                    index.zip(tape).map(|(index, tape)| {
                        Status::new(
                            &format!("LISTEN {}", tape.reading.programs[index].place),
                            &format!("{} · PLAY THE KEYBOARD", tape.name(index)),
                            Lamp::Loading,
                        )
                    })
                };
                if let Some(status) = status {
                    set_status(app, status);
                }
                redraw(app);
                return;
            }
        }
    }
}

/// A job's last step is done.
fn finish(app: &AppHandle) {
    let Some(job) = app.borrow_mut().cassette.job.take() else {
        return;
    };
    let status = match job.task {
        Task::Export { medium, .. } => match cassette::saved_file(medium, &job.captured) {
            Some(file) => {
                let status = Status::new(
                    &format!("SAVED {}", job.captured.len()),
                    &file.name,
                    Lamp::Off,
                );
                if let Err(error) = download(app, &file) {
                    Status::error("ERROR", &format!("THE FILE COULD NOT BE SAVED: {error:?}"))
                } else {
                    status
                }
            }
            None => Status::error("ERROR", "NO PROGRAMS COULD BE READ"),
        },
        Task::Install { .. } => Status::new(
            &format!("INSTALLED {}", job.installed),
            "IN THE USER BANK, ON THE PANEL'S PROGRAM SELECTOR",
            Lamp::Off,
        ),
        Task::Listen(_) | Task::StopListening => {
            let state = app.borrow();
            state
                .cassette
                .tape
                .as_ref()
                .map_or_else(Status::ready, loaded_status)
        }
    };
    set_status(app, status);
    redraw(app);
}

/// Saves a file to the player's disk, as a download.
fn download(app: &AppHandle, file: &SavedFile) -> Result<(), JsValue> {
    let (document, window) = {
        let state = app.borrow();
        (state.document.clone(), state.window.clone())
    };
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(&file.bytes[..]));
    let options = BlobPropertyBag::new();
    options.set_type(file.media_type);
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &options)?;
    let url = Url::create_object_url_with_blob(&blob)?;
    let anchor = document
        .create_element("a")?
        .dyn_into::<HtmlAnchorElement>()?;
    anchor.set_href(&url);
    anchor.set_download(&file.name);
    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("missing body"))?;
    body.append_child(&anchor)?;
    anchor.click();
    anchor.remove();
    let revoke = Closure::once_into_js(move || {
        let _ = Url::revoke_object_url(&url);
    });
    window.set_timeout_with_callback_and_timeout_and_arguments_0(revoke.unchecked_ref(), 30_000)?;
    Ok(())
}

fn start_export(app: &AppHandle) {
    let (ids, medium, capacity, listening) = {
        let state = app.borrow();
        let sounds = state
            .context
            .as_ref()
            .map_or(&[][..], |context| &context.instance.sounds[..]);
        (
            state.cassette.marked_ids(sounds),
            state.cassette.medium,
            state.cassette.capacity(),
            state.cassette.listening().is_some(),
        )
    };
    if ids.is_empty() {
        set_status(
            app,
            Status::new(
                "NO PROGRAMS",
                "MARK PROGRAMS IN THE PROGRAM MEMORY TO SAVE THEM",
                Lamp::Off,
            ),
        );
        redraw(app);
        return;
    }
    if ids.len() > capacity {
        set_status(
            app,
            Status::new(
                "TOO MANY",
                &format!(
                    "MARK {capacity} OR FEWER TO SAVE THEM AS {}",
                    medium.label()
                ),
                Lamp::Off,
            ),
        );
        redraw(app);
        return;
    }
    let total = ids.len();
    start(
        app,
        Task::Export { medium, total },
        export_steps(&ids, listening),
        Status::new(
            &format!("SAVING 1/{total}"),
            &format!("READING THE PROGRAMS FOR THE {}", medium.label()),
            Lamp::Saving,
        ),
    );
}

fn start_install(app: &AppHandle) {
    let (indices, free, listening) = {
        let state = app.borrow();
        let sounds = state
            .context
            .as_ref()
            .map_or(&[][..], |context| &context.instance.sounds[..]);
        (
            state
                .cassette
                .tape
                .as_ref()
                .map(|tape| tape.marked_indices())
                .unwrap_or_default(),
            cassette::free_places(sounds),
            state.cassette.listening().is_some(),
        )
    };
    if indices.is_empty() || indices.len() > free {
        return;
    }
    let total = indices.len();
    start(
        app,
        Task::Install { total },
        install_steps(&indices, listening),
        Status::new(
            &format!("LOADING 1/{total}"),
            "INSTALLING IN THE USER BANK",
            Lamp::Loading,
        ),
    );
}

fn listen(app: &AppHandle, index: usize) {
    let listening = app.borrow().cassette.listening();
    if listening == Some(index) {
        start(app, Task::StopListening, stop_steps(), Status::ready());
        return;
    }
    let place = app
        .borrow()
        .cassette
        .tape
        .as_ref()
        .and_then(|tape| {
            tape.reading
                .programs
                .get(index)
                .map(|program| program.place.clone())
        })
        .unwrap_or_default();
    start(
        app,
        Task::Listen(index),
        listen_steps(index, listening.is_some()),
        Status::new(
            &format!("LISTEN {place}"),
            "OPENING THE PROGRAM",
            Lamp::Loading,
        ),
    );
}

/// Reads a file LOAD FROM TAPE was given.
pub(super) fn load_file(app: &AppHandle, file: File) {
    if app.borrow().cassette.busy() {
        return;
    }
    let name = file.name();
    set_status(
        app,
        Status::new("READING", &name.to_uppercase(), Lamp::Loading),
    );
    redraw(app);
    if file.size() > cassette::MAX_FILE_BYTES as f64 {
        set_status(
            app,
            Status::error("LOAD ERROR", "THE FILE IS TOO LARGE FOR A TAPE"),
        );
        redraw(app);
        return;
    }
    let loaded_app = app.clone();
    let loaded = Closure::once(move |buffer: JsValue| {
        let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
        match cassette::load(&name, &bytes) {
            Ok(tape) => {
                // The held program is the old tape's: let it go first.
                if loaded_app.borrow().cassette.listening().is_some() {
                    start(
                        &loaded_app,
                        Task::StopListening,
                        stop_steps(),
                        Status::ready(),
                    );
                }
                let status = loaded_status(&tape);
                loaded_app.borrow_mut().cassette.tape = Some(tape);
                set_status(&loaded_app, status);
            }
            Err(error) => set_status(&loaded_app, Status::error("LOAD ERROR", &error)),
        }
        redraw(&loaded_app);
    });
    let failed_app = app.clone();
    let failed = Closure::once(move |_: JsValue| {
        set_status(
            &failed_app,
            Status::error("LOAD ERROR", "THE FILE COULD NOT BE READ"),
        );
        redraw(&failed_app);
    });
    let _ = file.array_buffer().then2(&loaded, &failed);
    loaded.forget();
    failed.forget();
}

fn index_of(element: &Element) -> Option<usize> {
    element.get_attribute("data-index")?.parse().ok()
}

/// A key or row pressed on the CONFIG page.
pub(super) fn click(app: &AppHandle, element: &Element, action: &str) {
    let busy = app.borrow().cassette.busy();
    match action {
        "tape-load" if !busy => {
            release_key(app, "button[data-action='tape-load']");
            if let Some(input) = app
                .borrow()
                .document
                .get_element_by_id(FILE_INPUT_ID)
                .and_then(|input| input.dyn_into::<HtmlInputElement>().ok())
            {
                input.set_value("");
                input.click();
            }
        }
        "tape-save" if !busy => {
            start_export(app);
            release_key(app, "button[data-action='tape-save']");
        }
        "format" if !busy => {
            let medium = match element.get_attribute("data-medium").as_deref() {
                Some("SYSEX") => Medium::Sysex,
                Some("FILE") => Medium::Document,
                _ => Medium::Tape,
            };
            app.borrow_mut().cassette.medium = medium;
            redraw(app);
            release_key(app, &format!("button[data-medium='{}']", medium.label()));
        }
        "mark-memory" if !busy => {
            if let Some(id) = element.get_attribute("data-id") {
                let mut state = app.borrow_mut();
                if !state.cassette.marked.remove(&id) {
                    state.cassette.marked.insert(id);
                }
            }
            redraw(app);
        }
        "mark-bank" if !busy => {
            let all = element.get_attribute("data-mark").as_deref() == Some("all");
            let rows = element
                .closest(".program-bank")
                .ok()
                .flatten()
                .and_then(|bank| bank.query_selector_all("[data-action='mark-memory']").ok());
            if let Some(rows) = rows {
                let mut state = app.borrow_mut();
                for index in 0..rows.length() {
                    let Some(id) = rows
                        .item(index)
                        .and_then(|node| node.dyn_into::<Element>().ok())
                        .and_then(|row| row.get_attribute("data-id"))
                    else {
                        continue;
                    };
                    if all {
                        state.cassette.marked.insert(id);
                    } else {
                        state.cassette.marked.remove(&id);
                    }
                }
            }
            redraw(app);
        }
        "mark-tape" if !busy => {
            if let Some(index) = index_of(element)
                && let Some(tape) = app.borrow_mut().cassette.tape.as_mut()
                && let Some(marked) = tape.marked.get_mut(index)
            {
                *marked = !*marked;
            }
            redraw(app);
        }
        "mark-tape-all" if !busy => {
            let all = element.get_attribute("data-mark").as_deref() == Some("all");
            if let Some(tape) = app.borrow_mut().cassette.tape.as_mut() {
                tape.marked.fill(all);
            }
            redraw(app);
        }
        "eject" if !busy => {
            if app.borrow().cassette.listening().is_some() {
                start(app, Task::StopListening, stop_steps(), Status::ready());
            }
            let mut state = app.borrow_mut();
            state.cassette.tape = None;
            if state.cassette.job.is_none() {
                state.cassette.status = Status::ready();
            }
            drop(state);
            redraw(app);
        }
        "listen" if !busy => {
            if let Some(index) = index_of(element) {
                listen(app, index);
            }
        }
        "install" if !busy => {
            start_install(app);
            release_key(app, "button[data-action='install']");
        }
        _ => {}
    }
}

/// A new context: the draft a job waits on may have come or gone.
pub(super) fn on_context(app: &AppHandle) {
    // RackForge closed a draft held for listening (another surface took
    // over, or the instance changed): stop listening.
    let dropped = app.borrow().cassette.listening().is_some() && current_draft(app).is_none();
    if dropped {
        let mut state = app.borrow_mut();
        state.cassette.job = None;
        state.cassette.status = state
            .cassette
            .tape
            .as_ref()
            .map_or_else(Status::ready, loaded_status);
    }
    redraw(app);
    run(app);
}

/// The file picker LOAD FROM TAPE opens, dropped files, and letting go of
/// a held draft when the page goes away.
pub(super) fn install(app: &AppHandle) -> Result<(), JsValue> {
    let (document, window, root) = {
        let state = app.borrow();
        (
            state.document.clone(),
            state.window.clone(),
            state.root.clone(),
        )
    };
    let input = document
        .create_element("input")?
        .dyn_into::<HtmlInputElement>()?;
    input.set_type("file");
    input.set_id(FILE_INPUT_ID);
    input.set_accept(".wav,.wave,.syx,.json,audio/wav,audio/x-wav");
    input.set_hidden(true);
    let picked_app = app.clone();
    let picked = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
        let file = event
            .target()
            .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
            .and_then(|input| input.files())
            .and_then(|files| files.get(0));
        if let Some(file) = file {
            load_file(&picked_app, file);
        }
    });
    input.add_event_listener_with_callback("change", picked.as_ref().unchecked_ref())?;
    picked.forget();
    document
        .body()
        .ok_or_else(|| JsValue::from_str("missing body"))?
        .append_child(&input)?;

    for event_name in ["dragenter", "dragover", "dragleave", "drop"] {
        let drop_app = app.clone();
        let handler = Closure::<dyn FnMut(DragEvent)>::new(move |event: DragEvent| {
            let carries_files = event
                .data_transfer()
                .is_some_and(|transfer| transfer.types().includes(&JsValue::from_str("Files"), 0));
            if !carries_files {
                return;
            }
            event.prevent_default();
            let dropping = match event_name {
                "dragenter" | "dragover" => true,
                // Leaving for a child of the page is not leaving it.
                "dragleave" => event.related_target().is_some(),
                _ => false,
            };
            if app_dropping(&drop_app) != dropping {
                drop_app.borrow_mut().cassette.dropping = dropping;
                redraw(&drop_app);
            }
            if event_name == "drop"
                && let Some(file) = event
                    .data_transfer()
                    .and_then(|transfer| transfer.files())
                    .and_then(|files| files.get(0))
            {
                load_file(&drop_app, file);
            }
        });
        root.add_event_listener_with_callback(event_name, handler.as_ref().unchecked_ref())?;
        handler.forget();
    }

    let leaving_app = app.clone();
    let leaving = Closure::<dyn FnMut(Event)>::new(move |_: Event| {
        let draft = leaving_app
            .borrow()
            .cassette
            .job
            .as_ref()
            .and_then(|job| job.draft.as_ref().map(|(id, _)| *id));
        if let Some(draft_id) = draft {
            request(
                &leaving_app,
                "plugin.cancel_program",
                serde_json::json!({ "draft_id": draft_id }),
                |_, _| {},
            );
        }
    });
    window.add_event_listener_with_callback("pagehide", leaving.as_ref().unchecked_ref())?;
    leaving.forget();
    Ok(())
}

fn app_dropping(app: &AppHandle) -> bool {
    app.borrow().cassette.dropping
}
