//! Parse error formatting helpers for human-friendly diagnostics.

use crate::ParseError;

/// Format a parse error with line/column and a caret indicator when possible.
pub fn format_parse_error(source: &str, err: &ParseError) -> String {
    match err {
        ParseError::UnexpectedToken { pos, .. } => {
            let (line_num, col_num, line_text) = line_col_at(source, *pos);
            let caret_pad = if col_num > 0 { col_num - 1 } else { 0 };
            let caret_line = format!("{:width$}^---ERROR HERE", "", width = caret_pad);
            format!(
                "Parse error: {err}\nERROR: {line_text}\n{caret_line}\n(at line {line_num}, column {col_num})"
            )
        }
        _ => format!("Parse error: {err}"),
    }
}

fn line_col_at(source: &str, pos: usize) -> (usize, usize, String) {
    let safe_pos = pos.min(source.len());
    let prefix = &source[..safe_pos];
    let line_num = prefix.lines().count().max(1);
    let col_num = prefix
        .rsplit('\n')
        .next()
        .map(|s| s.chars().count() + 1)
        .unwrap_or(1);
    let line_text = source
        .lines()
        .nth(line_num - 1)
        .unwrap_or("")
        .to_string();
    (line_num, col_num, line_text)
}

#[cfg(test)]
mod tests {
    use super::format_parse_error;
    use crate::parse;

    #[test]
    fn format_unexpected_token_with_caret() {
        let src = "define x as integer returns.";
        let err = parse(src).expect_err("expected parse error");
        let msg = format_parse_error(src, &err);

        assert!(msg.contains("ERROR: define x as integer returns."));
        assert!(msg.contains("^---ERROR HERE"));
        assert!(msg.contains("line 1"));
    }
}
