// SPDX-License-Identifier: Apache-2.0

/// Parse zsh alias output into (name, command) pairs.
///
/// Zsh outputs:  name='value'
pub fn parse_alias_lines(output: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(pair) = parse_one_alias(line) {
            results.push(pair);
        }
    }
    results
}

fn parse_one_alias(line: &str) -> Option<(String, String)> {
    let eq_pos = line.find('=')?;
    let name = line[..eq_pos].trim().to_string();
    if name.is_empty() {
        return None;
    }
    let rest = &line[eq_pos + 1..];
    let command = unquote(rest);
    Some((name, command))
}

/// Remove surrounding quotes and handle escape sequences.
fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }

    // $'...' ANSI-C quoting
    if s.starts_with("$'") && s.ends_with('\'') && s.len() >= 4 {
        let inner = &s[2..s.len() - 1];
        return unescape_ansi_c(inner);
    }

    // '...' single quoting (no escapes except '' -> ')
    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        let inner = &s[1..s.len() - 1];
        return inner.replace("'\\''", "'");
    }

    // "..." double quoting
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        let inner = &s[1..s.len() - 1];
        return unescape_double(inner);
    }

    // No quotes
    s.to_string()
}

fn unescape_ansi_c(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') | None => out.push('\\'),
                Some('\'') => out.push('\''),
                Some('"') => out.push('"'),
                Some('a') => out.push('\x07'),
                Some('b') => out.push('\x08'),
                Some('e' | 'E') => out.push('\x1b'),
                Some('f') => out.push('\x0c'),
                Some('v') => out.push('\x0b'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn unescape_double(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some('$') => out.push('$'),
                Some('`') => out.push('`'),
                Some('\n') => {} // line continuation
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_alias() {
        let input = "ll=ls -la";
        let result = parse_alias_lines(input);
        assert_eq!(result, vec![("ll".into(), "ls -la".into())]);
    }

    #[test]
    fn test_single_quoted() {
        let input = "gst='git status'";
        let result = parse_alias_lines(input);
        assert_eq!(result, vec![("gst".into(), "git status".into())]);
    }

    #[test]
    fn test_ansi_c_quoting() {
        let input = "test=$'hello\\nworld'";
        let result = parse_alias_lines(input);
        assert_eq!(result, vec![("test".into(), "hello\nworld".into())]);
    }

    #[test]
    fn test_double_quoted() {
        let input = r#"foo="bar \"baz\"""#;
        let result = parse_alias_lines(input);
        assert_eq!(result, vec![("foo".into(), "bar \"baz\"".into())]);
    }

    #[test]
    fn test_embedded_equals() {
        let input = "gcmsg='git commit -m'";
        let result = parse_alias_lines(input);
        assert_eq!(result, vec![("gcmsg".into(), "git commit -m".into())]);
    }

    #[test]
    fn test_multiple_lines() {
        let input = "ga='git add'\ngst='git status'\nll='ls -la'";
        let result = parse_alias_lines(input);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "ga");
        assert_eq!(result[2].1, "ls -la");
    }

    #[test]
    fn test_empty_lines_skipped() {
        let input = "\n\nga='git add'\n\n";
        let result = parse_alias_lines(input);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_single_quote_escape() {
        let input = "test='it'\\''s working'";
        let result = parse_alias_lines(input);
        assert_eq!(result, vec![("test".into(), "it's working".into())]);
    }
}
