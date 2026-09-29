//! The RF-5's own programs: what RECORD saves, kept as the Rev 3 kept them.
//!
//! A program is the 24 bytes of the original's program memory (each pot at
//! its 128 positions, each switch a bit), carried in a RackForge program
//! document as hex. Saving from the panel stores exactly what the Rev 3
//! would have; recalling one is a program button's recall. RackForge owns
//! the files: it hands every saved document back through `prepare` and
//! `install` when an instance starts, so a document must stay installable
//! whatever version wrote it.
//!
//! The documents and their envelopes are RackForge's (`rackforge-program-api`);
//! see "Portable individual-program editing" in its docs/PLUGIN_DEVELOPMENT.md.

use std::collections::BTreeMap;

use rackforge_program_api::{
    PROGRAM_EDIT_SCHEMA_VERSION, PROGRAM_EDITOR_SCHEMA_VERSION, PROGRAM_SCHEMA_VERSION,
    PreparedProgram, ProgramDocument, ProgramEditRequest, ProgramEditorChoice, ProgramEditorField,
    ProgramEditorFieldKind, ProgramEditorPage, ProgramEditorValue, ProgramEditorView,
};
use rf_5_dsp::{Engine, PROGRAM_MEMORY_BYTES};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

/// The plugin, as its manifest names it.
pub const PLUGIN_ID: &str = "org.rackforge.rf-5";
/// How many of its own programs the RF-5 keeps.
pub const MAX_PROGRAMS: usize = 64;
/// The longest name RackForge keeps.
const MAX_NAME_CHARS: usize = 64;
/// What `payload` holds: `{ "memory": "<48 hex digits>" }`.
const PAYLOAD_VERSION: u32 = 1;
/// The catalog id of an own program is this and its document's id.
const CUSTOM: &str = "custom.";
/// The bank the catalog lists them in; declared in presets.json.
const USER_BANK: &str = "user";
/// Their catalog order, after the factory programs.
const USER_ORDER: i64 = 1_000;

const FACTORY_CATALOG: &str = include_str!("../package/metadata/presets.json");
const RUNTIME: &str = include_str!("../package/metadata/runtime.json");

/// The RF-5's own programs, by document id.
pub type Library = BTreeMap<String, ProgramDocument>;

fn read<T: DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    serde_json::from_slice(bytes).ok()
}

fn write<T: Serialize>(value: &T, destination: &mut [u8]) -> Option<usize> {
    let bytes = serde_json::to_vec(value).ok()?;
    destination.get_mut(..bytes.len())?.copy_from_slice(&bytes);
    Some(bytes.len())
}

/// The state version the runtime descriptor declares, stamped on new documents.
fn state_version() -> Option<u32> {
    let runtime: Value = serde_json::from_str(RUNTIME).ok()?;
    u32::try_from(runtime.get("state_version")?.as_u64()?).ok()
}

fn to_hex(memory: &[u8; PROGRAM_MEMORY_BYTES]) -> String {
    memory.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn from_hex(hex: &str) -> Option<[u8; PROGRAM_MEMORY_BYTES]> {
    if hex.len() != PROGRAM_MEMORY_BYTES * 2 || !hex.is_ascii() {
        return None;
    }
    let mut memory = [0; PROGRAM_MEMORY_BYTES];
    for (byte, pair) in memory.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
        *byte = u8::from_str_radix(core::str::from_utf8(pair).ok()?, 16).ok()?;
    }
    Some(memory)
}

/// The memory a document carries, when it is one of the RF-5's. Any state
/// version is accepted: the memory format does not change with it.
pub fn memory(document: &ProgramDocument) -> Option<[u8; PROGRAM_MEMORY_BYTES]> {
    document.validate().ok()?;
    let name = document.name.trim();
    if document.plugin_id != PLUGIN_ID
        || document.payload_version != PAYLOAD_VERSION
        || name.is_empty()
        || name.chars().count() > MAX_NAME_CHARS
    {
        return None;
    }
    from_hex(document.payload.get("memory")?.as_str()?)
}

/// The envelope RackForge stores a document in and selects it by.
fn envelope(document: ProgramDocument) -> Option<PreparedProgram> {
    memory(&document)?;
    let prepared = PreparedProgram {
        schema_version: PROGRAM_EDIT_SCHEMA_VERSION,
        storage_path: format!("programs/{}.rackforge-program.json", document.id),
        preview_sound_id: format!("{CUSTOM}{}", document.id),
        document,
        artifacts: Vec::new(),
    };
    prepared.validate().ok()?;
    Some(prepared)
}

/// A prepared program exactly as `envelope` makes it, or nothing.
pub fn validated_prepared(bytes: &[u8]) -> Option<ProgramDocument> {
    let prepared: PreparedProgram = read(bytes)?;
    prepared.validate().ok()?;
    let expected = envelope(prepared.document.clone())?;
    (prepared == expected).then_some(prepared.document)
}

/// The name a factory program goes by in the catalog.
fn factory_name(id: &str) -> Option<String> {
    let catalog: Value = serde_json::from_str(FACTORY_CATALOG).ok()?;
    catalog
        .get("presets")?
        .as_array()?
        .iter()
        .find(|preset| preset.get("id").and_then(Value::as_str) == Some(id))?
        .get("name")?
        .as_str()
        .map(str::to_owned)
}

/// Starts a draft: of one of the RF-5's own programs, to save over it; of a
/// factory program, as a copy; or, with no id, of what the panel holds now,
/// stored as RECORD would store it.
pub fn begin(
    library: &Library,
    engine: &Engine,
    request: &[u8],
    destination: &mut [u8],
) -> Option<usize> {
    let request: ProgramEditRequest = read(request)?;
    request.validate().ok()?;
    if let Some(id) = request
        .program_id
        .as_deref()
        .and_then(|id| id.strip_prefix(CUSTOM))
    {
        return write(&envelope(library.get(id)?.clone())?, destination);
    }
    let (memory, name) = match request.program_id.as_deref() {
        None => (engine.program_memory(), "New program".to_owned()),
        Some(id) => (Engine::factory_program_memory(id)?, factory_name(id)?),
    };
    let id = (1..=MAX_PROGRAMS)
        .map(|number| format!("user-{number:02}"))
        .find(|id| !library.contains_key(id))?;
    let document = ProgramDocument {
        schema_version: PROGRAM_SCHEMA_VERSION,
        id,
        name,
        plugin_id: PLUGIN_ID.to_owned(),
        plugin_version: env!("CARGO_PKG_VERSION").to_owned(),
        plugin_state_version: state_version()?,
        payload_version: PAYLOAD_VERSION,
        category: None,
        tags: vec!["rf5".to_owned(), "user".to_owned()],
        payload: json!({ "memory": to_hex(&memory) }),
    };
    write(&envelope(document)?, destination)
}

/// Checks a document (a renamed draft, or a saved one coming back) and
/// envelopes it for RackForge to store and install.
pub fn prepare(document: &[u8], destination: &mut [u8]) -> Option<usize> {
    write(&envelope(read(document)?)?, destination)
}

/// Adds a prepared program to the library, or replaces the one with its id.
/// A new one is refused when the library is full or would no longer fit the
/// catalog in `transfer_bytes`.
pub fn install(library: &mut Library, prepared: &[u8], transfer_bytes: usize) -> bool {
    let Some(document) = validated_prepared(prepared) else {
        return false;
    };
    if !library.contains_key(&document.id) && library.len() >= MAX_PROGRAMS {
        return false;
    }
    let mut next = library.clone();
    next.insert(document.id.clone(), document);
    if catalog(&next, &mut vec![0; transfer_bytes]).is_none() {
        return false;
    }
    *library = next;
    true
}

/// The catalog: the factory programs, then the RF-5's own in the user bank.
pub fn catalog(library: &Library, destination: &mut [u8]) -> Option<usize> {
    let mut catalog: Value = serde_json::from_str(FACTORY_CATALOG).ok()?;
    let presets = catalog.get_mut("presets")?.as_array_mut()?;
    for (order, document) in (USER_ORDER..).zip(library.values()) {
        presets.push(json!({
            "id": format!("{CUSTOM}{}", document.id),
            "name": document.name.trim(),
            "bank": USER_BANK,
            "order": order,
            "tags": ["rf5", "user"],
            "editable": true,
        }));
    }
    write(&catalog, destination)
}

/// The memory of one of the RF-5's own programs, by its catalog id.
pub fn own_memory(library: &Library, catalog_id: &str) -> Option<[u8; PROGRAM_MEMORY_BYTES]> {
    memory(library.get(catalog_id.strip_prefix(CUSTOM)?)?)
}

/// The editor RackForge's controller surfaces show for a draft. A program is
/// the whole panel, so the tree only says what it is stored as; the panel
/// itself is where it is edited.
pub fn view(document: &[u8], destination: &mut [u8]) -> Option<usize> {
    let document: ProgramDocument = read(document)?;
    memory(&document)?;
    let view = ProgramEditorView {
        schema_version: PROGRAM_EDITOR_SCHEMA_VERSION,
        title: "RF-5 program".to_owned(),
        pages: vec![ProgramEditorPage {
            id: "program".to_owned(),
            label: "Program".to_owned(),
            detail: "The whole panel, as the Rev 3's memory stores it".to_owned(),
            enabled: true,
            pages: Vec::new(),
            fields: vec![ProgramEditorField {
                id: "memory".to_owned(),
                label: "Memory".to_owned(),
                detail: "24 bytes: 128 positions per pot, a bit per switch".to_owned(),
                value: ProgramEditorValue::Choice("rev3".to_owned()),
                kind: ProgramEditorFieldKind::Choice {
                    options: vec![ProgramEditorChoice {
                        value: "rev3".to_owned(),
                        label: "Prophet-5 Rev 3".to_owned(),
                        detail: None,
                    }],
                },
                live_preview: false,
            }],
        }],
    };
    view.validate().ok()?;
    write(&view, destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MAX_TRANSFER_BYTES, Rf5Processor};
    use rackforge_plugin_sdk::ParallelProcessor;

    fn call(
        processor: &mut Rf5Processor,
        method: fn(&mut Rf5Processor, &[u8], &mut [u8]) -> Option<usize>,
        input: &[u8],
    ) -> Option<Vec<u8>> {
        let mut out = vec![0; MAX_TRANSFER_BYTES];
        method(processor, input, &mut out).map(|length| out[..length].to_vec())
    }

    fn begin(processor: &mut Rf5Processor, program_id: Option<&str>) -> Option<PreparedProgram> {
        let request =
            serde_json::to_vec(&ProgramEditRequest::new(program_id.map(str::to_owned))).unwrap();
        let bytes = call(processor, Rf5Processor::begin_program_edit, &request)?;
        Some(serde_json::from_slice(&bytes).unwrap())
    }

    /// What `plugin.set_program_name` then `plugin.save_program` do.
    fn save(
        processor: &mut Rf5Processor,
        mut document: ProgramDocument,
        name: &str,
    ) -> PreparedProgram {
        document.name = name.to_owned();
        let bytes = call(
            processor,
            Rf5Processor::prepare_program_save,
            &serde_json::to_vec(&document).unwrap(),
        )
        .expect("a renamed draft prepares");
        assert!(processor.install_program(&bytes));
        serde_json::from_slice(&bytes).unwrap()
    }

    fn catalog(processor: &mut Rf5Processor) -> Value {
        let mut out = vec![0; MAX_TRANSFER_BYTES];
        let length = processor
            .write_program_catalog(&mut out)
            .expect("catalog fits");
        serde_json::from_slice(&out[..length]).unwrap()
    }

    #[test]
    fn record_stores_the_panel_as_the_rev_3_memory_holds_it() {
        let mut processor = Rf5Processor::default();
        let panel = processor.engine.program_memory();
        let draft = begin(&mut processor, None).expect("a new program begins");
        assert_eq!(draft.document.id, "user-01");
        assert_eq!(draft.preview_sound_id, "custom.user-01");
        assert_eq!(
            draft.storage_path,
            "programs/user-01.rackforge-program.json"
        );
        assert_eq!(memory(&draft.document), Some(panel));
        assert_eq!(
            draft.document.plugin_state_version,
            state_version().unwrap()
        );

        let saved = save(&mut processor, draft.document, "Mi Brass");
        let listed = catalog(&mut processor);
        let entry = listed["presets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|preset| preset["id"] == "custom.user-01")
            .expect("the saved program is listed");
        assert_eq!(entry["name"], "Mi Brass");
        assert_eq!(entry["bank"], "user");
        assert_eq!(entry["editable"], true);
        assert!(
            listed["banks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|bank| bank["id"] == "user")
        );

        // Another program, then the saved one back: the panel it recorded.
        assert!(processor.load_preset("original-12-low-strings"));
        assert_ne!(processor.engine.program_memory(), panel);
        assert!(processor.load_preset(&saved.preview_sound_id));
        assert_eq!(processor.engine.program_memory(), panel);
    }

    #[test]
    fn a_saved_program_is_saved_over_and_a_factory_one_copied() {
        let mut processor = Rf5Processor::default();
        let first = begin(&mut processor, None).unwrap();
        save(&mut processor, first.document, "Pad");

        let again = begin(&mut processor, Some("custom.user-01")).expect("its own program reopens");
        assert_eq!(again.document.id, "user-01");
        save(&mut processor, again.document, "Pad 2");
        assert_eq!(processor.programs.len(), 1);
        assert_eq!(processor.programs["user-01"].name, "Pad 2");

        let copy = begin(&mut processor, Some("original-12-low-strings"))
            .expect("a factory program copies");
        assert_eq!(copy.document.id, "user-02");
        assert_eq!(copy.document.name, "1-2 Low Strings");
        assert_eq!(
            memory(&copy.document),
            Engine::factory_program_memory("original-12-low-strings")
        );
        assert!(begin(&mut processor, Some("custom.user-99")).is_none());
        assert!(begin(&mut processor, Some("no-such-program")).is_none());
    }

    #[test]
    fn saved_programs_come_back_when_an_instance_starts() {
        let mut recorder = Rf5Processor::default();
        let draft = begin(&mut recorder, None).unwrap();
        let saved = save(&mut recorder, draft.document, "Stored");
        // RackForge keeps the document and hands it back to a new instance:
        // prepare, then install, before it first reads the catalog.
        let stored = serde_json::to_vec(&saved.document).unwrap();
        let mut restarted = Rf5Processor::default();
        let prepared = call(&mut restarted, Rf5Processor::prepare_program_save, &stored).unwrap();
        assert!(restarted.install_program(&prepared));
        assert!(restarted.load_preset("custom.user-01"));
        assert_eq!(
            restarted.engine.program_memory(),
            memory(&saved.document).unwrap()
        );

        // A document an older RF-5 state version wrote still installs.
        let mut older = saved.document.clone();
        older.plugin_state_version = 1;
        let prepared = call(
            &mut restarted,
            Rf5Processor::prepare_program_save,
            &serde_json::to_vec(&older).unwrap(),
        )
        .unwrap();
        assert!(restarted.install_program(&prepared));
    }

    fn factory_count() -> usize {
        let catalog: Value = serde_json::from_str(FACTORY_CATALOG).unwrap();
        catalog["presets"].as_array().unwrap().len()
    }

    #[test]
    fn the_library_holds_its_limit_and_the_catalog_still_fits() {
        let mut processor = Rf5Processor::default();
        for number in 0..MAX_PROGRAMS {
            let draft = begin(&mut processor, None).unwrap();
            let name = format!("{number:02} {}", "x".repeat(MAX_NAME_CHARS - 3));
            save(&mut processor, draft.document, &name);
        }
        assert_eq!(processor.programs.len(), MAX_PROGRAMS);
        assert!(
            begin(&mut processor, None).is_none(),
            "a full library starts no new program"
        );
        let listed = catalog(&mut processor);
        assert_eq!(
            listed["presets"].as_array().unwrap().len(),
            factory_count() + MAX_PROGRAMS
        );
        // Saving over one still works when full.
        let again = begin(&mut processor, Some("custom.user-64")).unwrap();
        save(&mut processor, again.document, "Last");
    }

    #[test]
    fn foreign_or_damaged_documents_are_refused() {
        let mut processor = Rf5Processor::default();
        let draft = begin(&mut processor, None).unwrap();
        let refused = |processor: &mut Rf5Processor, document: &ProgramDocument| {
            call(
                processor,
                Rf5Processor::prepare_program_save,
                &serde_json::to_vec(document).unwrap(),
            )
            .is_none()
        };
        let mut foreign = draft.document.clone();
        foreign.plugin_id = "org.rackforge.rf-7".to_owned();
        assert!(refused(&mut processor, &foreign));
        let mut short = draft.document.clone();
        short.payload = json!({ "memory": "00ff" });
        assert!(refused(&mut processor, &short));
        let mut unnamed = draft.document.clone();
        unnamed.name = "   ".to_owned();
        assert!(refused(&mut processor, &unnamed));
        // A prepared program that is not exactly the RF-5's own envelope.
        let mut tampered = draft.clone();
        tampered.storage_path = "elsewhere.json".to_owned();
        assert!(!processor.install_program(&serde_json::to_vec(&tampered).unwrap()));
    }

    #[test]
    fn a_draft_previews_and_has_an_editor_view() {
        let mut processor = Rf5Processor::default();
        let copy = begin(&mut processor, Some("original-12-low-strings")).unwrap();
        assert!(processor.preview_program(&serde_json::to_vec(&copy).unwrap()));
        assert_eq!(
            Some(processor.engine.program_memory()),
            memory(&copy.document)
        );
        let view = call(
            &mut processor,
            Rf5Processor::program_editor_view,
            &serde_json::to_vec(&copy.document).unwrap(),
        )
        .expect("an editor view");
        let view: ProgramEditorView = serde_json::from_slice(&view).unwrap();
        assert!(view.validate().is_ok());
        assert_eq!(
            processor.program_editing_capabilities(),
            rackforge_plugin_sdk::PROGRAM_EDIT_KNOWN_CAPABILITIES
        );
    }
}
