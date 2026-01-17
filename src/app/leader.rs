use std::time::Duration;

pub const LEADER_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderCommand {
    DeleteLine,
}

pub fn match_command(input: &str) -> Option<LeaderCommand> {
    match input {
        "dd" => Some(LeaderCommand::DeleteLine),
        _ => None,
    }
}

pub fn is_prefix(input: &str) -> bool {
    ["d", "dd"].iter().any(|cmd| cmd.starts_with(input))
}
