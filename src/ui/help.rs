use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Widget},
};

use super::theme;

pub struct HelpWidget;

impl HelpWidget {
    pub fn new() -> Self {
        Self
    }

    fn shortcuts() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Ctrl+C", "Copy all to clipboard"),
            ("Ctrl+Q", "Quit"),
            ("Ctrl+D", "Clear document"),
            ("Ctrl+Z", "Undo"),
            ("Ctrl+F", "Format markdown"),
            ("Ctrl+T", "Toggle task under cursor"),
            ("Ctrl+N", "New task"),
            ("Ctrl+O", "Open document picker"),
            ("Ctrl+G", "Go to line"),
            ("Ctrl+H / F1", "Toggle help"),
            ("Esc", "Close dialog"),
            ("↑/↓/←/→", "Move cursor"),
            ("PgUp/PgDn", "Scroll page"),
        ]
    }
}

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let shortcuts = Self::shortcuts();

        // Calculate popup size
        let popup_width = 50.min(area.width.saturating_sub(4));
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
            .title(" Help ")
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
            "Esc: close",
            Style::default().fg(theme::modal_fg_muted()).bg(bg),
        ));
        let hint_y = popup_area.y + popup_height - 2;
        buf.set_line(inner.x, hint_y, &hint, inner.width);
    }
}
