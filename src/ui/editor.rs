use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use super::theme;

#[derive(Clone, Copy, Default, PartialEq)]
pub enum GridStyle {
    None,
    Dots,
    #[default]
    Lines,
}

impl GridStyle {
    pub fn next(self) -> Self {
        match self {
            GridStyle::None => GridStyle::Dots,
            GridStyle::Dots => GridStyle::Lines,
            GridStyle::Lines => GridStyle::None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            GridStyle::None => "Grid: Off",
            GridStyle::Dots => "Grid: Dots",
            GridStyle::Lines => "Grid: Lines",
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            GridStyle::None => 0,
            GridStyle::Dots => 1,
            GridStyle::Lines => 2,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => GridStyle::None,
            1 => GridStyle::Dots,
            _ => GridStyle::Lines,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
}

impl TextAlign {
    pub fn next(self) -> Self {
        match self {
            TextAlign::Left => TextAlign::Center,
            TextAlign::Center => TextAlign::Left,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TextAlign::Left => "Body: Full",
            TextAlign::Center => "Body: Center",
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            TextAlign::Left => 0,
            TextAlign::Center => 1,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => TextAlign::Center,
            _ => TextAlign::Left,
        }
    }
}

pub struct EditorWidget<'a> {
    content: &'a str,
    cursor_line: usize,
    cursor_col: usize,
    scroll_offset: usize,
    show_line_numbers: bool,
    grid_style: GridStyle,
    text_align: TextAlign,
}

impl<'a> EditorWidget<'a> {
    pub fn new(content: &'a str, cursor_line: usize, cursor_col: usize) -> Self {
        Self {
            content,
            cursor_line,
            cursor_col,
            scroll_offset: 0,
            show_line_numbers: false,
            grid_style: GridStyle::default(),
            text_align: TextAlign::default(),
        }
    }

    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn grid_style(mut self, style: GridStyle) -> Self {
        self.grid_style = style;
        self
    }

    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }

    fn highlight_line(&self, line: &str, in_code_block: bool) -> Line<'a> {
        let base_style = Style::default().fg(Color::White);
        let code_style = base_style.add_modifier(Modifier::ITALIC);
        let link_style = base_style.add_modifier(Modifier::UNDERLINED);

        let mut spans = Vec::new();
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        // Add indent
        if indent > 0 {
            spans.push(Span::styled(
                " ".repeat(indent),
                if in_code_block { code_style } else { base_style.fg(Color::DarkGray) },
            ));
        }

        // Code block fence
        if trimmed.starts_with("```") {
            spans.push(Span::styled(
                trimmed.to_string(),
                Style::default().fg(theme::accent()).add_modifier(Modifier::BOLD),
            ));
            return Line::from(spans);
        }

        // Inside code block - italic font
        if in_code_block {
            spans.push(Span::styled(trimmed.to_string(), code_style));
            return Line::from(spans);
        }

        // Syntax highlighting for markdown
        if trimmed.starts_with("- [ ] ") {
            spans.push(Span::styled("- [ ] ", base_style.fg(theme::accent_yellow())));
            Self::parse_with_links(&trimmed[6..], base_style, link_style, &mut spans);
        } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            spans.push(Span::styled("- [x] ", base_style.fg(theme::accent_green())));
            Self::parse_with_links(&trimmed[6..], base_style, link_style, &mut spans);
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            Self::parse_with_links(trimmed, base_style, link_style, &mut spans);
        } else if trimmed.starts_with("> ") {
            let quote_style = base_style.add_modifier(Modifier::ITALIC);
            Self::parse_with_links(trimmed, quote_style, link_style, &mut spans);
        } else {
            Self::parse_with_links(trimmed, base_style, link_style, &mut spans);
        }

        Line::from(spans)
    }

    /// Check if text at given position starts with a URL prefix
    fn find_url_start(text: &str) -> Option<usize> {
        const URL_PREFIXES: &[&str] = &["https://", "http://", "file://", "ftp://"];
        for prefix in URL_PREFIXES {
            if let Some(pos) = text.find(prefix) {
                return Some(pos);
            }
        }
        None
    }

    /// Find the end of a URL (stops at whitespace)
    fn find_url_end(text: &str) -> usize {
        text.find(char::is_whitespace).unwrap_or(text.len())
    }

    /// Parse text and highlight markdown links [text](url), inline code `code`, and raw URLs
    fn parse_with_links(text: &str, base_style: Style, link_style: Style, spans: &mut Vec<Span<'a>>) {
        let code_style = base_style.add_modifier(Modifier::ITALIC);

        let mut remaining = text;

        while !remaining.is_empty() {
            // Find the next special character
            let next_bracket = remaining.find('[');
            let next_backtick = remaining.find('`');
            let next_url = Self::find_url_start(remaining);

            // Find the earliest match
            let candidates = [next_bracket, next_backtick, next_url];
            let earliest = candidates.iter().filter_map(|&x| x).min();

            match earliest {
                Some(pos) if Some(pos) == next_url => {
                    // URL comes first
                    if pos > 0 {
                        spans.push(Span::styled(remaining[..pos].to_string(), base_style));
                    }
                    let url_text = &remaining[pos..];
                    let url_end = Self::find_url_end(url_text);
                    spans.push(Span::styled(url_text[..url_end].to_string(), link_style));
                    remaining = &url_text[url_end..];
                }
                Some(pos) if Some(pos) == next_bracket => {
                    // Try to parse markdown link
                    if let Some((link_end, before, link)) = Self::try_parse_link(remaining, pos) {
                        if !before.is_empty() {
                            spans.push(Span::styled(before.to_string(), base_style));
                        }
                        spans.push(Span::styled(link.to_string(), link_style));
                        remaining = &remaining[link_end..];
                        continue;
                    }
                    // Not a valid link, output up to and including [
                    spans.push(Span::styled(remaining[..pos + 1].to_string(), base_style));
                    remaining = &remaining[pos + 1..];
                }
                Some(pos) if Some(pos) == next_backtick => {
                    // Try to parse inline code
                    if let Some((code_end, before, code)) = Self::try_parse_inline_code(remaining, pos) {
                        if !before.is_empty() {
                            spans.push(Span::styled(before.to_string(), base_style));
                        }
                        spans.push(Span::styled(code.to_string(), code_style));
                        remaining = &remaining[code_end..];
                        continue;
                    }
                    spans.push(Span::styled(remaining[..pos + 1].to_string(), base_style));
                    remaining = &remaining[pos + 1..];
                }
                _ => {
                    // No special characters, output the rest
                    spans.push(Span::styled(remaining.to_string(), base_style));
                    break;
                }
            }
        }
    }

    /// Try to parse a markdown link starting at position `start`
    /// Returns (end_position, text_before, link_text) if successful
    fn try_parse_link(text: &str, start: usize) -> Option<(usize, &str, &str)> {
        let after_open = &text[start + 1..];
        let close_bracket = after_open.find(']')?;
        let after_close = &after_open[close_bracket + 1..];

        if !after_close.starts_with('(') {
            return None;
        }

        let close_paren = after_close.find(')')?;
        let link_end = start + 1 + close_bracket + 1 + close_paren + 1;

        Some((link_end, &text[..start], &text[start..link_end]))
    }

    /// Try to parse inline code starting at position `start`
    /// Returns (end_position, text_before, code_text) if successful
    fn try_parse_inline_code(text: &str, start: usize) -> Option<(usize, &str, &str)> {
        let after_open = &text[start + 1..];
        let close_backtick = after_open.find('`')?;

        // Don't match empty backticks ``
        if close_backtick == 0 {
            return None;
        }

        let code_end = start + 1 + close_backtick + 1;

        Some((code_end, &text[..start], &text[start..code_end]))
    }
}

impl Widget for EditorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = area;

        // Text area padding (grid is drawn full area, text has padding)
        const PADDING_TOP: u16 = 2;
        const PADDING_LEFT: u16 = 1;
        const PADDING_RIGHT: u16 = 1;

        let lines: Vec<&str> = self.content.lines().collect();
        let visible_height = inner.height.saturating_sub(PADDING_TOP) as usize;
        let total_lines = lines.len();

        // Calculate line number width
        let line_num_width = if self.show_line_numbers {
            total_lines.to_string().len().max(3) + 1 // +1 for space
        } else {
            0
        };

        // Calculate body centering offset (text only, grid stays full width)
        const MAX_BODY_WIDTH: u16 = 80;
        let text_area_width = inner.width.saturating_sub(line_num_width as u16 + PADDING_LEFT + PADDING_RIGHT);
        let body_center_offset = if self.text_align == TextAlign::Center && text_area_width > MAX_BODY_WIDTH {
            (text_area_width - MAX_BODY_WIDTH) / 2
        } else {
            0
        };

        // Draw grid background (always full width, no padding)
        let grid_x = inner.x;
        let grid_width = inner.width;

        if self.grid_style != GridStyle::None {
            let style = Style::default().fg(theme::grid_line());

            match self.grid_style {
                GridStyle::Dots => {
                    // Draw dots every 4 columns
                    for row in 0..inner.height {
                        for col in (0..grid_width).step_by(4) {
                            buf.set_string(grid_x + col, inner.y + row, "·", style);
                        }
                    }
                }
                GridStyle::Lines => {
                    // Terminal cells are taller than wide (~1:2 ratio)
                    let row_spacing = 1u16;
                    let col_spacing = 2u16;

                    for row in 0..inner.height {
                        let is_horizontal_line = row % row_spacing == 0;
                        for col in 0..grid_width {
                            let is_vertical_line = col % col_spacing == 0;
                            let ch = match (is_horizontal_line, is_vertical_line) {
                                (true, true) => "┼",
                                (true, false) => "─",
                                (false, true) => "│",
                                (false, false) => continue,
                            };
                            buf.set_string(grid_x + col, inner.y + row, ch, style);
                        }
                    }
                }
                GridStyle::None => {}
            }
        }

        // Track code block state for all lines up to visible area
        let mut in_code_block = false;
        for line in lines.iter().take(self.scroll_offset) {
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
            }
        }

        for (i, line_idx) in (self.scroll_offset..).take(visible_height).enumerate() {
            if line_idx >= lines.len() {
                break;
            }

            let line = lines[line_idx];
            let is_cursor_line = line_idx == self.cursor_line;
            let y = inner.y + PADDING_TOP + i as u16;

            // Check if this line starts/ends a code block
            let is_fence = line.trim_start().starts_with("```");
            let was_in_code_block = in_code_block;
            if is_fence {
                in_code_block = !in_code_block;
            }

            // Draw line number
            if self.show_line_numbers {
                let line_num = format!("{:>width$} ", line_idx + 1, width = line_num_width - 1);
                let num_style = if is_cursor_line {
                    Style::default().fg(theme::accent_yellow())
                } else {
                    Style::default().fg(theme::fg_muted())
                };
                buf.set_string(inner.x + PADDING_LEFT, y, &line_num, num_style);
            }

            // Draw line content (fence lines and content inside code block)
            let show_as_code = is_fence || was_in_code_block;
            let styled_line = self.highlight_line(line, show_as_code);
            let line_content_x = inner.x + PADDING_LEFT + line_num_width as u16 + body_center_offset;

            buf.set_line(line_content_x, y, &styled_line, text_area_width);

            // Draw cursor
            if is_cursor_line {
                // Convert character index to display width
                use unicode_width::UnicodeWidthChar;
                let display_col: usize = line.chars()
                    .take(self.cursor_col)
                    .map(|c| c.width().unwrap_or(0))
                    .sum();
                let cursor_x = line_content_x + display_col as u16;
                if cursor_x < inner.x + inner.width {
                    // Get the character at cursor position (or space if at end of line)
                    let cursor_char = line.chars().nth(self.cursor_col).unwrap_or(' ');
                    let char_str = cursor_char.to_string();
                    // Draw with reversed colors
                    buf.set_string(
                        cursor_x,
                        y,
                        &char_str,
                        Style::default().bg(theme::cursor_bg()).fg(theme::cursor_fg()),
                    );
                }
            }
        }
    }
}
