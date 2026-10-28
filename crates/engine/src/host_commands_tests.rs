use serde_json::{Value, json};

use crate::cmd::find_command;
use crate::host_commands::{ARGUMENT_COMMANDS, ASSISTANT_COMMANDS, LEFT_OUT, allowed};
use crate::Session;

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"pages": 2})).unwrap();
    s
}

fn items(s: &Session) -> usize {
    s.doc().unwrap().doc.spreads.iter().map(|sp| sp.items.len()).sum()
}

fn batch(s: &mut Session, label: &str, steps: Value) -> Result<Value, String> {
    s.run_command_batch(&json!({ "label": label, "steps": steps }))
}

#[test]
fn every_listed_command_is_a_registry_command_and_none_reaches_a_file_or_history() {
    for id in ASSISTANT_COMMANDS {
        assert!(find_command(id).is_some(), "{id} is not a registry command");
        assert!(!id.starts_with("file.") && !id.starts_with("edit.undo") && !id.starts_with("edit.redo"), "{id}");
    }
    assert!(ARGUMENT_COMMANDS.is_empty());
    let mut seen = std::collections::BTreeSet::new();
    for id in allowed() {
        assert!(seen.insert(id), "{id} is listed twice");
    }
}

#[test]
fn a_run_is_one_undo_step_named_by_its_label() {
    let mut s = session();
    let before = s.doc().unwrap().history.undo.len();
    let r = batch(
        &mut s,
        "Lay out a cover",
        json!([
            { "name": "frame.create", "args": { "rect": [36, 36, 300, 120], "content": "text", "text": "Annual report" } },
            { "name": "frame.create", "args": { "rect": [36, 140, 300, 400], "shape": "ellipse" } },
            { "name": "line.create", "args": { "a": [36, 130], "b": [300, 130] } },
        ]),
    )
    .unwrap();
    assert_eq!(r["steps"].as_array().unwrap().len(), 3);
    assert_eq!(r["undoStep"], true);
    assert_eq!(items(&s), 3);
    assert_eq!(s.doc().unwrap().history.undo.len(), before + 1, "the three steps are one");
    assert_eq!(s.doc().unwrap().history.undo.last().unwrap().label, "Lay out a cover");
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(items(&s), 0, "one undo takes the whole run back");
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(items(&s), 3);
}

#[test]
fn a_failing_step_takes_back_the_steps_before_it() {
    let mut s = session();
    s.execute("frame.create", &json!({"rect": [10, 10, 50, 50]})).unwrap();
    let before = s.doc().unwrap().history.undo.len();
    let e = batch(
        &mut s,
        "Half done",
        json!([
            { "name": "frame.create", "args": { "rect": [60, 10, 100, 50] } },
            { "name": "layer.set", "args": { "id": 9999, "name": "No such layer" } },
        ]),
    )
    .unwrap_err();
    assert!(e.starts_with("step-failed: step 2"), "{e}");
    assert_eq!(items(&s), 1, "the second frame is gone again");
    assert_eq!(s.doc().unwrap().history.undo.len(), before);
}

#[test]
fn what_is_not_listed_is_refused_whole_and_nothing_runs() {
    let mut s = session();
    for (steps, code) in [
        (json!([{ "name": "frame.create", "args": { "rect": [0, 0, 10, 10] } }, { "name": "file.saveAs", "args": {} }]), "command-not-allowed"),
        (json!([{ "name": "edit.undo" }]), "command-not-allowed"),
        (json!([{ "name": "file.place", "args": {} }]), "command-not-allowed"),
        (json!([{ "name": "frame.create", "args": { "rect": [0, 0, 10, 10], "path": "/etc/passwd" } }]), "argument-not-allowed"),
        (json!([{ "name": "frame.create", "args": { "rect": [0, 0, 10, 10], "url": "https://x.example" } }]), "argument-not-allowed"),
        (json!([]), "run-empty"),
    ] {
        let e = batch(&mut s, "x", steps).unwrap_err();
        assert!(e.starts_with(code), "{e}");
        assert_eq!(items(&s), 0, "{e}");
    }
    assert!(batch(&mut s, &"l".repeat(121), json!([{ "name": "document.inspect" }])).unwrap_err().starts_with("label-too-long"));
    let many: Vec<Value> = (0..51).map(|_| json!({ "name": "document.inspect" })).collect();
    assert!(batch(&mut s, "x", Value::Array(many)).unwrap_err().starts_with("run-too-long"));
}

#[test]
fn a_read_only_run_leaves_no_undo_step_and_reads_back_what_was_made() {
    let mut s = session();
    batch(&mut s, "Draw", json!([{ "name": "frame.create", "args": { "rect": [36, 36, 300, 120], "content": "text", "text": "Hello" } }])).unwrap();
    let depth = s.doc().unwrap().history.undo.len();
    let r = batch(&mut s, "Look", json!([{ "name": "document.inspect", "args": {} }])).unwrap();
    assert_eq!(r["undoStep"], false);
    assert_eq!(s.doc().unwrap().history.undo.len(), depth);
    assert_eq!(r["steps"][0]["result"]["stories"][0]["preview"], "Hello");
}

#[test]
fn a_run_with_no_document_is_refused_by_name() {
    let mut s = Session::new();
    assert!(batch(&mut s, "x", json!([{ "name": "document.inspect" }])).unwrap_err().starts_with("no-document"));
}

#[test]
fn every_registry_command_is_allowed_or_left_out_with_a_reason() {
    let registry: std::collections::BTreeSet<&str> = crate::cmd::command_specs().iter().map(|s| s.id).collect();
    let allowed: std::collections::BTreeSet<&str> = ASSISTANT_COMMANDS.iter().copied().collect();
    let left: Vec<&str> = LEFT_OUT.iter().flat_map(|(_, ids)| ids.iter().copied()).collect();
    for (reason, ids) in LEFT_OUT {
        assert!(reason.len() > 20, "a command left out states why: {ids:?}");
    }
    let left_set: std::collections::BTreeSet<&str> = left.iter().copied().collect();
    assert_eq!(left.len(), left_set.len(), "a command is left out under two reasons");
    assert!(allowed.is_disjoint(&left_set), "allowed and left out overlap: {:?}", allowed.intersection(&left_set).collect::<Vec<_>>());
    let accounted: std::collections::BTreeSet<&str> = allowed.union(&left_set).copied().collect();
    let missing: Vec<&&str> = registry.difference(&accounted).collect();
    let unknown: Vec<&&str> = accounted.difference(&registry).collect();
    assert!(missing.is_empty(), "registry commands neither allowed nor left out: {missing:?}");
    assert!(unknown.is_empty(), "listed commands the registry does not have: {unknown:?}");
    assert_eq!(registry.len(), allowed.len() + left_set.len());
}
