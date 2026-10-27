//! The assistant's command surface for the OneOS host bridge (`craft:command {method: "command.batch"}`).
//!
//! The host relays an allow-listed run: `{label, steps: [{name, args?}]}`. The whole run is judged BEFORE anything runs (an id
//! outside the list, or an argument that names a file or an address, refuses the run), every step runs through the same registry
//! the menus, the control channel and the MCP server use (so a step the document's state forbids is refused exactly as it would be
//! for the person), and the run is ONE undo step named by `label`: a step that fails takes back every step before it, so a run is
//! all or nothing. There is no file-path, dialog or undo command in the list: files move only through the host.
//!
//! This list is the one declared in the host's `platform-craft` registry as `layout` commands (pinned equal there).

use serde_json::{Value, json};

use crate::Session;

/// Registry commands the assistant may run. Each acts at once on the document (no dialog, no file picker, no pointer gesture), and
/// none is Undo or Redo: undoing a run is the person's.
pub const ASSISTANT_COMMANDS: &[&str] = &[
    // Make things.
    "frame.create",
    "line.create",
    "frame.grid",
    "table.insert",
    "text.insert",
    "story.setText",
    "story.replaceRange",
    "guide.add",
    "guide.move",
    "guide.moveBy",
    "guide.delete",
    "guide.deleteAll",
    // Pages, spreads, parents and layers.
    "layout.pages.insert",
    "layout.pages.delete",
    "layout.pages.move",
    "layout.pages.duplicateSpread",
    "layout.pages.applyParent",
    "layout.parents.new",
    "layout.pageSize",
    "layout.marginsAndColumns",
    "layout.createGuides",
    "layer.new",
    "layer.set",
    "layer.delete",
    "layer.move",
    // Structure: order, groups, lock and visibility.
    "object.group",
    "object.ungroup",
    "object.arrange",
    "object.bringToFront",
    "object.bringForward",
    "object.sendBackward",
    "object.sendToBack",
    "object.lock",
    "object.unlockAll",
    "object.hide",
    "object.showAll",
    "object.setLayer",
    "object.reorder",
    "object.rename",
    "object.align",
    "object.distribute",
    "edit.duplicate",
    "edit.clear",
    "edit.stepAndRepeat",
    // Place and size.
    "transform.move",
    "transform.resize",
    "transform.rotate",
    "transform.scale",
    "transform.shear",
    "transform.flip",
    "transform.set",
    // Properties: paint, effects, frames and their content.
    "object.fill",
    "object.stroke",
    "object.swapFillStroke",
    "object.opacity",
    "object.dropShadow",
    "object.innerShadow",
    "object.outerGlow",
    "object.innerGlow",
    "object.feather",
    "object.gradient",
    "object.color",
    "object.content",
    "object.fit",
    "object.fittingOptions",
    "object.textFrameOptions",
    "object.cornerOptions",
    "object.textWrap",
    "object.convertShape",
    "object.altText",
    "object.label",
    // Text.
    "type.char",
    "type.para",
    "type.align",
    "type.alignLeft",
    "type.alignCenter",
    "type.alignRight",
    "type.justify",
    "type.bold",
    "type.italic",
    "type.underline",
    "type.allCaps",
    "type.changeCase",
    "type.sizeUp",
    "type.sizeDown",
    "text.thread",
    "text.unthread",
    "story.options",
    // Tables.
    "table.insertRow",
    "table.insertRowAbove",
    "table.insertRowBelow",
    "table.insertColumn",
    "table.insertColumnLeft",
    "table.insertColumnRight",
    "table.deleteRow",
    "table.deleteColumn",
    "table.merge",
    "table.unmerge",
    "table.setCell",
    "table.setRowHeight",
    "table.setColumnWidth",
    "table.distributeRows",
    "table.distributeColumns",
    // Styles and swatches.
    "style.paragraph.create",
    "style.paragraph.edit",
    "style.paragraph.apply",
    "style.paragraph.delete",
    "style.character.create",
    "style.character.edit",
    "style.character.apply",
    "style.character.delete",
    "style.object.create",
    "style.object.edit",
    "style.object.apply",
    "style.object.clearOverrides",
    "style.cell.create",
    "style.cell.edit",
    "style.cell.apply",
    "style.table.create",
    "style.table.edit",
    "style.table.apply",
    "swatch.create",
    "swatch.delete",
    "swatch.options",
    // Choosing what the next step acts on.
    "selection.set",
    "selection.toggle",
    // Read-only.
    "document.inspect",
    "find.objects",
];

/// Commands the window itself provides.
pub const ARGUMENT_COMMANDS: &[&str] = &[];

/// Every command id a run may name, in the order the host's registry declares them.
pub fn allowed() -> Vec<&'static str> {
    ASSISTANT_COMMANDS.iter().chain(ARGUMENT_COMMANDS).copied().collect()
}

const MAX_STEPS: usize = 50;
const MAX_LABEL: usize = 120;
/// Argument names that would point a command at a file or a network address: refused anywhere in a step's arguments.
const FORBIDDEN_ARGS: &[&str] = &["path", "dir", "folder", "file", "url", "uri", "href", "server", "data", "base64", "src", "link"];

/// The first forbidden argument name anywhere in `v`.
fn forbidden_arg(v: &Value) -> Option<String> {
    match v {
        Value::Object(o) => o.iter().find_map(|(k, x)| if FORBIDDEN_ARGS.contains(&k.as_str()) { Some(k.clone()) } else { forbidden_arg(x) }),
        Value::Array(a) => a.iter().find_map(forbidden_arg),
        _ => None,
    }
}

impl Session {
    /// One `command.batch`: all of it, as ONE undo step, or none of it.
    pub fn run_command_batch(&mut self, params: &Value) -> Result<Value, String> {
        let label = params.get("label").and_then(Value::as_str).map(str::trim).unwrap_or("");
        if label.chars().count() > MAX_LABEL {
            return Err(format!("label-too-long: a run's label is at most {MAX_LABEL} characters"));
        }
        let steps = params.get("steps").and_then(Value::as_array).ok_or("run-empty: a run states its steps")?;
        if steps.is_empty() {
            return Err("run-empty: a run states at least one command".into());
        }
        if steps.len() > MAX_STEPS {
            return Err(format!("run-too-long: a run is at most {MAX_STEPS} steps"));
        }
        // Judge the whole run first; nothing has run when one id is outside the list.
        let mut named = Vec::with_capacity(steps.len());
        for step in steps {
            let name = step.get("name").and_then(Value::as_str).unwrap_or("");
            if !ASSISTANT_COMMANDS.contains(&name) && !ARGUMENT_COMMANDS.contains(&name) {
                return Err(format!("command-not-allowed: Layout does not allow the command '{name}' -- nothing was run"));
            }
            let args = step.get("args").cloned().unwrap_or_else(|| json!({}));
            if let Some(k) = forbidden_arg(&args) {
                return Err(format!(
                    "argument-not-allowed: step '{name}' names '{k}', and a run does not reach files or addresses -- nothing was run"
                ));
            }
            named.push((name, args));
        }
        let Some(index) = self.active else { return Err("no-document: no document is open in this window".into()) };
        if self.docs.get(index).is_some_and(|d| d.interaction.is_some()) {
            return Err("gesture-running: a drag is in progress in the window -- finish it first; nothing was run".into());
        }
        let label = if label.is_empty() { "Assistant" } else { label };

        // What to put back when a step fails: the whole state of the document (its documents are shared, so this is cheap) and the journal.
        let before = self.docs[index].clone();
        let depth = before.history.undo.len();
        let journal_len = self.journal.len();
        let limit = before.history.limit;
        // The steps inside are folded afterwards, so they must not push the oldest history out.
        self.docs[index].history.limit = usize::MAX;
        let mut results: Vec<Value> = Vec::with_capacity(named.len());
        let mut failure: Option<String> = None;
        for (i, (name, args)) in named.iter().enumerate() {
            match self.execute(name, args) {
                Ok(v) => results.push(json!({ "name": name, "result": v })),
                Err(e) => {
                    failure = Some(format!("step-failed: step {} ({name}): {e} -- nothing was changed", i + 1));
                    break;
                }
            }
        }
        // A step may have closed or switched documents; the run works on the one it began on.
        let Some(at) = self.docs.iter().position(|d| d.uid == before.uid) else {
            return Err("document-gone: the document closed during the run".into());
        };
        if let Some(why) = failure {
            self.docs[at] = before;
            self.journal.truncate(journal_len);
            self.active = Some(at);
            return Err(why);
        }
        let st = &mut self.docs[at];
        st.history.limit = limit;
        let mut folded = false;
        if st.history.undo.len() > depth {
            // The first step of the run holds the document as it was before the run: the folded step goes back to exactly there.
            st.history.undo.truncate(depth + 1);
            if let Some(first) = st.history.undo.get_mut(depth) {
                first.label = label.to_string();
            }
            while st.history.undo.len() > limit.max(1) {
                st.history.undo.remove(0);
            }
            folded = true;
        }
        Ok(json!({ "label": label, "steps": results, "undoStep": folded }))
    }
}
