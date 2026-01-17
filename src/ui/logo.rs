use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Widget},
};

use super::theme;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct LogoWidget;

impl LogoWidget {
    pub fn new() -> Self {
        Self
    }

    fn logo_lines() -> &'static [&'static str] {
        &[
            r" adPPYYba    dPPPPPb  ",
            r"       Y8   dP     Yb ",
            r" adPPPPP88  dPPPPPPPP",
            r"88     88   dP        ",
            r"  8bbdP Y8   YbPPPPP  ",
        ]
    }

    fn tagline() -> &'static str {
        "A TUI markdown paper"
    }
}

impl Widget for LogoWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let logo = Self::logo_lines();
        let tagline = Self::tagline();
        let version = format!("v{}", VERSION);

        // Calculate popup size with more padding
        let content_width = logo.iter().map(|l| l.len()).max().unwrap_or(0).max(tagline.len());
        let popup_width = (content_width as u16 + 12).min(area.width.saturating_sub(4));
        let popup_height = (logo.len() as u16 + 10).min(area.height.saturating_sub(4));

        // Center the popup
        let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Fill with solid light background
        let bg = theme::modal_bg();
        let fg = theme::modal_fg();
        let fg_muted = theme::modal_fg_muted();
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
            .title(" aeph ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(fg).bg(bg))
            .style(Style::default().bg(bg))
            .padding(Padding::new(2, 2, 1, 1));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        // Draw logo (centered)
        let text_style = Style::default().fg(fg).bg(bg);
        for (i, line_str) in logo.iter().enumerate() {
            if i as u16 >= inner.height.saturating_sub(4) {
                break;
            }

            let x_offset = (inner.width.saturating_sub(line_str.len() as u16)) / 2;
            let line = Line::from(Span::styled(*line_str, text_style));
            buf.set_line(inner.x + x_offset, inner.y + i as u16, &line, inner.width);
        }

        // Draw tagline (centered)
        let tagline_y = inner.y + logo.len() as u16 + 1;
        if tagline_y < popup_area.y + popup_height - 3 {
            let x_offset = (inner.width.saturating_sub(tagline.len() as u16)) / 2;
            let tagline_line = Line::from(Span::styled(
                tagline,
                Style::default().fg(fg_muted).bg(bg),
            ));
            buf.set_line(inner.x + x_offset, tagline_y, &tagline_line, inner.width);
        }

        // Draw version (centered)
        let version_y = tagline_y + 2;
        if version_y < popup_area.y + popup_height - 1 {
            let x_offset = (inner.width.saturating_sub(version.len() as u16)) / 2;
            let version_line = Line::from(Span::styled(
                &version,
                Style::default().fg(fg_muted).bg(bg),
            ));
            buf.set_line(inner.x + x_offset, version_y, &version_line, inner.width);
        }
    }
}
