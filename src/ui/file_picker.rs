use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Widget},
};

use super::theme;

pub struct FilePickerWidget<'a> {
    files: &'a [(usize, &'a str, bool, String)], // (index, name, modified, preview)
    selection: usize,
}

impl<'a> FilePickerWidget<'a> {
    pub fn new(files: &'a [(usize, &'a str, bool, String)], _current: usize, selection: usize) -> Self {
        Self { files, selection }
    }
}

impl Widget for FilePickerWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate popup size
        let popup_width = 70.min(area.width.saturating_sub(4));
        let popup_height = (self.files.len() as u16 + 4).min(area.height.saturating_sub(4));

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
            .title(" Open Document ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(fg).bg(bg))
            .style(Style::default().bg(bg))
            .padding(Padding::horizontal(1));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        // Draw file list
        for (i, (idx, _name, _modified, preview)) in self.files.iter().enumerate() {
            if i as u16 >= inner.height.saturating_sub(1) {
                break;
            }

            let is_selected = *idx == self.selection;

            let preview_text = if preview.is_empty() {
                "(empty)".to_string()
            } else {
                preview.clone()
            };

            let line = Line::from(vec![
                Span::styled(
                    format!(" {} ", idx + 1),
                    if is_selected {
                        Style::default().fg(theme::modal_bg()).bg(fg).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
                    },
                ),
                Span::styled(
                    format!(" {}", preview_text),
                    if is_selected {
                        Style::default().fg(fg).bg(bg)
                    } else {
                        Style::default().fg(theme::modal_fg_muted()).bg(bg)
                    },
                ),
            ]);

            buf.set_line(inner.x, inner.y + i as u16, &line, inner.width);
        }

        // Draw hint at bottom
        let hint = Line::from(Span::styled(
            "↑↓/1-9: select, Enter: open, Esc: close",
            Style::default().fg(theme::modal_fg_muted()).bg(bg),
        ));
        let hint_y = popup_area.y + popup_height - 2;
        buf.set_line(inner.x, hint_y, &hint, inner.width);
    }
}
