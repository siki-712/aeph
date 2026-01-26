use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;

use super::document::Document;
use crate::storage;
use crate::storage::Config;
use crate::ui::GridStyle;

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
    pub goto_input: Option<String>,
    pub notification: Option<String>,
    pub leader_buffer: Option<(String, Instant)>,
    pub grid_style: GridStyle,
}

impl AppState {
    pub fn new(file: Option<PathBuf>) -> Result<Self> {
        let mut documents = Vec::new();

        // Create default documents (paper1.md ~ paper9.md) in data directory
        for i in 1..=MAX_DOCUMENTS {
            let path = storage::get_paper_path(i)?;
            documents.push(Document::new(Some(path))?);
        }

        // Determine starting document
        let current_doc = if let Some(path) = file {
            // If a specific file was provided, replace the first document
            documents[0] = Document::new(Some(path))?;
            0
        } else {
            // Restore last opened document
            storage::load_last_doc().min(MAX_DOCUMENTS - 1)
        };

        // Show logo on first launch (all documents empty)
        let all_empty = documents.iter().all(|d| d.content.is_empty());

        // Load configuration
        let config = storage::load_config();

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
            goto_input: None,
            notification: None,
            leader_buffer: None,
            grid_style: GridStyle::default(),
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
