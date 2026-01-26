use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

use crate::markdown;
use super::typing;

pub struct Document {
    pub content: String,
    pub file_path: Option<PathBuf>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub modified: bool,
    undo_stack: Vec<String>,
}

impl Document {
    pub fn new(file_path: Option<PathBuf>) -> Result<Self> {
        let content = if let Some(ref path) = file_path {
            if path.exists() {
                fs::read_to_string(path)
                    .with_context(|| format!("Failed to read: {}", path.display()))?
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        Ok(Self {
            content,
            file_path,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            modified: false,
            undo_stack: Vec::new(),
        })
    }

    pub fn name(&self) -> &str {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[scratch]")
    }

    pub fn preview(&self, max_chars: usize) -> String {
        // Find first non-empty line
        let first_content = self.content
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty())
            .unwrap_or("");

        if first_content.chars().count() > max_chars {
            format!("{}...", first_content.chars().take(max_chars).collect::<String>())
        } else {
            first_content.to_string()
        }
    }

    pub fn lines(&self) -> Vec<&str> {
        self.content.lines().collect()
    }

    pub fn line_count(&self) -> usize {
        self.content.lines().count().max(1)
    }

    pub fn current_line(&self) -> &str {
        self.lines().get(self.cursor_line).copied().unwrap_or("")
    }

    fn line_byte_offset(&self, line_idx: usize) -> usize {
        self.content
            .lines()
            .take(line_idx)
            .map(|l| l.len() + 1)
            .sum()
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(ref path) = self.file_path {
            fs::write(path, &self.content)
                .with_context(|| format!("Failed to save: {}", path.display()))?;
            self.modified = false;
        }
        Ok(())
    }

    pub fn format(&mut self) -> Result<()> {
        let formatted = markdown::format_markdown(&self.content)?;
        if formatted != self.content {
            self.content = formatted;
            self.modified = true;
            let line_count = self.line_count();
            self.cursor_line = self.cursor_line.min(line_count.saturating_sub(1));
            let line_len = self.current_line().len();
            self.cursor_col = self.cursor_col.min(line_len);
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        if !self.content.is_empty() {
            self.push_undo();
            self.content.clear();
            self.cursor_line = 0;
            self.cursor_col = 0;
            self.scroll_offset = 0;
            self.modified = true;
        }
    }

    pub fn push_undo(&mut self) {
        const MAX_UNDO: usize = 10;
        // Don't push if content is same as last undo state
        if self.undo_stack.last().map(|s| s.as_str()) == Some(&self.content) {
            return;
        }
        if self.undo_stack.len() >= MAX_UNDO {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(self.content.clone());
    }

    pub fn undo(&mut self) -> bool {
        // Skip states that match current content
        while self.undo_stack.last().map(|s| s.as_str()) == Some(&self.content) {
            self.undo_stack.pop();
        }
        if let Some(prev) = self.undo_stack.pop() {
            self.content = prev;
            self.cursor_line = self.cursor_line.min(self.line_count().saturating_sub(1));
            self.cursor_col = self.cursor_col.min(self.current_line().chars().count());
            self.modified = true;
            true
        } else {
            false
        }
    }

    pub fn toggle_current_task(&mut self) {
        let line = self.current_line().to_string();
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        let new_line = if trimmed.starts_with("- [ ] ") {
            format!("{}- [x] {}", " ".repeat(indent), &trimmed[6..])
        } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            format!("{}- [ ] {}", " ".repeat(indent), &trimmed[6..])
        } else {
            return;
        };

        self.replace_current_line(&new_line);
    }

    pub fn insert_task(&mut self) {
        self.move_cursor_line_end();
        self.insert_newline();
        self.insert_str("- [ ] ");
    }

    /// Check if current line is a list line (bullet, task, numbered, or blockquote)
    pub fn is_list_line(&self) -> bool {
        let trimmed = self.current_line().trim_start();
        // Check for list markers - be lenient about what follows
        trimmed.starts_with("- [ ]")
            || trimmed.starts_with("- [x]")
            || trimmed.starts_with("- [X]")
            || trimmed.starts_with("- ")
            || trimmed.starts_with("-\t")
            || trimmed == "-"
            || trimmed.starts_with("* ")
            || trimmed.starts_with("*\t")
            || trimmed == "*"
            || trimmed.starts_with("> ")
            || trimmed.starts_with(">\t")
            || trimmed == ">"
            || Self::detect_numbered_list(trimmed).is_some()
    }

    /// Check if current line has leading whitespace
    pub fn has_leading_whitespace(&self) -> bool {
        self.current_line().starts_with(' ')
    }

    /// Indent current line by adding 2 spaces at the beginning
    pub fn indent_line(&mut self) {
        let line_start = self.line_byte_offset(self.cursor_line);
        self.content.insert_str(line_start, "  ");
        self.cursor_col += 2;
        self.modified = true;
    }

    /// Unindent current line by removing up to 2 spaces from the beginning
    pub fn unindent_line(&mut self) {
        let line = self.current_line();
        let spaces_to_remove = line.chars().take(2).take_while(|&c| c == ' ').count();

        if spaces_to_remove == 0 {
            return;
        }

        let line_start = self.line_byte_offset(self.cursor_line);
        self.content.drain(line_start..line_start + spaces_to_remove);
        self.cursor_col = self.cursor_col.saturating_sub(spaces_to_remove);
        self.modified = true;
    }

    fn replace_current_line(&mut self, new_line: &str) {
        let lines: Vec<&str> = self.content.lines().collect();
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

        if self.cursor_line < new_lines.len() {
            new_lines[self.cursor_line] = new_line.to_string();
        }

        self.content = new_lines.join("\n");
        if !self.content.ends_with('\n') && !self.content.is_empty() {
            self.content.push('\n');
        }
        self.modified = true;
    }

    pub fn insert_char(&mut self, c: char) {
        let offset = self.cursor_offset();
        self.content.insert(offset, c);
        self.cursor_col += 1;
        self.modified = true;
    }

    /// Get up to n characters before the cursor on the current line
    fn chars_before_cursor(&self, n: usize) -> String {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();
        let start = self.cursor_col.saturating_sub(n);
        chars[start..self.cursor_col].iter().collect()
    }

    /// Get the character at the cursor position (if any)
    fn char_at_cursor(&self) -> Option<char> {
        let line = self.current_line();
        line.chars().nth(self.cursor_col)
    }

    /// Move cursor past consecutive occurrences of the given character
    fn move_past_char(&mut self, c: char) {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();
        while self.cursor_col < chars.len() && chars[self.cursor_col] == c {
            self.cursor_col += 1;
        }
    }

    /// Insert a character with auto-pairing support
    pub fn insert_char_with_auto_pair(&mut self, c: char) {
        // Get context before inserting
        let chars_before_insert = self.chars_before_cursor(4);
        let char_immediately_before = chars_before_insert.chars().last();
        let char_at_cursor = self.char_at_cursor();

        // Escape mechanism: if we're inside a pair of the same character (char before AND after
        // are the same as the typed char), move cursor past the sequence first, then insert.
        // This allows typing ``` after `` without getting stuck in the middle.
        if char_immediately_before == Some(c) && char_at_cursor == Some(c) {
            self.move_past_char(c);
            self.insert_char(c);
            return; // No centering after escape
        }

        // Insert the character normally
        self.insert_char(c);

        // Check for bracket pairing (e.g., "(" followed by ")" -> move cursor back)
        if let Some(move_back) = typing::should_center_for_bracket(char_immediately_before, c) {
            self.cursor_col = self.cursor_col.saturating_sub(move_back);
            return;
        }

        // Check for same-char centering (e.g., "``", "****")
        if let Some(move_back) = typing::should_center_for_same_char(&chars_before_insert, c) {
            self.cursor_col = self.cursor_col.saturating_sub(move_back);
        }
    }

    pub fn insert_str(&mut self, s: &str) {
        let offset = self.cursor_offset();
        self.content.insert_str(offset, s);
        self.cursor_col += s.width();
        self.modified = true;
    }

    pub fn insert_newline(&mut self) {
        let current = self.current_line().to_string();
        let trimmed = current.trim_start();
        let indent = current.len() - trimmed.len();
        let indent_str = &current[..indent];

        let prefix = if trimmed.starts_with("- [ ] ") {
            Some("- [ ] ")
        } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            Some("- [ ] ")
        } else if trimmed.starts_with("- ") {
            Some("- ")
        } else if trimmed.starts_with("* ") {
            Some("* ")
        } else if trimmed.starts_with("> ") {
            Some("> ")
        } else {
            Self::detect_numbered_list(trimmed)
        };

        if let Some(p) = prefix {
            if trimmed == p.trim_end() || trimmed == p {
                self.move_cursor_line_start();
                let line_len = current.len();
                for _ in 0..line_len {
                    self.delete_char_after();
                }
                return;
            }
        }

        let offset = self.cursor_offset();
        self.content.insert(offset, '\n');
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.modified = true;

        if let Some(p) = prefix {
            let continuation = if p == "NUM" {
                Self::next_number(trimmed)
            } else {
                p.to_string()
            };
            let full_prefix = format!("{}{}", indent_str, continuation);
            self.insert_str(&full_prefix);
        }
    }

    fn detect_numbered_list(line: &str) -> Option<&'static str> {
        let mut chars = line.chars().peekable();
        let mut has_digit = false;

        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                has_digit = true;
                chars.next();
            } else {
                break;
            }
        }

        if has_digit && chars.next() == Some('.') && chars.next() == Some(' ') {
            return Some("NUM");
        }
        None
    }

    fn next_number(line: &str) -> String {
        if let Some(dot_pos) = line.find(". ") {
            if let Ok(num) = line[..dot_pos].parse::<usize>() {
                return format!("{}. ", num + 1);
            }
        }
        "1. ".to_string()
    }

    pub fn delete_char_before(&mut self) {
        if self.cursor_col > 0 {
            // Find byte range of character before cursor
            let line = self.current_line();
            let char_indices: Vec<(usize, char)> = line.char_indices().collect();
            if self.cursor_col <= char_indices.len() {
                let line_start = self.line_byte_offset(self.cursor_line);
                let (char_byte_offset, ch) = char_indices[self.cursor_col - 1];
                let start = line_start + char_byte_offset;
                let end = start + ch.len_utf8();
                self.content.drain(start..end);
                self.cursor_col -= 1;
                self.modified = true;
            }
        } else if self.cursor_line > 0 {
            let lines: Vec<&str> = self.content.lines().collect();
            let prev_char_count = lines.get(self.cursor_line - 1).map(|l| l.chars().count()).unwrap_or(0);

            let offset = self.line_byte_offset(self.cursor_line) - 1;
            if offset < self.content.len() {
                self.content.remove(offset);
                self.cursor_line -= 1;
                self.cursor_col = prev_char_count;
                self.modified = true;
            }
        }
    }

    pub fn delete_char_after(&mut self) {
        let offset = self.cursor_offset();
        if offset < self.content.len() {
            // Find the character at offset and remove all its bytes
            if let Some(ch) = self.content[offset..].chars().next() {
                self.content.drain(offset..offset + ch.len_utf8());
                self.modified = true;
            }
        }
    }

    fn cursor_offset(&self) -> usize {
        let line_start = self.line_byte_offset(self.cursor_line);
        let line = self.current_line();
        // Convert character index to byte offset
        let byte_col: usize = line.chars().take(self.cursor_col).map(|c| c.len_utf8()).sum();
        line_start + byte_col
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.clamp_cursor_col();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_line < self.line_count().saturating_sub(1) {
            self.cursor_line += 1;
            self.clamp_cursor_col();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.current_line().chars().count();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let char_count = self.current_line().chars().count();
        if self.cursor_col < char_count {
            self.cursor_col += 1;
        } else if self.cursor_line < self.line_count().saturating_sub(1) {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_cursor_line_start(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_cursor_line_end(&mut self) {
        self.cursor_col = self.current_line().chars().count();
    }

    pub fn page_up(&mut self, viewport_height: usize) {
        let jump = viewport_height.saturating_sub(2);
        self.cursor_line = self.cursor_line.saturating_sub(jump);
        self.clamp_cursor_col();
    }

    pub fn page_down(&mut self, viewport_height: usize) {
        let jump = viewport_height.saturating_sub(2);
        self.cursor_line = (self.cursor_line + jump).min(self.line_count().saturating_sub(1));
        self.clamp_cursor_col();
    }

    fn clamp_cursor_col(&mut self) {
        let char_count = self.current_line().chars().count();
        self.cursor_col = self.cursor_col.min(char_count);
    }

    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor_line - viewport_height + 1;
        }
    }

    pub fn goto_line(&mut self, line_num: usize) {
        let target = line_num.saturating_sub(1);
        self.cursor_line = target.min(self.line_count().saturating_sub(1));
        self.cursor_col = 0;
    }

    pub fn delete_line(&mut self) {
        let lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
        if lines.is_empty() {
            return;
        }

        let mut new_lines = lines;
        if self.cursor_line < new_lines.len() {
            new_lines.remove(self.cursor_line);
        }

        if new_lines.is_empty() {
            self.content.clear();
            self.cursor_line = 0;
            self.cursor_col = 0;
        } else {
            self.content = new_lines.join("\n");
            if !self.content.is_empty() {
                self.content.push('\n');
            }
            self.cursor_line = self.cursor_line.min(new_lines.len().saturating_sub(1));
            self.clamp_cursor_col();
        }
        self.modified = true;
    }
}
