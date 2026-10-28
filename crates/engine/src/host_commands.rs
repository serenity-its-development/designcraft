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

/// Registry commands the assistant may run: EVERY command of the registry except those in [`LEFT_OUT`], each with its reason.
/// The document's whole command registry (frames, text, tables, styles, swatches, pages, parents, layers, effects, links, data merge). A step the document's state forbids, or one that needs a dialog or a pointer, is refused by the registry exactly as it would be for
/// the person, and the run is taken back whole.
pub const ASSISTANT_COMMANDS: &[&str] = &[
    // anchored
    "anchored.insert",
    "anchored.options",
    "anchored.release",
    "anchored.list",
    // path
    "path.addAnchor",
    "path.deleteAnchor",
    "path.convertAnchor",
    "path.erase",
    "path.smooth",
    "path.join",
    "path.convertPoints",
    // article
    "article.list",
    "article.new",
    "article.add",
    "article.remove",
    "article.options",
    "article.delete",
    // button
    "button.set",
    "button.clear",
    // form
    "form.set",
    "form.clear",
    "form.list",
    // button
    "button.list",
    // object
    "object.caption",
    // changes
    "changes.track",
    "changes.list",
    "changes.accept",
    "changes.reject",
    "changes.acceptAll",
    "changes.rejectAll",
    // color
    "color.extractTheme",
    // swatch
    "swatch.addColorTheme",
    // color
    "color.convert",
    // condition
    "condition.list",
    "condition.new",
    "condition.options",
    "condition.delete",
    "condition.apply",
    // data
    "data.source.select",
    "data.source.update",
    "data.source.remove",
    "data.placeholder.add",
    "data.placeholder.remove",
    "data.options",
    "data.preview",
    "data.preview.stop",
    "data.merge",
    // selection
    "selection.set",
    "selection.toggle",
    "selection.key",
    // edit
    "edit.selectAll",
    "edit.deselectAll",
    "edit.clear",
    "edit.copy",
    "edit.cut",
    "edit.pasteWithoutFormatting",
    "edit.paste",
    "edit.pasteInto",
    "edit.pasteInPlace",
    "edit.duplicate",
    "edit.commitPreview",
    "edit.cancelPreview",
    // note
    "note.new",
    "note.convertFromText",
    "note.edit",
    "note.delete",
    "note.convertToText",
    "note.list",
    // endnote
    "endnote.insert",
    "endnote.edit",
    "endnote.delete",
    "endnote.list",
    "endnote.options",
    // find
    "find.objects",
    "find.changeObjects",
    "find.find",
    "find.change",
    "find.next",
    // flex
    "flex.add",
    "flex.convert",
    "flex.remove",
    "flex.set",
    "flex.reorder",
    // font
    "font.list",
    "font.replace",
    // guide
    "guide.add",
    "guide.move",
    "guide.moveBy",
    "guide.delete",
    "guide.deleteAll",
    // view
    "view.lockColumnGuides",
    // guide
    "guide.deselect",
    // layout
    "layout.createGuides",
    // index
    "index.addReference",
    "index.list",
    "index.generate",
    "index.update",
    // document
    "document.inspect",
    "document.list",
    // tool
    "tool.select",
    "tool.list",
    // document
    "document.history",
    // hyperlink
    "hyperlink.create",
    "hyperlink.delete",
    "hyperlink.list",
    "hyperlink.edit",
    "hyperlink.goToSource",
    // bookmark
    "bookmark.rename",
    "bookmark.add",
    "bookmark.delete",
    "bookmark.list",
    // layout
    "layout.pages.insert",
    "layout.rotateSpreadView",
    "layout.pages.delete",
    "layout.pages.move",
    "layout.pages.duplicateSpread",
    "layout.pages.applyParent",
    "layout.parents.new",
    "layout.parents.delete",
    "layout.parents.options",
    "layout.marginsAndColumns",
    "layout.moveColumnGuide",
    "layout.documentSetup",
    "layout.pages.toSpread",
    "layout.spreadShuffle",
    "layout.pageSize",
    "layout.pageOptions",
    "layout.section",
    "layout.chapterNumbering",
    "layout.numberingOptions",
    // layer
    "layer.move",
    "layer.selectItems",
    "layer.new",
    "layer.set",
    "layer.delete",
    "layer.merge",
    "layer.deleteUnused",
    "layer.others",
    "layer.activate",
    // conveyor
    "conveyor.collect",
    "conveyor.place",
    "conveyor.list",
    "conveyor.clear",
    // story
    "story.placeAndLink",
    "story.links",
    "story.updateLink",
    "story.unlink",
    // links
    "links.list",
    "links.update",
    "links.embed",
    "links.goTo",
    // liquid
    "liquid.pageRule",
    "liquid.object",
    // guide
    "guide.liquid",
    // layout
    "layout.createAlternate",
    // list
    "list.bullets",
    "list.numbers",
    "list.define",
    "list.delete",
    // style
    "style.load.list",
    // math
    "math.insert",
    "math.svg",
    // media
    "media.options",
    "media.get",
    // footnote
    "footnote.insert",
    "footnote.options",
    "footnote.list",
    "footnote.setText",
    "footnote.delete",
    // frame
    "frame.create",
    // line
    "line.create",
    // transform
    "transform.move",
    "transform.resize",
    "transform.rotate",
    "transform.scale",
    "transform.again",
    "transform.clear",
    "transform.shear",
    // object
    "object.matchAttributes",
    // transform
    "transform.flip",
    "transform.set",
    "transform.info",
    // object
    "object.arrange",
    "object.bringToFront",
    "object.bringForward",
    "object.sendBackward",
    "object.sendToBack",
    "object.group",
    "object.ungroup",
    "object.lock",
    "object.unlockAll",
    "object.hide",
    "object.showAll",
    "object.setFlags",
    "object.fill",
    "object.swapFillStroke",
    "object.defaultFillStroke",
    "object.stroke",
    "object.opacity",
    "object.dropShadow",
    "object.innerShadow",
    "object.outerGlow",
    "object.exportOptions",
    "object.innerGlow",
    "object.bevel",
    "object.satin",
    "object.directionalFeather",
    "object.effects",
    "object.feather",
    "object.content",
    "object.textFrameOptions",
    "object.cornerOptions",
    "object.textWrap",
    "object.fit",
    "object.fittingOptions",
    "object.rename",
    // edit
    "edit.stepAndRepeat",
    // path
    "path.create",
    "path.appendAnchor",
    "path.setAnchor",
    "path.reverse",
    "path.close",
    "path.moveAnchors",
    // object
    "object.align",
    "object.distribute",
    // gap
    "gap.move",
    // frame
    "frame.grid",
    // object
    "object.clippingPath",
    "object.clippingPathOptions",
    "object.primaryTextFrame",
    "object.attributes",
    "object.globalLight",
    "object.label",
    "object.altText",
    "object.transparencyGroup",
    "object.findByLabel",
    "object.setLayer",
    "object.reorder",
    // layout
    "layout.overrideParentItems",
    "layout.removeOverrides",
    "layout.detachAll",
    // edit
    "edit.pasteSvg",
    // type
    "type.onPath",
    "type.pathOptions",
    // object
    "object.makeCompoundPath",
    "object.releaseCompoundPath",
    // type
    "type.createOutlines",
    // object
    "object.pathfinder",
    "object.gradientFeather",
    // path
    "path.split",
    // object
    "object.pdfLayers",
    "object.layerOptions",
    // preflight
    "preflight.run",
    // edit
    "edit.transparencyBlendSpace",
    // document
    "document.preferences",
    // object
    "object.qrCode",
    // select
    "select.firstAbove",
    "select.nextAbove",
    "select.nextBelow",
    "select.lastBelow",
    "select.container",
    "select.content",
    "select.nextInGroup",
    "select.previousInGroup",
    // object
    "object.convertShape",
    // spelling
    "spelling.check",
    // hyphenation
    "hyphenation.addException",
    "hyphenation.removeException",
    "hyphenation.list",
    // spelling
    "spelling.setWords",
    "spelling.words",
    "spelling.addWord",
    "spelling.change",
    // states
    "states.create",
    "states.list",
    "states.show",
    "states.rename",
    "states.release",
    // strokeStyle
    "strokeStyle.list",
    "strokeStyle.new",
    "strokeStyle.delete",
    // style
    "style.compositeFont.list",
    "style.compositeFont.set",
    "style.exportTag",
    "style.paragraph.apply",
    "style.character.apply",
    "style.paragraph.redefine",
    "style.character.redefine",
    "style.paragraph.create",
    "style.paragraph.edit",
    "style.paragraph.delete",
    "style.character.create",
    "style.character.edit",
    "style.character.delete",
    "style.list",
    "style.object.apply",
    "style.object.create",
    "style.object.edit",
    "style.object.clearOverrides",
    // swatch
    "swatch.create",
    "swatch.new",
    // object
    "object.gradient",
    // style
    "style.group",
    // object
    "object.color",
    // color
    "color.sample",
    // swatch
    "swatch.addToSwatches",
    "swatch.options",
    "swatch.addUnnamed",
    "swatch.delete",
    // ink
    "ink.list",
    "ink.options",
    // swatch
    "swatch.newColorGroup",
    "swatch.moveToGroup",
    "swatch.ungroupColorGroup",
    "swatch.renameColorGroup",
    // style
    "style.cell.create",
    "style.cell.apply",
    "style.cell.edit",
    "style.table.create",
    "style.table.apply",
    "style.table.edit",
    // table
    "table.insert",
    "table.convertFromText",
    "table.convertToText",
    "table.insertRow",
    "table.insertRowAbove",
    "table.insertRowBelow",
    "table.insertColumn",
    "table.insertColumnLeft",
    "table.insertColumnRight",
    "table.deleteRow",
    "table.deleteColumn",
    "table.delete",
    "table.merge",
    "table.dropCells",
    "table.placeGraphic",
    "table.textCell",
    "table.moveRow",
    "table.moveColumn",
    "table.sortRows",
    "table.unmerge",
    "table.splitHorizontally",
    "table.splitVertically",
    "table.setCell",
    "table.setRowHeight",
    "table.setColumnWidth",
    "table.resizeBorder",
    "table.distributeColumns",
    "table.distributeRows",
    "table.options",
    "table.convertRowsToHeader",
    "table.convertRowsToBody",
    "table.convertRowsToFooter",
    "table.select",
    "table.selectTable",
    "table.selectRow",
    "table.selectColumn",
    "table.nextCell",
    "table.prevCell",
    "table.get",
    // type
    "type.tabs.add",
    "type.tabs.move",
    "type.tabs.change",
    "type.tabs.remove",
    "type.tabs.clear",
    "type.tabs.repeat",
    // text
    "text.placeCaret",
    "text.release",
    "text.selectWord",
    // type
    "type.pickUpFormat",
    // text
    "text.selectSentence",
    "text.selectParagraph",
    "text.selectStory",
    "text.select",
    "text.insert",
    "text.delete",
    "text.move",
    "text.exitToFrame",
    // story
    "story.setDirection",
    "story.options",
    "story.setText",
    "story.replaceRange",
    "story.get",
    // type
    "type.fillWithPlaceholder",
    "type.char",
    "type.kernPair",
    "type.para",
    "type.align",
    "type.alignLeft",
    "type.alignCenter",
    "type.alignRight",
    "type.justify",
    "type.bold",
    "type.italic",
    "type.swapFillStroke",
    "type.underline",
    "type.allCaps",
    "type.tateChuYoko",
    "type.ruby",
    "type.kenten",
    "type.shatai",
    "type.autoTateChuYoko",
    "type.gridSettings",
    "type.rotateRoman",
    "type.romanWordBreak",
    "type.warichu",
    "type.storyDirection",
    "type.kernStep",
    "type.sizeUp",
    "type.sizeDown",
    "type.changeCase",
    "type.openType",
    // style
    "style.breakLink",
    // type
    "type.openTypeFeatures",
    "type.feature",
    // text
    "text.thread",
    "text.unthread",
    "text.removeFromThread",
    // toc
    "toc.generate",
    "toc.update",
    "toc.entries",
    // page
    "page.transition",
    "page.transitions",
    // variables
    "variables.list",
    "variables.define",
    "variables.delete",
    "variables.insert",
    "variables.convertToText",
    "variables.convertAllToText",
    // xml
    "xml.tags",
    "xml.newTag",
    "xml.deleteTag",
    "xml.tagText",
    "xml.tag",
    "xml.mapStyle",
    "xml.structure",
    "xml.deleteDtd",
    // xref
    "xref.insert",
    // anchor
    "anchor.create",
    // xref
    "xref.list",
    // anchor
    "anchor.list",
    // xref
    "xref.formats",
    "xref.defineFormat",
    "xref.setFormat",
    // color
    "color.settings",
    // data
    "data.fields",
    // edit
    "edit.preview",
    // font
    "font.reload",
    // footnote
    "footnote.goToReference",
    // guide
    "guide.list",
    "guide.select",
    "guide.selectAll",
    // layout
    "layout.parents.edit",
    "layout.selectPage",
    // note
    "note.next",
    "note.previous",
    // prefs
    "prefs.set",
    // table
    "table.getCellStroke",
    // text
    "text.extendTo",
    "text.selectAt",
    // tool
    "tool.polygonSettings",
    // type
    "type.selectionAttrs",
    "type.tabs.get",
    // view
    "view.lockZeroPoint",
    "view.zeroPoint",
    // xml
    "xml.validate",
];

/// Commands the window itself provides.
pub const ARGUMENT_COMMANDS: &[&str] = &[];

/// Registry commands left OUT of the allow-list, grouped by the reason (the audit lists them; every registry command is in exactly one of the two).
pub const LEFT_OUT: &[(&str, &[&str])] = &[
    (
        "files: opens, saves, exports, places or prints a file (files move only through the host)",
        &["file.exportPdf", "file.exportEpub", "document.epubSplit", "file.exportFixedEpub", "file.printBooklet", "file.exportHtml", "file.exportPptx", "file.exportText", "file.new", "file.newSample", "file.presets", "file.savePreset", "file.deletePreset", "file.open", "file.openBytes", "file.openRecent", "file.recentFiles", "file.clearRecent", "file.save", "file.saveAs", "file.saveACopy", "file.revert", "file.recovery.list", "file.recovery.save", "file.recovery.open", "file.serialize", "file.close", "file.activate", "place.styles", "place.load", "place.drop", "file.place", "file.exportIdml", "file.openIdml", "file.printers", "file.exportXml", "file.importXml", "file.package", "file.print", "swatch.save"],
    ),
    (
        "history: Undo and Redo are the person's",
        &["edit.undo", "edit.redo"],
    ),
    (
        "reads or writes a file on disk by path: scripts, snippets, Object Library and book files, relinking, colour profiles, swatch and style files, DTDs",
        &["book.new", "book.open", "book.close", "book.add", "book.remove", "book.styleSource", "book.list", "book.paginate", "book.syncStyles", "book.exportPdf", "color.loadProfile", "snippet.export", "snippet.place", "library.new", "library.open", "library.list", "library.json", "library.add", "library.place", "library.remove", "library.close", "links.relink", "links.relinkFolder", "style.load", "links.copyTo", "script.run", "swatch.load", "xml.loadDtd"],
    ),
    (
        "application commands (links, quit): not an edit of the document",
        &["app.links"],
    ),
];

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
