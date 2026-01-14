use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Task {
    pub line_number: usize,
    pub completed: bool,
    pub text: String,
    pub indent: usize,
}

pub fn parse_tasks(content: &str) -> Vec<Task> {
    content
        .lines()
        .enumerate()
        .filter_map(|(line_num, line)| {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with("- [ ] ") {
                Some(Task {
                    line_number: line_num,
                    completed: false,
                    text: trimmed[6..].to_string(),
                    indent,
                })
            } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
                Some(Task {
                    line_number: line_num,
                    completed: true,
                    text: trimmed[6..].to_string(),
                    indent,
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn list_tasks(path: &Path, pending_only: bool) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let tasks = parse_tasks(&content);
    let filtered: Vec<_> = if pending_only {
        tasks.iter().filter(|t| !t.completed).collect()
    } else {
        tasks.iter().collect()
    };

    if filtered.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    for (i, task) in filtered.iter().enumerate() {
        let marker = if task.completed { "✓" } else { "○" };
        let indent = "  ".repeat(task.indent / 2);
        println!("{:>3}. {} {}{}", i + 1, marker, indent, task.text);
    }

    let completed = tasks.iter().filter(|t| t.completed).count();
    println!("\n{}/{} tasks completed", completed, tasks.len());

    Ok(())
}

pub fn add_task(path: &Path, text: &str) -> Result<()> {
    let content = if path.exists() {
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?
    } else {
        String::new()
    };

    let new_task = format!("- [ ] {}", text);
    let new_content = if content.is_empty() {
        new_task
    } else if content.ends_with('\n') {
        format!("{}{}\n", content, new_task)
    } else {
        format!("{}\n{}\n", content, new_task)
    };

    fs::write(path, new_content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    println!("Added: {}", text);
    Ok(())
}

pub fn toggle_task(path: &Path, number: usize) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let tasks = parse_tasks(&content);

    if number == 0 || number > tasks.len() {
        anyhow::bail!("Invalid task number: {}. Valid range: 1-{}", number, tasks.len());
    }

    let task = &tasks[number - 1];
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

    let line = &lines[task.line_number];
    let new_line = if task.completed {
        line.replacen("- [x] ", "- [ ] ", 1)
            .replacen("- [X] ", "- [ ] ", 1)
    } else {
        line.replacen("- [ ] ", "- [x] ", 1)
    };

    new_lines[task.line_number] = new_line;

    let new_content = new_lines.join("\n") + "\n";
    fs::write(path, new_content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    let status = if task.completed {
        "uncompleted"
    } else {
        "completed"
    };
    println!("Task {} marked as {}: {}", number, status, task.text);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tasks() {
        let content = "# Todo\n- [ ] Task 1\n- [x] Task 2\n  - [ ] Subtask\n";
        let tasks = parse_tasks(content);

        assert_eq!(tasks.len(), 3);
        assert!(!tasks[0].completed);
        assert_eq!(tasks[0].text, "Task 1");
        assert!(tasks[1].completed);
        assert_eq!(tasks[2].indent, 2);
    }
}
