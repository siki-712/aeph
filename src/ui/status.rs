use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

use super::theme;

pub struct StatusBar<'a> {
    modified: bool,
    cursor_line: usize,
    cursor_col: usize,
    total_lines: usize,
    goto_input: Option<&'a str>,
    doc_num: usize,
    notification: Option<&'a str>,
    leader_input: Option<&'a str>,
}

impl<'a> StatusBar<'a> {
    pub fn new(
        modified: bool,
        cursor_line: usize,
        cursor_col: usize,
        total_lines: usize,
    ) -> Self {
        Self {
            modified,
            cursor_line,
            cursor_col,
            total_lines,
            goto_input: None,
            doc_num: 1,
            notification: None,
            leader_input: None,
        }
    }

    pub fn goto_input(mut self, input: Option<&'a str>) -> Self {
        self.goto_input = input;
        self
    }

    pub fn doc_indicator(mut self, num: usize, _total: usize) -> Self {
        self.doc_num = num;
        self
    }

    pub fn notification(mut self, msg: Option<&'a str>) -> Self {
        self.notification = msg;
        self
    }

    pub fn leader_input(mut self, input: Option<&'a str>) -> Self {
        self.leader_input = input;
        self
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = theme::bg_dark();
        let fg = theme::fg_primary();

        // Fill background with solid color
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_char(' ');
                cell.set_style(Style::default().bg(bg));
            }
        }

        // If in goto mode, show input prompt
        if let Some(input) = self.goto_input {
            let prompt = Line::from(vec![
                Span::styled(
                    " Go to line: ",
                    Style::default().fg(theme::accent_yellow()).bg(bg),
                ),
                Span::styled(
                    format!("{}_", input),
                    Style::default().fg(theme::fg_primary()).bg(bg),
                ),
            ]);
            buf.set_line(area.x, area.y, &prompt, area.width);
            return;
        }

        // If in leader key mode, show command input
        if let Some(input) = self.leader_input {
            let prompt = Line::from(Span::styled(
                format!(" :{}", input),
                Style::default().fg(theme::accent_yellow()).bg(bg),
            ));
            buf.set_line(area.x, area.y, &prompt, area.width);
            return;
        }

        // Show notification if present
        if let Some(msg) = self.notification {
            let notification = Line::from(Span::styled(
                format!(" {} ", msg),
                Style::default().fg(theme::accent_green()).bg(bg),
            ));
            buf.set_line(area.x, area.y, &notification, area.width);
            return;
        }

        // Left side: doc number only
        let modified_marker = if self.modified { "•" } else { "" };

        let left = Line::from(vec![
            Span::styled(
                format!(" [{}]{} ", self.doc_num, modified_marker),
                Style::default().fg(theme::accent()).bg(bg),
            ),
        ]);

        // Right side: help hint + position info + app name
        let help_hint = "Ctrl+H for help ";
        let right_text = format!(
            "{}:{} / {} ",
            self.cursor_line + 1,
            self.cursor_col + 1,
            self.total_lines
        );
        let app_name = concat!("aeph v", env!("CARGO_PKG_VERSION"), " ");

        buf.set_line(area.x, area.y, &left, area.width);

        let total_right_len = help_hint.len() + right_text.len() + app_name.len();
        let right_x = area.x + area.width - total_right_len as u16;
        if right_x > area.x {
            let right = Line::from(vec![
                Span::styled(help_hint, Style::default().fg(theme::fg_muted()).bg(bg)),
                Span::styled(right_text, Style::default().fg(fg).bg(bg)),
                Span::styled(app_name, Style::default().fg(theme::accent()).bg(bg)),
            ]);
            buf.set_line(right_x, area.y, &right, area.width);
        }
    }
}
