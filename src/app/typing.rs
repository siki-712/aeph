/// Bracket pair definitions (opening, closing)
pub const BRACKET_PAIRS: &[(char, char)] = &[
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
];

/// Same character centering definitions (char, count needed to trigger centering)
pub const SAME_CHAR_CENTER: &[(char, usize)] = &[
    ('`', 2),   // `` -> cursor in middle
    ('"', 2),   // "" -> cursor in middle
    ('\'', 2),  // '' -> cursor in middle
    ('*', 4),   // **** -> cursor in middle
    ('~', 4),   // ~~~~ -> cursor in middle
];

/// Get the closing bracket for an opening bracket
#[allow(dead_code)]
pub fn get_closing_bracket(c: char) -> Option<char> {
    BRACKET_PAIRS.iter()
        .find(|(open, _)| *open == c)
        .map(|(_, close)| *close)
}

/// Get the opening bracket for a closing bracket
pub fn get_opening_bracket(c: char) -> Option<char> {
    BRACKET_PAIRS.iter()
        .find(|(_, close)| *close == c)
        .map(|(open, _)| *open)
}

/// Check if cursor should be centered after typing a closing bracket.
/// Returns Some(move_back) if the char before cursor is the matching opening bracket.
pub fn should_center_for_bracket(char_before: Option<char>, c: char) -> Option<usize> {
    if let Some(opening) = get_opening_bracket(c) {
        if char_before == Some(opening) {
            return Some(1);
        }
    }
    None
}

/// Check if cursor should be centered after typing N consecutive same characters.
/// Returns Some(move_back) with the number of positions to move back.
pub fn should_center_for_same_char(chars_before: &str, c: char) -> Option<usize> {
    for &(target_char, count) in SAME_CHAR_CENTER {
        if c == target_char {
            // We need (count - 1) consecutive chars before cursor to trigger centering
            let needed = count - 1;
            if chars_before.len() >= needed {
                let suffix: String = chars_before.chars().rev().take(needed).collect();
                if suffix.chars().all(|ch| ch == target_char) {
                    // Move back by half the count
                    return Some(count / 2);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_closing_bracket() {
        assert_eq!(get_closing_bracket('('), Some(')'));
        assert_eq!(get_closing_bracket('['), Some(']'));
        assert_eq!(get_closing_bracket('{'), Some('}'));
        assert_eq!(get_closing_bracket('a'), None);
    }

    #[test]
    fn test_get_opening_bracket() {
        assert_eq!(get_opening_bracket(')'), Some('('));
        assert_eq!(get_opening_bracket(']'), Some('['));
        assert_eq!(get_opening_bracket('}'), Some('{'));
        assert_eq!(get_opening_bracket('a'), None);
    }

    #[test]
    fn test_should_center_for_bracket() {
        assert_eq!(should_center_for_bracket(Some('('), ')'), Some(1));
        assert_eq!(should_center_for_bracket(Some('['), ']'), Some(1));
        assert_eq!(should_center_for_bracket(Some('{'), '}'), Some(1));
        assert_eq!(should_center_for_bracket(Some('a'), ')'), None);
        assert_eq!(should_center_for_bracket(None, ')'), None);
    }

    #[test]
    fn test_should_center_for_same_char() {
        // Backtick: 2 chars for centering
        assert_eq!(should_center_for_same_char("`", '`'), Some(1));
        assert_eq!(should_center_for_same_char("", '`'), None);

        // Quote: 2 chars for centering
        assert_eq!(should_center_for_same_char("\"", '"'), Some(1));
        assert_eq!(should_center_for_same_char("", '"'), None);

        // Single quote: 2 chars for centering
        assert_eq!(should_center_for_same_char("'", '\''), Some(1));

        // Asterisk: 4 chars for centering
        assert_eq!(should_center_for_same_char("***", '*'), Some(2));
        assert_eq!(should_center_for_same_char("**", '*'), None);
        assert_eq!(should_center_for_same_char("*", '*'), None);

        // Tilde: 4 chars for centering
        assert_eq!(should_center_for_same_char("~~~", '~'), Some(2));
        assert_eq!(should_center_for_same_char("~~", '~'), None);
    }
}
