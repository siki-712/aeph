use std::time::Duration;

pub const LEADER_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderCommand {
    DeleteLine,
    YankLine,
    Paste,
}

pub fn match_command(input: &str) -> Option<LeaderCommand> {
    match input {
        "dd" => Some(LeaderCommand::DeleteLine),
        "yy" => Some(LeaderCommand::YankLine),
        "p" => Some(LeaderCommand::Paste),
        _ => None,
    }
}

pub fn is_prefix(input: &str) -> bool {
    ["d", "dd", "y", "yy", "p"].iter().any(|cmd| cmd.starts_with(input))
}
