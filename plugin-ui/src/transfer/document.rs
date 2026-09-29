//! RackForge's program documents: how RackForge keeps each of the RF-5's
//! own programs, and the file one is passed on in. The RF-5's payload is
//! `{ "memory": "<48 hex digits>" }`, the program's 24 bytes.
//!
//! One program travels as its document, a `.rackforge-program.json`; several
//! as a bundle of documents. RackForge envelopes (`{ "document": … }`) are
//! read too.

use serde_json::{Value, json};

use super::{Medium, Memory, PROGRAM_BYTES, Program, ReadError, Reading, clean_name, error};

pub const PLUGIN_ID: &str = "org.rackforge.rf-5";
/// A bundle of several programs' documents.
pub const BUNDLE_FORMAT: &str = "org.rackforge.rf-5.programs";
const BUNDLE_VERSION: u64 = 1;
/// The most documents a bundle is read for.
const MAX_DOCUMENTS: usize = 1_000;

pub fn to_hex(memory: &Memory) -> String {
    memory.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn from_hex(hex: &str) -> Option<Memory> {
    if hex.len() != PROGRAM_BYTES * 2 || !hex.is_ascii() {
        return None;
    }
    let mut memory = [0; PROGRAM_BYTES];
    for (byte, pair) in memory.iter_mut().zip(hex.as_bytes().as_chunks::<2>().0) {
        *byte = u8::from_str_radix(core::str::from_utf8(pair).ok()?, 16).ok()?;
    }
    Some(memory)
}

/// The name and memory of one of the RF-5's program documents.
pub fn program(document: &Value) -> Option<(String, Memory)> {
    let document = document.get("document").unwrap_or(document);
    if document.get("plugin_id")?.as_str()? != PLUGIN_ID {
        return None;
    }
    let memory = from_hex(document.get("payload")?.get("memory")?.as_str()?)?;
    let name = clean_name(document.get("name")?.as_str()?)?;
    Some((name, memory))
}

/// A draft's document carrying another program: its identity kept, as
/// RackForge requires of a draft, its name and memory replaced.
pub fn with_program(draft: &Value, name: &str, memory: &Memory) -> Option<Value> {
    let mut document = draft.clone();
    let object = document.as_object_mut()?;
    object.insert("name".to_owned(), Value::String(clean_name(name)?));
    object.insert("payload".to_owned(), json!({ "memory": to_hex(memory) }));
    Some(document)
}

/// Reads a program document, an envelope, or a bundle of documents.
pub fn read(bytes: &[u8]) -> Result<Reading, ReadError> {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return error("DAMAGED PROGRAM FILE");
    };
    let documents: Vec<&Value> = match value.get("programs").and_then(Value::as_array) {
        Some(documents) => documents.iter().take(MAX_DOCUMENTS).collect(),
        None => vec![&value],
    };
    let total = documents.len();
    let programs: Vec<Program> = documents
        .into_iter()
        .filter_map(program)
        .enumerate()
        .map(|(index, (name, memory))| Program {
            place: format!("{}", index + 1),
            name: Some(name),
            memory,
        })
        .collect();
    if programs.is_empty() {
        return error("NO RF-5 PROGRAMS IN THIS FILE");
    }
    let skipped = total - programs.len();
    Ok(Reading {
        medium: Medium::Document,
        warning: (skipped > 0).then(|| format!("{skipped} DOCUMENTS ARE NOT RF-5 PROGRAMS")),
        programs,
    })
}

/// One document as its own file; several as a bundle.
pub fn write(documents: &[Value]) -> Vec<u8> {
    let value = match documents {
        [document] => document.clone(),
        documents => json!({
            "format": BUNDLE_FORMAT,
            "version": BUNDLE_VERSION,
            "plugin_id": PLUGIN_ID,
            "programs": documents,
        }),
    };
    let mut bytes = serde_json::to_vec_pretty(&value).unwrap_or_default();
    bytes.push(b'\n');
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(id: &str, name: &str, memory: &Memory) -> Value {
        json!({
            "schema_version": 1,
            "id": id,
            "name": name,
            "plugin_id": PLUGIN_ID,
            "plugin_version": "0.1.18",
            "plugin_state_version": 14,
            "payload_version": 1,
            "tags": ["rf5", "user"],
            "payload": { "memory": to_hex(memory) },
        })
    }

    #[test]
    fn hex_is_the_engines() {
        let memory: Memory = core::array::from_fn(|index| (index * 11) as u8);
        assert_eq!(from_hex(&to_hex(&memory)), Some(memory));
        assert_eq!(from_hex("zz"), None);
        assert_eq!(from_hex(&"g".repeat(48)), None);
    }

    #[test]
    fn one_program_is_its_document_several_a_bundle() {
        let first = document("user-01", "Horns", &[1; PROGRAM_BYTES]);
        let second = document("user-02", "Bells", &[2; PROGRAM_BYTES]);
        let single = read(&write(std::slice::from_ref(&first))).unwrap();
        assert_eq!(single.programs.len(), 1);
        assert_eq!(single.programs[0].name.as_deref(), Some("Horns"));
        let bundle = write(&[first, second.clone(), json!({ "plugin_id": "other" })]);
        let reading = read(&bundle).unwrap();
        assert_eq!(reading.medium, Medium::Document);
        assert_eq!(reading.programs.len(), 2);
        assert_eq!(reading.programs[1].memory, [2; PROGRAM_BYTES]);
        assert_eq!(
            reading.warning.as_deref(),
            Some("1 DOCUMENTS ARE NOT RF-5 PROGRAMS")
        );
        let envelope = serde_json::to_vec(&json!({ "document": second })).unwrap();
        assert_eq!(
            read(&envelope).unwrap().programs[0].name.as_deref(),
            Some("Bells")
        );
    }

    #[test]
    fn a_draft_keeps_its_identity_and_takes_the_program() {
        let draft = document("user-07", "New program", &[0; PROGRAM_BYTES]);
        let replaced = with_program(&draft, " Tape 1-1 ", &[9; PROGRAM_BYTES]).unwrap();
        assert_eq!(replaced["id"], "user-07");
        assert_eq!(replaced["plugin_state_version"], 14);
        assert_eq!(
            program(&replaced),
            Some(("Tape 1-1".to_owned(), [9; PROGRAM_BYTES]))
        );
        assert_eq!(with_program(&draft, "  ", &[9; PROGRAM_BYTES]), None);
    }

    #[test]
    fn damaged_or_foreign_files_are_refused() {
        assert!(read(b"{ nope").is_err());
        assert!(read(br#"{"plugin_id":"org.rackforge.rf-dls","name":"x","payload":{}}"#).is_err());
    }
}
