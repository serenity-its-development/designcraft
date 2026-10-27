//! One Layout editor window's host side for node: the engine the browser build runs, so OneOS's `craft-layout-ai-drive.test.ts` can
//! drive a window through the SAME `command.batch` the browser build answers `craft:command` with
//! (`designcraft_engine::host_commands`), and read back the `.designcraft` document or IDML package the editor saves.

use designcraft_engine::Session;
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

fn err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

#[wasm_bindgen]
pub struct Window {
    session: Session,
}

#[wasm_bindgen]
impl Window {
    /// A window with a new document of `pages` pages.
    #[wasm_bindgen(constructor)]
    pub fn new(pages: u32) -> Result<Window, JsError> {
        let mut session = Session::new();
        session.execute("file.new", &json!({ "pages": pages.max(1) })).map_err(err)?;
        Ok(Window { session })
    }

    /// Open a document the host sent (`craft:open`): the editor's own document or an IDML package, base64.
    pub fn open_b64(&mut self, name: &str, b64: &str) -> Result<(), JsError> {
        self.session.execute("file.openBytes", &json!({ "name": name, "base64": b64 })).map(|_| ()).map_err(err)
    }

    /// The assistant's run (`craft:command`, `command.batch`): the answer as JSON text, or an error `<code>: <reason>`.
    pub fn command_batch(&mut self, params_json: &str) -> Result<String, JsError> {
        let params: Value = serde_json::from_str(params_json).map_err(err)?;
        self.session.run_command_batch(&params).map(|v| v.to_string()).map_err(|e| JsError::new(&e))
    }

    /// The document as the editor saves it: `designcraft` (its own document) or `idml` (an IDML package); base64.
    pub fn file_b64(&mut self, kind: &str) -> Result<String, JsError> {
        let command = match kind {
            "designcraft" => "file.serialize",
            "idml" => "file.exportIdml",
            other => return Err(JsError::new(&format!("no such file kind '{other}'"))),
        };
        let r = self.session.execute(command, &json!({})).map_err(err)?;
        r["base64"].as_str().map(str::to_owned).ok_or_else(|| JsError::new("the save made no data"))
    }

    /// How many undo steps the open document holds.
    pub fn undo_depth(&self) -> usize {
        self.session.active().map(|d| d.history.undo.len()).unwrap_or(0)
    }

    /// Undo one step (the person's Undo).
    pub fn undo(&mut self) -> Result<(), JsError> {
        self.session.execute("edit.undo", &json!({})).map(|_| ()).map_err(err)
    }

    /// Whether the document has unsaved changes.
    pub fn dirty(&self) -> bool {
        self.session.active().is_some_and(|d| d.is_dirty())
    }
}
