use anyhow::{Context, Result};
use pulldown_cmark::{Event, Options, Parser};
use pulldown_cmark_to_cmark::cmark;
use std::fs;
use std::path::Path;

pub fn format_markdown(content: &str) -> Result<String> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(content, options);
    let events: Vec<Event> = parser.collect();

    let mut formatted = String::new();
    cmark(events.iter().cloned(), &mut formatted)
        .context("Failed to format markdown")?;

    // Ensure trailing newline
    if !formatted.ends_with('\n') {
        formatted.push('\n');
    }

    Ok(formatted)
}

pub fn format_file(path: &Path, write: bool) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let formatted = format_markdown(&content)?;

    if write {
        fs::write(path, &formatted)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;
        println!("Formatted: {}", path.display());
    } else {
        print!("{}", formatted);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_markdown() {
        let content = "#   Title\n\n-  item 1\n-  item 2\n";
        let formatted = format_markdown(content).unwrap();
        assert!(formatted.contains("# Title"));
    }
}
