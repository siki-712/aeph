use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use super::theme;

/// Placeholder quotes shown when document is empty (Unix Philosophy - universal)
const PLACEHOLDER_QUOTES: &[&str] = &[
    "Do one thing and do it well.",
    "Keep it simple.",
    "Silence is golden.",
    "Small is beautiful.",
    "Build a prototype as soon as possible.",
    "Choose simplicity over efficiency.",
    "Make it work, make it right, make it fast.",
    "Worse is better.",
    "When in doubt, use brute force.",
    "Rule of Diversity: Distrust all claims for the one true way.",
];

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
    quote_seed: u64,
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
            quote_seed: 0,
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

    pub fn quote_seed(mut self, seed: u64) -> Self {
        self.quote_seed = seed;
        self
    }

    /// Restore grid characters at positions where the buffer cell contains whitespace.
    fn restore_grid_over_whitespace(
        grid_style: GridStyle,
        buf: &mut Buffer,
        origin: (u16, u16),
        area_width: u16,
        x_start: u16,
        x_end: u16,
        y: u16,
    ) {
        if grid_style == GridStyle::None {
            return;
        }
        let grid_line_style = Style::default().fg(theme::grid_line());
        let dot_style = Style::default().fg(Color::Rgb(60, 60, 65));
        for x in x_start..x_end {
            if x >= origin.0 + area_width {
                break;
            }
            let cell = &buf[(x, y)];
            if cell.symbol().chars().all(|c| c.is_whitespace()) {
                let col = x - origin.0;
                let row = y - origin.1;
                let grid_char = match grid_style {
                    GridStyle::Dots => {
                        if col % 4 == 0 { Some(("·", dot_style)) } else { None }
                    }
                    GridStyle::Lines => {
                        let is_h = row % 1 == 0;
                        let is_v = col % 2 == 0;
                        match (is_h, is_v) {
                            (true, true) => Some(("┼", grid_line_style)),
                            (true, false) => Some(("─", grid_line_style)),
                            (false, true) => Some(("│", grid_line_style)),
                            (false, false) => None,
                        }
                    }
                    GridStyle::None => None,
                };
                if let Some((ch, st)) = grid_char {
                    buf.set_string(x, y, ch, st);
                }
            }
        }
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

    /// Segment of a line for visual display
    fn wrap_line_for_display(line: &str, max_width: Option<u16>) -> Vec<VisualSegment> {
        use unicode_width::UnicodeWidthChar;

        let max_width = match max_width {
            Some(w) if w > 0 => w,
            _ => return vec![VisualSegment {
                text: line.to_string(),
                char_start: 0,
                char_end: line.chars().count(),
            }],
        };

        let mut segments = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut seg_start: usize = 0;

        while seg_start < chars.len() {
            let mut width: u16 = 0;
            let mut seg_end = seg_start;

            for (i, &ch) in chars[seg_start..].iter().enumerate() {
                let ch_width = ch.width().unwrap_or(0) as u16;
                if width + ch_width > max_width && i > 0 {
                    break;
                }
                width += ch_width;
                seg_end = seg_start + i + 1;
            }

            let text: String = chars[seg_start..seg_end].iter().collect();
            segments.push(VisualSegment {
                text,
                char_start: seg_start,
                char_end: seg_end,
            });

            seg_start = seg_end;
        }

        if segments.is_empty() {
            segments.push(VisualSegment {
                text: String::new(),
                char_start: 0,
                char_end: 0,
            });
        }

        segments
    }
}

/// Visual segment of a wrapped line
struct VisualSegment {
    text: String,
    char_start: usize,
    char_end: usize,
}

impl Widget for EditorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = area;

        // Text area padding (grid is drawn full area, text has padding)
        const PADDING_TOP: u16 = 2;
        const PADDING_LEFT: u16 = 1;
        const PADDING_RIGHT: u16 = 1;

        let collected: Vec<&str> = self.content.lines().collect();
        let lines = if collected.is_empty() { vec![""] } else { collected };
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
        // Text content width (constrained to MAX_BODY_WIDTH when centering)
        let content_width = text_area_width.saturating_sub(2 * body_center_offset);

        // Draw grid background (always full width)
        let grid_x = inner.x;
        let grid_width = inner.width;

        if self.grid_style != GridStyle::None {
            let style = Style::default().fg(theme::grid_line());

            match self.grid_style {
                GridStyle::Dots => {
                    // Dots use a slightly brighter color than lines
                    let dot_style = Style::default().fg(Color::Rgb(60, 60, 65));
                    for row in 0..inner.height {
                        for col in (0..grid_width).step_by(4) {
                            buf.set_string(grid_x + col, inner.y + row, "·", dot_style);
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

        // Show placeholder quote when document is empty
        if self.content.is_empty() {
            // Select quote based on seed (changes each launch)
            let quote_idx = (self.quote_seed as usize) % PLACEHOLDER_QUOTES.len();
            let quote = PLACEHOLDER_QUOTES[quote_idx];
            let placeholder_x = inner.x + PADDING_LEFT + line_num_width as u16 + body_center_offset;
            let placeholder_y = inner.y + PADDING_TOP;
            let placeholder_style = Style::default().fg(Color::Rgb(140, 140, 140)).add_modifier(Modifier::ITALIC);
            buf.set_string(placeholder_x, placeholder_y, quote, placeholder_style);

            // Restore grid characters where placeholder has whitespace
            Self::restore_grid_over_whitespace(
                self.grid_style, buf, (inner.x, inner.y), inner.width,
                placeholder_x, placeholder_x + quote.len() as u16, placeholder_y,
            );
        }

        // Track code block state for all lines up to visible area
        let mut in_code_block = false;
        for line in lines.iter().take(self.scroll_offset) {
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
            }
        }

        let line_content_x = inner.x + PADDING_LEFT + line_num_width as u16 + body_center_offset;
        let wrap_width = if body_center_offset > 0 { Some(content_width) } else { None };

        let mut display_row: usize = 0;

        for line_idx in self.scroll_offset..lines.len() {
            if display_row >= visible_height {
                break;
            }

            let line = lines[line_idx];
            let is_cursor_line = line_idx == self.cursor_line;

            // Check if this line starts/ends a code block
            let is_fence = line.trim_start().starts_with("```");
            let was_in_code_block = in_code_block;
            if is_fence {
                in_code_block = !in_code_block;
            }

            let show_as_code = is_fence || was_in_code_block;

            // Get visual lines (wrapped segments)
            let visual_lines = Self::wrap_line_for_display(line, wrap_width);

            for (seg_idx, segment) in visual_lines.iter().enumerate() {
                if display_row >= visible_height {
                    break;
                }

                let y = inner.y + PADDING_TOP + display_row as u16;

                // Draw line number (only for first segment)
                if self.show_line_numbers && seg_idx == 0 {
                    let line_num = format!("{:>width$} ", line_idx + 1, width = line_num_width - 1);
                    let num_style = if is_cursor_line {
                        Style::default().fg(theme::accent_yellow())
                    } else {
                        Style::default().fg(theme::fg_muted())
                    };
                    let num_x = inner.x + PADDING_LEFT;
                    buf.set_string(num_x, y, &line_num, num_style);
                    Self::restore_grid_over_whitespace(
                        self.grid_style, buf, (inner.x, inner.y), inner.width,
                        num_x, num_x + line_num_width as u16, y,
                    );
                }

                // Draw line content
                let styled_line = self.highlight_line(&segment.text, show_as_code);
                buf.set_line(line_content_x, y, &styled_line, content_width);

                // Restore grid characters
                {
                    use unicode_width::UnicodeWidthStr;
                    let seg_display_width = segment.text.width().min(content_width as usize) as u16;
                    Self::restore_grid_over_whitespace(
                        self.grid_style, buf, (inner.x, inner.y), inner.width,
                        line_content_x, line_content_x + seg_display_width, y,
                    );
                }

                // Draw cursor if on this segment
                if is_cursor_line {
                    if self.cursor_col >= segment.char_start && self.cursor_col < segment.char_end {
                        use unicode_width::UnicodeWidthChar;
                        let col_in_seg = self.cursor_col - segment.char_start;
                        let display_col: usize = segment.text.chars()
                            .take(col_in_seg)
                            .map(|c| c.width().unwrap_or(0))
                            .sum();
                        let cursor_x = line_content_x + display_col as u16;
                        if cursor_x < line_content_x + content_width {
                            let cursor_char = segment.text.chars().nth(col_in_seg).unwrap_or(' ');
                            buf.set_string(
                                cursor_x, y, &cursor_char.to_string(),
                                Style::default().bg(theme::cursor_bg()).fg(theme::cursor_fg()),
                            );
                        }
                    } else if seg_idx == visual_lines.len() - 1 && self.cursor_col >= segment.char_end {
                        // Cursor at end of line
                        use unicode_width::UnicodeWidthStr;
                        let cursor_x = line_content_x + segment.text.width() as u16;
                        if cursor_x < line_content_x + content_width {
                            buf.set_string(
                                cursor_x, y, " ",
                                Style::default().bg(theme::cursor_bg()).fg(theme::cursor_fg()),
                            );
                        }
                    }
                }

                display_row += 1;
            }
        }
    }
}
