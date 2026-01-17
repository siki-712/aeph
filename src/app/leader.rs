use std::time::Duration;

pub const LEADER_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderCommand {
    DeleteLine,
    YankLine,
    Paste,
    Quit,
    SwitchDoc(usize),
}

pub fn match_command(input: &str) -> Option<LeaderCommand> {
    match input {
        "dd" => Some(LeaderCommand::DeleteLine),
        "yy" => Some(LeaderCommand::YankLine),
        "p" => Some(LeaderCommand::Paste),
        "q" => Some(LeaderCommand::Quit),
        "1" => Some(LeaderCommand::SwitchDoc(0)),
        "2" => Some(LeaderCommand::SwitchDoc(1)),
        "3" => Some(LeaderCommand::SwitchDoc(2)),
        "4" => Some(LeaderCommand::SwitchDoc(3)),
        "5" => Some(LeaderCommand::SwitchDoc(4)),
        "6" => Some(LeaderCommand::SwitchDoc(5)),
        "7" => Some(LeaderCommand::SwitchDoc(6)),
        "8" => Some(LeaderCommand::SwitchDoc(7)),
        "9" => Some(LeaderCommand::SwitchDoc(8)),
        _ => None,
    }
}

pub fn is_prefix(input: &str) -> bool {
    ["d", "dd", "y", "yy", "p", "q"].iter().any(|cmd| cmd.starts_with(input))
}
