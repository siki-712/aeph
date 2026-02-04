use anyhow::Result;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use super::document::Document;
use crate::storage;
use crate::storage::Config;
use crate::ui::{GridStyle, TextAlign};

const MAX_DOCUMENTS: usize = 9;

pub struct AppState {
    pub documents: Vec<Document>,
    pub current_doc: usize,
    pub config: Config,
    pub show_help: bool,
    pub help_page: usize,
    pub show_file_picker: bool,
    pub show_logo: bool,
    pub picker_selection: usize,
    pub viewport_height: usize,
    pub viewport_width: u16,
    pub goto_input: Option<String>,
    pub notification: Option<String>,
    pub leader_buffer: Option<(String, Instant)>,
    pub grid_style: GridStyle,
    pub text_align: TextAlign,
    pub quote_seed: u64,
}

impl AppState {
    pub fn new(file: Option<PathBuf>) -> Result<Self> {
        let mut documents = Vec::new();

        // Create default documents (paper1.md ~ paper9.md) in data directory
        for i in 1..=MAX_DOCUMENTS {
            let path = storage::get_paper_path(i)?;
            documents.push(Document::new(Some(path))?);
        }

        // Load session data
        let session = storage::load_session();

        // Determine starting document
        let current_doc = if let Some(path) = file {
            // If a specific file was provided, replace the first document
            documents[0] = Document::new(Some(path))?;
            0
        } else {
            // Restore last opened document
            session.last_doc.min(MAX_DOCUMENTS - 1)
        };

        // Show logo on first launch (all documents empty)
        let all_empty = documents.iter().all(|d| d.content.is_empty());

        // Load configuration
        let config = storage::load_config();

        // Generate random seed from current time
        let quote_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let text_align = TextAlign::from_u8(session.text_align);

        // wrap_width will be set by update_wrap_width() after first render
        // when viewport_width is known

        Ok(Self {
            documents,
            current_doc,
            config,
            show_help: false,
            help_page: 0,
            show_file_picker: false,
            show_logo: all_empty,
            picker_selection: 0,
            viewport_height: 24,
            viewport_width: 80,
            goto_input: None,
            notification: None,
            leader_buffer: None,
            grid_style: GridStyle::from_u8(session.grid_style),
            text_align,
            quote_seed,
        })
    }

    pub fn doc(&self) -> &Document {
        &self.documents[self.current_doc]
    }

    pub fn doc_mut(&mut self) -> &mut Document {
        &mut self.documents[self.current_doc]
    }

    pub fn switch_to(&mut self, index: usize) {
        if index < self.documents.len() {
            self.current_doc = index;
        }
    }

    pub fn document_names(&self) -> Vec<(usize, &str, bool, String)> {
        self.documents
            .iter()
            .enumerate()
            .map(|(i, d)| (i, d.name(), d.modified, d.preview(30)))
            .collect()
    }

    // Delegate methods to current document
    pub fn save(&mut self) -> Result<()> {
        self.doc_mut().save()
    }

    pub fn has_modified(&self) -> bool {
        self.documents.iter().any(|d| d.modified)
    }

    pub fn save_all_modified(&mut self) -> Result<()> {
        for doc in &mut self.documents {
            if doc.modified {
                doc.save()?;
            }
        }
        Ok(())
    }

    pub fn format(&mut self) -> Result<()> {
        self.doc_mut().format()
    }

    /// Update wrap_width for all documents based on text_align and viewport_width.
    /// This ensures cursor movement matches the visual line wrapping in the editor.
    /// Must match the logic in editor.rs exactly.
    pub fn update_wrap_width(&mut self) {
        const PADDING_LEFT: u16 = 1;
        const PADDING_RIGHT: u16 = 1;
        const MAX_BODY_WIDTH: u16 = 80;

        let text_area_width = self.viewport_width.saturating_sub(PADDING_LEFT + PADDING_RIGHT);

        // Must match editor.rs calculation exactly:
        // body_center_offset = (text_area_width - MAX_BODY_WIDTH) / 2
        // content_width = text_area_width - 2 * body_center_offset
        // wrap_width = if body_center_offset > 0 { Some(content_width) } else { None }
        let wrap_width = if self.text_align == TextAlign::Center && text_area_width > MAX_BODY_WIDTH {
            let body_center_offset = (text_area_width - MAX_BODY_WIDTH) / 2;
            let content_width = text_area_width.saturating_sub(2 * body_center_offset);
            Some(content_width)
        } else {
            None
        };

        for doc in &mut self.documents {
            doc.wrap_width = wrap_width;
        }
    }

    pub fn goto_line(&mut self, line_num: usize) {
        let vh = self.viewport_height;
        self.doc_mut().goto_line(line_num);
        self.doc_mut().adjust_scroll(vh);
    }

    pub fn clear(&mut self) {
        self.doc_mut().clear()
    }

    pub fn undo(&mut self) -> bool {
        self.doc_mut().undo()
    }

    pub fn push_undo(&mut self) {
        self.doc_mut().push_undo()
    }
}
