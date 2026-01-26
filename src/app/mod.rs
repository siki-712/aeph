mod document;
mod leader;
mod state;
mod typing;

use anyhow::Result;
use crossterm::{
    event::{self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Layout}, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::storage;
use crate::ui::{EditorWidget, FilePickerWidget, HelpWidget, LogoWidget, StatusBar};
use state::AppState;

pub fn run(file: Option<PathBuf>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize state
    let mut state = AppState::new(file)?;

    // Main loop
    let result = run_app(&mut terminal, &mut state);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    )?;
    terminal.show_cursor()?;

    result
}

const AUTO_SAVE_DELAY: Duration = Duration::from_millis(500);
const NOTIFICATION_DURATION: Duration = Duration::from_secs(2);

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
) -> Result<()> {
    let mut last_edit: Option<Instant> = None;
    let mut notification_time: Option<Instant> = None;

    loop {
        // Auto-save after user stops typing
        if let Some(edit_time) = last_edit {
            if edit_time.elapsed() >= AUTO_SAVE_DELAY && state.has_modified() {
                state.save_all_modified()?;
                last_edit = None;
            }
        }

        // Clear notification after timeout
        if let Some(notif_time) = notification_time {
            if notif_time.elapsed() >= NOTIFICATION_DURATION {
                state.notification = None;
                notification_time = None;
            }
        }

        // Track notification timing
        if state.notification.is_some() && notification_time.is_none() {
            notification_time = Some(Instant::now());
        }

        // Clear leader buffer after timeout
        if let Some((buffer, started)) = state.leader_buffer.take() {
            if started.elapsed() < leader::LEADER_TIMEOUT {
                // Still valid, put it back
                state.leader_buffer = Some((buffer, started));
            } else {
                // Timed out - insert ':' and buffered chars as regular text
                if last_edit.is_none() {
                    state.push_undo();
                }
                state.doc_mut().insert_char(':');
                for c in buffer.chars() {
                    state.doc_mut().insert_char(c);
                }
                last_edit = Some(Instant::now());
            }
        }

        terminal.draw(|frame| {
            let area = frame.area();

            // Layout: editor + status bar
            let chunks = Layout::vertical([
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);

            // Update viewport height for scrolling
            state.viewport_height = chunks[0].height as usize;

            let doc = state.doc();

            // Editor
            let editor = EditorWidget::new(
                &doc.content,
                doc.cursor_line,
                doc.cursor_col,
            )
            .scroll_offset(doc.scroll_offset)
            .show_line_numbers(state.goto_input.is_some())
            .grid_style(state.grid_style)
            .text_align(state.text_align)
            .quote_seed(state.quote_seed.wrapping_add(state.current_doc as u64));

            frame.render_widget(editor, chunks[0]);

            // Status bar
            let leader_str = state.leader_buffer.as_ref().map(|(s, _)| s.as_str());
            let status = StatusBar::new(
                doc.modified,
                doc.cursor_line,
                doc.cursor_col,
                doc.line_count(),
            )
            .goto_input(state.goto_input.as_deref())
            .doc_indicator(state.current_doc + 1, state.documents.len())
            .notification(state.notification.as_deref())
            .leader_input(leader_str);
            frame.render_widget(status, chunks[1]);

            // Help overlay
            if state.show_help {
                frame.render_widget(HelpWidget::new(state.help_page), area);
            }

            // File picker overlay
            if state.show_file_picker {
                let names = state.document_names();
                frame.render_widget(
                    FilePickerWidget::new(&names, state.current_doc, state.picker_selection),
                    area,
                );
            }

            // Logo overlay
            if state.show_logo {
                frame.render_widget(LogoWidget::new(), area);
            }
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            let event = event::read()?;

            // Handle paste (for IME input)
            if let Event::Paste(text) = &event {
                if !state.show_file_picker && !state.show_help && state.goto_input.is_none() {
                    if last_edit.is_none() {
                        state.push_undo();
                    }
                    for c in text.chars() {
                        if c == '\n' {
                            let vh = state.viewport_height;
                            state.doc_mut().insert_newline();
                            state.doc_mut().adjust_scroll(vh);
                        } else {
                            state.doc_mut().insert_char(c);
                        }
                    }
                    last_edit = Some(Instant::now());
                }
                continue;
            }

            if let Event::Key(key) = event {
                // File picker mode
                if state.show_file_picker {
                    match key.code {
                        KeyCode::Esc => {
                            state.show_file_picker = false;
                        }
                        KeyCode::Up => {
                            if state.picker_selection > 0 {
                                state.picker_selection -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if state.picker_selection < state.documents.len() - 1 {
                                state.picker_selection += 1;
                            }
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                            let idx = c.to_digit(10).unwrap() as usize - 1;
                            if idx < state.documents.len() {
                                state.switch_to(idx);
                                state.show_file_picker = false;
                            }
                        }
                        KeyCode::Enter => {
                            state.switch_to(state.picker_selection);
                            state.show_file_picker = false;
                        }
                        _ => {}
                    }
                    continue;
                }

                // Help page navigation
                if state.show_help {
                    match key.code {
                        KeyCode::Esc => {
                            state.show_help = false;
                        }
                        KeyCode::Left => {
                            if state.help_page > 0 {
                                state.help_page -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if state.help_page < HelpWidget::TOTAL_PAGES - 1 {
                                state.help_page += 1;
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // Go to line input mode
                if state.goto_input.is_some() {
                    match key.code {
                        KeyCode::Esc => {
                            state.goto_input = None;
                        }
                        KeyCode::Enter => {
                            if let Some(ref input) = state.goto_input {
                                if let Ok(line_num) = input.parse::<usize>() {
                                    state.goto_line(line_num);
                                }
                            }
                            state.goto_input = None;
                        }
                        KeyCode::Backspace => {
                            if let Some(ref mut input) = state.goto_input {
                                input.pop();
                            }
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            if let Some(ref mut input) = state.goto_input {
                                input.push(c);
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                match (key.modifiers, key.code) {
                    // Quit
                    (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                        break;
                    }

                    // Save
                    (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                        state.save()?;
                    }

                    // Copy all to clipboard
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if clipboard.set_text(&state.doc().content).is_ok() {
                                state.notification = Some("Copied to clipboard".to_string());
                            }
                        }
                    }

                    // Format
                    (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
                        state.format()?;
                    }

                    // Toggle task
                    (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
                        state.doc_mut().toggle_current_task();
                    }

                    // New task
                    (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                        state.doc_mut().insert_task();
                    }

                    // Open file picker
                    (KeyModifiers::CONTROL, KeyCode::Char('o')) => {
                        state.picker_selection = state.current_doc;
                        state.show_file_picker = true;
                    }

                    // Go to line
                    (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
                        state.goto_input = Some(String::new());
                    }

                    // Help
                    (KeyModifiers::CONTROL, KeyCode::Char('h')) |
                    (_, KeyCode::F(1)) => {
                        state.show_help = !state.show_help;
                        if state.show_help {
                            state.help_page = 0;
                        }
                    }

                    // Logo
                    (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                        state.show_logo = !state.show_logo;
                    }

                    // Toggle grid style
                    (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
                        state.grid_style = state.grid_style.next();
                        state.notification = Some(state.grid_style.name().to_string());
                    }

                    // Toggle text alignment
                    (m, KeyCode::Char('b') | KeyCode::Char('B')) if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) => {
                        state.text_align = state.text_align.next();
                        state.notification = Some(state.text_align.name().to_string());
                    }

                    // Switch to previous document
                    (m, KeyCode::Left) if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) => {
                        if state.current_doc > 0 {
                            state.switch_to(state.current_doc - 1);
                        }
                    }

                    // Switch to next document
                    (m, KeyCode::Right) if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) => {
                        if state.current_doc < state.documents.len() - 1 {
                            state.switch_to(state.current_doc + 1);
                        }
                    }

                    // Clear document
                    (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                        state.clear();
                        last_edit = Some(Instant::now());
                    }

                    // Undo
                    (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
                        state.undo();
                        last_edit = Some(Instant::now());
                    }

                    // Close dialogs
                    (_, KeyCode::Esc) => {
                        state.show_help = false;
                        state.show_file_picker = false;
                        state.show_logo = false;
                    }

                    // Navigation
                    (_, KeyCode::Up) => {
                        let vh = state.viewport_height;
                        state.doc_mut().move_cursor_up();
                        state.doc_mut().adjust_scroll(vh);
                    }
                    (_, KeyCode::Down) => {
                        let vh = state.viewport_height;
                        state.doc_mut().move_cursor_down();
                        state.doc_mut().adjust_scroll(vh);
                    }
                    (_, KeyCode::Left) => {
                        state.doc_mut().move_cursor_left();
                    }
                    (_, KeyCode::Right) => {
                        state.doc_mut().move_cursor_right();
                    }
                    (_, KeyCode::Home) => {
                        state.doc_mut().move_cursor_line_start();
                    }
                    (_, KeyCode::End) => {
                        state.doc_mut().move_cursor_line_end();
                    }
                    (_, KeyCode::PageUp) => {
                        let vh = state.viewport_height;
                        state.doc_mut().page_up(vh);
                        state.doc_mut().adjust_scroll(vh);
                    }
                    (_, KeyCode::PageDown) => {
                        let vh = state.viewport_height;
                        state.doc_mut().page_down(vh);
                        state.doc_mut().adjust_scroll(vh);
                    }

                    // Text input
                    (_, KeyCode::Char(c)) => {
                        // Check leader key timeout
                        if let Some((buffer, started)) = state.leader_buffer.take() {
                            if started.elapsed() < leader::LEADER_TIMEOUT {
                                let mut new_buffer = buffer.clone();
                                new_buffer.push(c);

                                // Check for complete command
                                if let Some(cmd) = leader::match_command(&new_buffer) {
                                    match cmd {
                                        leader::LeaderCommand::DeleteLine => {
                                            state.push_undo();
                                            state.doc_mut().delete_line();
                                            last_edit = Some(Instant::now());
                                        }
                                        leader::LeaderCommand::YankLine => {
                                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                                let line = state.doc().current_line().to_string();
                                                if clipboard.set_text(&line).is_ok() {
                                                    state.notification = Some("Line yanked".to_string());
                                                }
                                            }
                                        }
                                        leader::LeaderCommand::Paste => {
                                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                                if let Ok(text) = clipboard.get_text() {
                                                    state.push_undo();
                                                    state.doc_mut().move_cursor_line_end();
                                                    state.doc_mut().insert_newline();
                                                    state.doc_mut().insert_str(&text);
                                                    last_edit = Some(Instant::now());
                                                }
                                            }
                                        }
                                        leader::LeaderCommand::Quit => {
                                            // Save session for next launch
                                            let _ = storage::save_session(&storage::Session {
                                                last_doc: state.current_doc,
                                                grid_style: state.grid_style.to_u8(),
                                                text_align: state.text_align.to_u8(),
                                            });
                                            return Ok(());
                                        }
                                        leader::LeaderCommand::SwitchDoc(idx) => {
                                            if idx < state.documents.len() {
                                                state.switch_to(idx);
                                            }
                                        }
                                    }
                                    continue;
                                }

                                // Check if still a valid prefix
                                if leader::is_prefix(&new_buffer) {
                                    state.leader_buffer = Some((new_buffer, started));
                                    continue;
                                }

                                // Invalid sequence - insert buffered chars + current
                                if last_edit.is_none() {
                                    state.push_undo();
                                }
                                state.doc_mut().insert_char(':');
                                for bc in buffer.chars() {
                                    state.doc_mut().insert_char(bc);
                                }
                                state.doc_mut().insert_char(c);
                                last_edit = Some(Instant::now());
                                continue;
                            } else {
                                // Timeout - insert buffered chars
                                if last_edit.is_none() {
                                    state.push_undo();
                                }
                                state.doc_mut().insert_char(':');
                                for bc in buffer.chars() {
                                    state.doc_mut().insert_char(bc);
                                }
                                // Fall through to handle current char
                            }
                        }

                        // Start leader sequence with ':'
                        if c == ':' {
                            state.leader_buffer = Some((String::new(), Instant::now()));
                            continue;
                        }

                        if last_edit.is_none() {
                            state.push_undo();
                        }
                        if state.config.auto_pair {
                            state.doc_mut().insert_char_with_auto_pair(c);
                        } else {
                            state.doc_mut().insert_char(c);
                        }
                        last_edit = Some(Instant::now());
                    }
                    (_, KeyCode::Enter) => {
                        if last_edit.is_none() {
                            state.push_undo();
                        }
                        let vh = state.viewport_height;
                        state.doc_mut().insert_newline();
                        state.doc_mut().adjust_scroll(vh);
                        last_edit = Some(Instant::now());
                    }
                    (_, KeyCode::Backspace) => {
                        if last_edit.is_none() {
                            state.push_undo();
                        }
                        state.doc_mut().delete_char_before();
                        last_edit = Some(Instant::now());
                    }
                    (_, KeyCode::Delete) => {
                        if last_edit.is_none() {
                            state.push_undo();
                        }
                        state.doc_mut().delete_char_after();
                        last_edit = Some(Instant::now());
                    }
                    // Shift+Tab: unindent any line with leading whitespace
                    // Handle both BackTab and Shift+Tab (some terminals send one or the other)
                    (_, KeyCode::BackTab) | (KeyModifiers::SHIFT, KeyCode::Tab) => {
                        if state.doc().has_leading_whitespace() {
                            if last_edit.is_none() {
                                state.push_undo();
                            }
                            state.doc_mut().unindent_line();
                            last_edit = Some(Instant::now());
                        }
                    }
                    // Tab: indent list line or insert spaces
                    (_, KeyCode::Tab) => {
                        if last_edit.is_none() {
                            state.push_undo();
                        }
                        if state.doc().is_list_line() {
                            state.doc_mut().indent_line();
                        } else {
                            state.doc_mut().insert_str("  ");
                        }
                        last_edit = Some(Instant::now());
                    }

                    _ => {}
                }
            }
        }
    }

    // Save session for next launch
    let _ = storage::save_session(&storage::Session {
        last_doc: state.current_doc,
        grid_style: state.grid_style.to_u8(),
        text_align: state.text_align.to_u8(),
    });

    Ok(())
}
