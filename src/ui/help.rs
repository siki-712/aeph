use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Widget},
};

use super::theme;

pub struct HelpWidget {
    page: usize,
}

impl HelpWidget {
    pub fn new(page: usize) -> Self {
        Self { page }
    }

    pub const TOTAL_PAGES: usize = 2;

    fn page_title(&self) -> &'static str {
        match self.page {
            0 => " Help (1/2) ",
            1 => " Leader Mode (2/2) ",
            _ => " Help ",
        }
    }

    fn shortcuts(&self) -> Vec<(&'static str, &'static str)> {
        match self.page {
            0 => vec![
                ("Ctrl+C", "Copy all to clipboard"),
                ("Ctrl+D", "Clear document"),
                ("Ctrl+Z", "Undo"),
                ("Ctrl+F", "Format markdown"),
                ("Ctrl+T", "Toggle task under cursor"),
                ("Ctrl+N", "New task"),
                ("Ctrl+O", "Open document picker"),
                ("Ctrl+Shift+←/→", "Switch document"),
                ("Ctrl+G", "Go to line"),
                ("Ctrl+B", "Toggle grid style"),
                ("Ctrl+H / F1", "Toggle help"),
                ("↑/↓/←/→", "Move cursor"),
                ("PgUp/PgDn", "Scroll page"),
            ],
            1 => vec![
                (":", "Start command (type within 500ms)"),
                (":q", "Quit"),
                (":dd", "Delete current line"),
                (":yy", "Yank (copy) current line"),
                (":p", "Paste below cursor"),
                (":1 - :9", "Switch to document 1-9"),
            ],
            _ => vec![],
        }
    }
}

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let shortcuts = self.shortcuts();

        // Calculate popup size
        let popup_width = 56.min(area.width.saturating_sub(4));
        let popup_height = (shortcuts.len() as u16 + 4).min(area.height.saturating_sub(4));

        // Center the popup
        let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Fill with solid light background
        let bg = theme::modal_bg();
        let fg = theme::modal_fg();
        for y in popup_area.y..popup_area.y + popup_area.height {
            for x in popup_area.x..popup_area.x + popup_area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ');
                    cell.set_style(Style::default().bg(bg));
                }
            }
        }

        // Draw border
        let block = Block::default()
            .title(self.page_title())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(fg).bg(bg))
            .style(Style::default().bg(bg))
            .padding(Padding::horizontal(1));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        // Draw shortcuts
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            if i as u16 >= inner.height.saturating_sub(1) {
                break;
            }

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>12}", key),
                    Style::default()
                        .fg(fg)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ", Style::default().bg(bg)),
                Span::styled(*desc, Style::default().fg(theme::modal_fg_muted()).bg(bg)),
            ]);

            buf.set_line(inner.x, inner.y + i as u16, &line, inner.width);
        }

        // Draw hint at bottom
        let hint = Line::from(Span::styled(
            "←/→: page  Esc: close",
            Style::default().fg(theme::modal_fg_muted()).bg(bg),
        ));
        let hint_y = popup_area.y + popup_height - 2;
        buf.set_line(inner.x, hint_y, &hint, inner.width);
    }
}
