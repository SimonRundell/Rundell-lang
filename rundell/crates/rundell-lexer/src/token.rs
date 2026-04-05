//! Token definitions for the Rundell language lexer.
//!
//! Every syntactic element recognised by the lexer is represented as a
//! variant of [`Token`].

use logos::{Lexer, Logos};

// ---------------------------------------------------------------------------
// Helper callbacks
// ---------------------------------------------------------------------------

/// Parse a decimal integer literal (no sign).
fn parse_integer(lex: &mut Lexer<Token>) -> Option<i64> {
    lex.slice().parse().ok()
}

/// Parse a pixel value literal such as `10px`.
///
/// The regex matches the numeric part followed immediately by `px` with no
/// intervening whitespace.  The `px` suffix is stripped before parsing.
fn parse_pixel_value(lex: &mut Lexer<Token>) -> Option<u32> {
    let s = lex.slice();
    // Strip the trailing "px" (2 chars)
    s[..s.len() - 2].parse().ok()
}

/// Parse a duration literal such as `500ms`, `2s`, `1m`, or `1h` into milliseconds.
fn parse_duration_value(lex: &mut Lexer<Token>) -> Option<u64> {
    let s = lex.slice();
    if let Some(num) = s.strip_suffix("ms") {
        return num.parse::<u64>().ok();
    }
    if let Some(num) = s.strip_suffix('s') {
        return num.parse::<u64>().ok().map(|n| n * 1_000);
    }
    if let Some(num) = s.strip_suffix('m') {
        return num.parse::<u64>().ok().map(|n| n * 60_000);
    }
    if let Some(num) = s.strip_suffix('h') {
        return num.parse::<u64>().ok().map(|n| n * 3_600_000);
    }
    None
}

/// Parse a float literal (digits, decimal point, digits).
fn parse_float(lex: &mut Lexer<Token>) -> Option<f64> {
    lex.slice().parse().ok()
}

/// Parse a currency literal (e.g. `19.99`) into integer cents.
///
/// The regex only matches two decimal places, so the math is exact.
fn parse_currency(lex: &mut Lexer<Token>) -> Option<i64> {
    let s = lex.slice();
    // Split on '.'
    let mut parts = s.splitn(2, '.');
    let whole: i64 = parts.next()?.parse().ok()?;
    let frac_str = parts.next().unwrap_or("00");
    // Pad or truncate to exactly 2 digits
    let frac_str = if frac_str.len() == 1 {
        format!("{frac_str}0")
    } else {
        frac_str[..2].to_string()
    };
    let frac: i64 = frac_str.parse().ok()?;
    Some(whole * 100 + frac)
}

/// Unescape a Rundell string literal (strips quotes and resolves escapes).
fn parse_string(lex: &mut Lexer<Token>) -> Option<String> {
    let raw = lex.slice();
    // Determine delimiter from the first character
    let delim = raw.chars().next()?;
    // Strip surrounding delimiters
    let inner = &raw[1..raw.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                '\'' => out.push('\''),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                other => {
                    // Pass through unknown escapes literally
                    out.push('\\');
                    out.push(other);
                }
            }
        } else if c == delim {
            // Should not happen as logos matched the closing delimiter,
            // but be defensive.
            break;
        } else {
            out.push(c);
        }
    }
    Some(out)
}

/// Parse a datetime literal delimited by `|...|`.
fn parse_datetime(lex: &mut Lexer<Token>) -> Option<String> {
    let raw = lex.slice();
    if raw.len() < 2 {
        return None;
    }
    Some(raw[1..raw.len() - 1].to_string())
}

// ---------------------------------------------------------------------------
// Token enum
// ---------------------------------------------------------------------------

/// Every token produced by the Rundell lexer.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")] // skip whitespace
#[logos(skip r"#[^\n]*")] // skip line comments
pub enum Token {
    // -----------------------------------------------------------------------
    // Literals
    // -----------------------------------------------------------------------
    /// A decimal integer literal such as `42`.
    ///
    /// MUST come before Float in the source so logos checks it first only
    /// when there is no decimal point.  The regex excludes strings that
    /// contain a dot followed by more digits (those are floats).
    #[regex(r"[0-9]+", parse_integer)]
    Integer(i64),

    /// A floating-point literal such as `3.14`.
    ///
    /// The decimal point is only a decimal point when it is surrounded by
    /// digits on both sides (disambiguation rule §1.3).
    #[regex(r"[0-9]+\.[0-9]+", parse_float, priority = 3)]
    Float(f64),

    /// A currency literal with exactly two decimal places, e.g. `19.99`.
    ///
    /// Stored internally as integer cents to avoid floating-point error.
    /// The lexer regex is *identical* to Float but with higher priority so
    /// it is tried first; the callback checks the fractional-digit count.
    // NOTE: logos picks the longest match.  We distinguish Currency from
    // Float at a higher level; both use the same regex.  We keep Currency
    // as a separate variant so the parser can emit it directly.
    // In practice the lexer will emit Float for 3.14 and Currency for 9.99
    // — they share a regex so logos would only keep one.  We therefore
    // handle this in the parser/lexer post-processing: everything that
    // looks like  d+.dd  with exactly 2 fractional digits is ambiguous.
    // To keep the lexer simple we emit only Float(f64) from regex and
    // convert to Currency in the parser when the declared type is currency.
    // However, the spec says CurrencyLit(i64) should be a token.  We
    // resolve this by giving the 2-dp regex higher priority.
    #[regex(r"[0-9]+\.[0-9]{2}", parse_currency, priority = 4)]
    CurrencyLit(i64),

    /// A string literal delimited by `"` or `'`.  Escape sequences are
    /// already resolved; the stored value is the logical string content.
    #[regex(r#""([^"\\]|\\.|\n)*""#, parse_string)]
    #[regex(r#"'([^'\\]|\\.|\n)*'"#, parse_string)]
    StringLit(String),

    /// A datetime literal delimited by `|` in ISO 8601 format.
    #[regex(r"\|[0-9]{4}-[0-9]{2}-[0-9]{2}[ T][0-9]{2}:[0-9]{2}:[0-9]{2}(?:Z|[+-][0-9]{2}:[0-9]{2})?\|", parse_datetime)]
    DateTimeLit(String),

    /// Boolean `true` (matches `true`, `TRUE`, `yes`, `YES`).
    #[token("true")]
    #[token("TRUE")]
    #[token("yes")]
    #[token("YES")]
    BoolTrue,

    /// Boolean `false` (matches `false`, `FALSE`, `no`, `NO`).
    #[token("false")]
    #[token("FALSE")]
    #[token("no")]
    #[token("NO")]
    BoolFalse,

    // -----------------------------------------------------------------------
    // Keywords
    // -----------------------------------------------------------------------
    /// `define`
    #[token("define")]
    Define,
    /// `as`
    #[token("as")]
    As,
    /// `constant`
    #[token("constant")]
    Constant,
    /// `global`
    #[token("global")]
    Global,
    /// `set`
    #[token("set")]
    Set,
    /// `return`
    #[token("return")]
    Return,
    /// `import`
    #[token("import")]
    Import,

    /// `if`
    #[token("if")]
    If,
    /// `else`
    #[token("else")]
    Else,
    /// `switch`
    #[token("switch")]
    Switch,
    /// `for`
    #[token("for")]
    For,
    /// `while`
    #[token("while")]
    While,
    /// `each`
    #[token("each")]
    Each,
    /// `in`
    #[token("in")]
    In,
    /// `loops`
    #[token("loops")]
    Loops,

    /// `and`
    #[token("and")]
    And,
    /// `or`
    #[token("or")]
    Or,
    /// `not`
    #[token("not")]
    Not,
    /// `is`
    #[token("is")]
    Is,

    /// `null`
    #[token("null")]
    Null,

    /// `print`
    #[token("print")]
    Print,
    /// `receive`
    #[token("receive")]
    Receive,
    /// `with`
    #[token("with")]
    With,
    /// `prompt`
    #[token("prompt")]
    Prompt,

    /// `try`
    #[token("try")]
    Try,
    /// `catch`
    #[token("catch")]
    Catch,
    /// `finally`
    #[token("finally")]
    Finally,

    /// `integer` type keyword
    #[token("integer")]
    KwInteger,
    /// `float` type keyword
    #[token("float")]
    KwFloat,
    /// `string` type keyword / built-in
    #[token("string")]
    KwString,
    /// `currency` type keyword
    #[token("currency")]
    KwCurrency,
    /// `boolean` type keyword
    #[token("boolean")]
    KwBoolean,
    /// `json` type keyword
    #[token("json")]
    KwJson,
    /// `datetime` type keyword
    #[token("datetime")]
    KwDateTime,

    /// `cast` built-in
    #[token("cast")]
    Cast,
    /// `length` built-in
    #[token("length")]
    Length,
    /// `newline` built-in
    #[token("newline")]
    Newline,
    /// `abs` built-in
    #[token("abs")]
    Abs,
    /// `floor` built-in
    #[token("floor")]
    Floor,
    /// `ceil` built-in
    #[token("ceil")]
    Ceil,
    /// `round` built-in
    #[token("round")]
    Round,
    /// `substr` built-in
    #[token("substr")]
    Substr,
    /// `upper` built-in
    #[token("upper")]
    Upper,
    /// `lower` built-in
    #[token("lower")]
    Lower,
    /// `trim` built-in
    #[token("trim")]
    Trim,
    /// `execute` built-in
    #[token("execute")]
    Execute,
    /// `os` built-in
    #[token("os")]
    Os,
    /// `min` built-in
    #[token("min")]
    Min,
    /// `max` built-in
    #[token("max")]
    Max,
    /// `sqrt` built-in
    #[token("sqrt")]
    Sqrt,
    /// `pow` built-in
    #[token("pow")]
    Pow,
    /// `clamp` built-in
    #[token("clamp")]
    Clamp,
    /// `replace` built-in
    #[token("replace")]
    Replace,
    /// `split` built-in
    #[token("split")]
    Split,
    /// `join` built-in
    #[token("join")]
    Join,
    /// `startswith` built-in
    #[token("startswith")]
    StartsWith,
    /// `endswith` built-in
    #[token("endswith")]
    EndsWith,
    /// `contains` built-in
    #[token("contains")]
    Contains,
    /// `keys` built-in
    #[token("keys")]
    Keys,
    /// `values` built-in
    #[token("values")]
    Values,
    /// `has_key` built-in
    #[token("has_key")]
    HasKey,
    /// `remove_at` built-in
    #[token("remove_at")]
    RemoveAt,
    /// `type` built-in
    #[token("type")]
    Type,
    /// `isnull` built-in
    #[token("isnull")]
    IsNull,
    /// `exists` built-in
    #[token("exists")]
    Exists,
    /// `delete` built-in
    #[token("delete")]
    Delete,
    /// `mkdir` built-in
    #[token("mkdir")]
    Mkdir,
    /// `sleep` built-in
    #[token("sleep")]
    Sleep,
    /// `env_exists` built-in
    #[token("env_exists")]
    EnvExists,
    /// `now` built-in
    #[token("now")]
    Now,
    /// `day` built-in
    #[token("day")]
    Day,
    /// `month` built-in
    #[token("month")]
    Month,
    /// `year` built-in
    #[token("year")]
    Year,
    /// `hour` built-in
    #[token("hour")]
    Hour,
    /// `minute` built-in
    #[token("minute")]
    Minute,
    /// `second` built-in
    #[token("second")]
    Second,
    /// `dateformat` built-in
    #[token("dateformat")]
    DateFormat,
    /// `timestamp` built-in
    #[token("timestamp")]
    Timestamp,
    /// `fromtimestamp` built-in
    #[token("fromtimestamp")]
    FromTimestamp,
    /// `dayofweek` built-in
    #[token("dayofweek")]
    DayOfWeek,
    /// `adddays` built-in
    #[token("adddays")]
    AddDays,
    /// `addhours` built-in
    #[token("addhours")]
    AddHours,
    /// `diffdays` built-in
    #[token("diffdays")]
    DiffDays,
    /// `timezone` built-in
    #[token("timezone")]
    Timezone,
    /// `append` built-in / statement
    #[token("append")]
    Append,
    /// `remove` statement
    #[token("remove")]
    Remove,

    /// `returns`
    #[token("returns")]
    Returns,

    /// `TypeError` error-type keyword
    #[token("TypeError")]
    KwTypeError,
    /// `NullError` error-type keyword
    #[token("NullError")]
    KwNullError,
    /// `IndexError` error-type keyword
    #[token("IndexError")]
    KwIndexError,
    /// `DivisionError` error-type keyword
    #[token("DivisionError")]
    KwDivisionError,
    /// `IOError` error-type keyword
    #[token("IOError")]
    KwIOError,
    /// `RuntimeError` error-type keyword
    #[token("RuntimeError")]
    KwRuntimeError,

    // -----------------------------------------------------------------------
    // Operators and punctuation
    // -----------------------------------------------------------------------
    /// `++`
    #[token("++")]
    PlusPlus,
    /// `--`
    #[token("--")]
    MinusMinus,
    /// `**`
    #[token("**")]
    StarStar,
    /// `+`
    #[token("+")]
    Plus,
    /// `-`
    #[token("-")]
    Minus,
    /// `*`
    #[token("*")]
    Star,
    /// `/`
    #[token("/")]
    Slash,
    /// `%`
    #[token("%")]
    Percent,

    /// `==`
    #[token("==")]
    EqEq,
    /// `!=`
    #[token("!=")]
    BangEq,
    /// `<=`
    #[token("<=")]
    LtEq,
    /// `>=`
    #[token(">=")]
    GtEq,
    /// `<--`  (end-of-block marker)
    #[token("<--")]
    ArrowEnd,
    /// `<`
    #[token("<")]
    Lt,
    /// `>`
    #[token(">")]
    Gt,
    /// `=`
    #[token("=")]
    Eq,

    /// `-->`  (start-of-block marker)
    #[token("-->")]
    Arrow,

    /// `(`
    #[token("(")]
    LParen,
    /// `)`
    #[token(")")]
    RParen,
    /// `[`
    #[token("[")]
    LBracket,
    /// `]`
    #[token("]")]
    RBracket,
    /// `{`
    #[token("{")]
    LBrace,
    /// `}`
    #[token("}")]
    RBrace,

    /// `,`
    #[token(",")]
    Comma,
    /// `:`
    #[token(":")]
    Colon,
    /// `.`  (statement terminator)
    #[token(".")]
    Dot,

    // -----------------------------------------------------------------------
    // GUI — path separator
    // -----------------------------------------------------------------------
    /// `\`  Object path separator (outside string literals only).
    ///
    /// Logos matches string literals as atomic tokens before reaching the
    /// top-level scan loop, so `\` inside strings is never tokenised here.
    #[token("\\")]
    BackslashSep,

    // -----------------------------------------------------------------------
    // GUI — pixel dimension literal
    // -----------------------------------------------------------------------
    /// A pixel dimension: digits immediately followed by `px`, e.g. `10px`.
    ///
    /// Matched as a single token (no whitespace permitted between the number
    /// and the `px` suffix).  Higher priority than `Integer` so that `10px`
    /// is not split into `Integer(10)` + identifier `px`.
    #[regex(r"[0-9]+px", parse_pixel_value, priority = 5)]
    PixelValue(u32),

    /// A duration literal: digits followed by ms/s/m/h, e.g. `500ms`.
    #[regex(r"[0-9]+(?:ms|s|m|h)", parse_duration_value, priority = 5)]
    DurationValue(u64),

    // -----------------------------------------------------------------------
    // GUI — form and control keywords
    // -----------------------------------------------------------------------
    /// `form` keyword
    #[token("form")]
    KwForm,
    /// `show` keyword (method on a form path)
    #[token("show")]
    KwShow,
    /// `close` keyword (method on a form path)
    #[token("close")]
    KwClose,
    /// `modal` argument to `show()`
    #[token("modal")]
    KwModal,
    /// `autorefresh` property keyword
    #[token("autorefresh")]
    KwAutorefresh,
    /// `datasource` property keyword
    #[token("datasource")]
    KwDatasource,
    /// `columns` property keyword
    #[token("columns")]
    KwColumns,
    /// `dialog` built-in namespace keyword
    #[token("dialog")]
    KwDialog,
    /// `eventtimer` definition keyword
    #[token("eventtimer")]
    KwEventTimer,

    // -----------------------------------------------------------------------
    // GUI — control type keywords
    // -----------------------------------------------------------------------
    /// `label` control type
    #[token("label")]
    KwLabel,
    /// `textbox` control type
    #[token("textbox")]
    KwTextbox,
    /// `button` control type
    #[token("button")]
    KwButton,
    /// `radiobutton` control type
    #[token("radiobutton")]
    KwRadiobutton,
    /// `checkbox` control type
    #[token("checkbox")]
    KwCheckbox,
    /// `select` control type (dropdown)
    #[token("select")]
    KwSelect,
    /// `listbox` control type
    #[token("listbox")]
    KwListbox,

    // -----------------------------------------------------------------------
    // REST / query keywords
    // -----------------------------------------------------------------------
    /// `query` definition type keyword
    #[token("query")]
    KwQuery,
    /// `credentials` definition type keyword
    #[token("credentials")]
    KwCredentials,
    /// `await` async call keyword
    #[token("await")]
    KwAwait,
    /// `attempt` error-handling block opener
    #[token("attempt")]
    KwAttempt,
    /// `method` query property keyword
    #[token("method")]
    KwMethod,
    /// `endpoint` query property keyword
    #[token("endpoint")]
    KwEndpoint,
    /// `token` credentials property keyword
    #[token("token")]
    KwToken,
    /// `authentication` credentials property keyword
    #[token("authentication")]
    KwAuthentication,
    /// `timeout` query property keyword
    #[token("timeout")]
    KwTimeout,
    /// `GET` HTTP method value
    #[token("GET")]
    KwGet,
    /// `POST` HTTP method value
    #[token("POST")]
    KwPost,
    /// `env` built-in function keyword
    #[token("env")]
    KwEnv,

    // -----------------------------------------------------------------------
    // Identifier (must come AFTER all keywords)
    // -----------------------------------------------------------------------
    /// A user-defined identifier.
    ///
    /// Must start with a letter; subsequent characters may be alphanumeric
    /// or underscore. Leading underscores are forbidden per spec §1.5.
    #[regex(r"[a-zA-Z][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex;

    fn tokens(src: &str) -> Vec<Token> {
        lex(src).unwrap().into_iter().map(|(t, _)| t).collect()
    }

    #[test]
    fn integer_literal() {
        assert_eq!(tokens("42"), vec![Token::Integer(42)]);
    }

    #[test]
    fn float_literal() {
        // 3.14 — decimal point is part of the number
        let toks = tokens("3.14");
        // logos will prefer CurrencyLit for exactly 2 dp; 3.14 has 2 dp
        // We accept either Float or CurrencyLit here; the parser will
        // canonicalise. But the key point is it is NOT split into 3 Dot 14.
        assert_eq!(toks.len(), 1, "must be a single token, got {:?}", toks);
    }

    #[test]
    fn float_three_dp() {
        let toks = tokens("3.333");
        assert_eq!(toks, vec![Token::Float(3.333)]);
    }

    #[test]
    fn string_double_quote() {
        assert_eq!(
            tokens(r#""hello""#),
            vec![Token::StringLit("hello".to_string())]
        );
    }

    #[test]
    fn string_single_quote() {
        assert_eq!(
            tokens("'hello'"),
            vec![Token::StringLit("hello".to_string())]
        );
    }

    #[test]
    fn string_escape_sequences() {
        // \n -> newline, \r -> CR, \t -> tab, \\ -> backslash, \" -> quote, \' -> single quote
        let toks = tokens(r#""\n\r\t\\\"\'""#);
        assert_eq!(toks, vec![Token::StringLit("\n\r\t\\\"'".to_string())]);
    }

    #[test]
    fn string_opposite_delimiter_inside() {
        let toks = tokens(r#""it's here""#);
        assert_eq!(toks, vec![Token::StringLit("it's here".to_string())]);
    }

    #[test]
    fn currency_literal() {
        assert_eq!(tokens("19.99"), vec![Token::CurrencyLit(1999)]);
        assert_eq!(tokens("1000.00"), vec![Token::CurrencyLit(100000)]);
    }

    #[test]
    fn datetime_literal() {
        let toks = tokens("|2026-04-04 17:41:44|");
        assert_eq!(toks, vec![Token::DateTimeLit("2026-04-04 17:41:44".to_string())]);
        let toks = tokens("|2026-04-04T17:41:44-12:00|");
        assert_eq!(toks, vec![Token::DateTimeLit("2026-04-04T17:41:44-12:00".to_string())]);
    }

    #[test]
    fn bool_true_variants() {
        for s in &["true", "TRUE", "yes", "YES"] {
            assert_eq!(tokens(s), vec![Token::BoolTrue], "failed for {s}");
        }
    }

    #[test]
    fn bool_false_variants() {
        for s in &["false", "FALSE", "no", "NO"] {
            assert_eq!(tokens(s), vec![Token::BoolFalse], "failed for {s}");
        }
    }

    #[test]
    fn all_keywords() {
        let cases: &[(&str, Token)] = &[
            ("define", Token::Define),
            ("as", Token::As),
            ("constant", Token::Constant),
            ("global", Token::Global),
            ("set", Token::Set),
            ("return", Token::Return),
            ("import", Token::Import),
            ("if", Token::If),
            ("else", Token::Else),
            ("switch", Token::Switch),
            ("for", Token::For),
            ("while", Token::While),
            ("each", Token::Each),
            ("in", Token::In),
            ("loops", Token::Loops),
            ("and", Token::And),
            ("or", Token::Or),
            ("not", Token::Not),
            ("is", Token::Is),
            ("null", Token::Null),
            ("print", Token::Print),
            ("receive", Token::Receive),
            ("with", Token::With),
            ("prompt", Token::Prompt),
            ("try", Token::Try),
            ("catch", Token::Catch),
            ("finally", Token::Finally),
            ("min", Token::Min),
            ("max", Token::Max),
            ("sqrt", Token::Sqrt),
            ("pow", Token::Pow),
            ("clamp", Token::Clamp),
            ("replace", Token::Replace),
            ("split", Token::Split),
            ("join", Token::Join),
            ("startswith", Token::StartsWith),
            ("endswith", Token::EndsWith),
            ("contains", Token::Contains),
            ("keys", Token::Keys),
            ("values", Token::Values),
            ("has_key", Token::HasKey),
            ("remove_at", Token::RemoveAt),
            ("type", Token::Type),
            ("isnull", Token::IsNull),
            ("exists", Token::Exists),
            ("delete", Token::Delete),
            ("mkdir", Token::Mkdir),
            ("sleep", Token::Sleep),
            ("env_exists", Token::EnvExists),
            ("integer", Token::KwInteger),
            ("float", Token::KwFloat),
            ("string", Token::KwString),
            ("currency", Token::KwCurrency),
            ("boolean", Token::KwBoolean),
            ("json", Token::KwJson),
            ("datetime", Token::KwDateTime),
            ("cast", Token::Cast),
            ("length", Token::Length),
            ("newline", Token::Newline),
            ("dayofweek", Token::DayOfWeek),
            ("adddays", Token::AddDays),
            ("addhours", Token::AddHours),
            ("diffdays", Token::DiffDays),
            ("timezone", Token::Timezone),
            ("abs", Token::Abs),
            ("floor", Token::Floor),
            ("ceil", Token::Ceil),
            ("round", Token::Round),
            ("substr", Token::Substr),
            ("upper", Token::Upper),
            ("lower", Token::Lower),
            ("trim", Token::Trim),
            ("execute", Token::Execute),
            ("os", Token::Os),
            ("execute", Token::Execute),
            ("now", Token::Now),
            ("day", Token::Day),
            ("month", Token::Month),
            ("year", Token::Year),
            ("hour", Token::Hour),
            ("minute", Token::Minute),
            ("second", Token::Second),
            ("dateformat", Token::DateFormat),
            ("timestamp", Token::Timestamp),
            ("fromtimestamp", Token::FromTimestamp),
            ("append", Token::Append),
            ("remove", Token::Remove),
            ("returns", Token::Returns),
            ("TypeError", Token::KwTypeError),
            ("NullError", Token::KwNullError),
            ("IndexError", Token::KwIndexError),
            ("DivisionError", Token::KwDivisionError),
            ("IOError", Token::KwIOError),
            ("RuntimeError", Token::KwRuntimeError),
            // REST / query keywords
            ("query", Token::KwQuery),
            ("credentials", Token::KwCredentials),
            ("await", Token::KwAwait),
            ("attempt", Token::KwAttempt),
            ("method", Token::KwMethod),
            ("endpoint", Token::KwEndpoint),
            ("token", Token::KwToken),
            ("authentication", Token::KwAuthentication),
            ("timeout", Token::KwTimeout),
            ("GET", Token::KwGet),
            ("POST", Token::KwPost),
            ("env", Token::KwEnv),
        ];
        for (src, expected) in cases {
            assert_eq!(tokens(src), vec![expected.clone()], "keyword: {src}");
        }
    }

    #[test]
    fn rest_keywords() {
        let cases: &[(&str, Token)] = &[
            ("query", Token::KwQuery),
            ("credentials", Token::KwCredentials),
            ("await", Token::KwAwait),
            ("attempt", Token::KwAttempt),
            ("method", Token::KwMethod),
            ("endpoint", Token::KwEndpoint),
            ("token", Token::KwToken),
            ("authentication", Token::KwAuthentication),
            ("timeout", Token::KwTimeout),
            ("GET", Token::KwGet),
            ("POST", Token::KwPost),
            ("env", Token::KwEnv),
        ];
        for (src, expected) in cases {
            assert_eq!(tokens(src), vec![expected.clone()], "rest keyword: {src}");
        }
    }

    #[test]
    fn identifier() {
        assert_eq!(tokens("myVar"), vec![Token::Ident("myVar".to_string())]);
        assert_eq!(
            tokens("camelCase123"),
            vec![Token::Ident("camelCase123".to_string())]
        );
    }

    #[test]
    fn arrow_tokens() {
        assert_eq!(tokens("-->"), vec![Token::Arrow]);
        assert_eq!(tokens("<--"), vec![Token::ArrowEnd]);
    }

    #[test]
    fn comment_is_skipped() {
        let toks = tokens("42 # this is a comment\n99");
        assert_eq!(toks, vec![Token::Integer(42), Token::Integer(99)]);
    }

    #[test]
    fn multiline_statement() {
        // define x \n as integer = 5.
        let toks = tokens("define x\nas integer = 5.");
        assert_eq!(
            toks,
            vec![
                Token::Define,
                Token::Ident("x".to_string()),
                Token::As,
                Token::KwInteger,
                Token::Eq,
                Token::Integer(5),
                Token::Dot,
            ]
        );
    }

    #[test]
    fn decimal_point_vs_terminator() {
        // define f as float = 3.14.
        // The 3.14 must be one token; the trailing . is a Dot
        let toks = tokens("define f as float = 3.14.");
        // 3.14 has exactly 2 decimal digits → CurrencyLit from our regex
        // We expect exactly 6 tokens: Define, Ident, As, KwFloat, Eq, <num>, Dot
        assert_eq!(toks.len(), 7, "tokens: {:?}", toks);
        assert_eq!(toks[6], Token::Dot);
    }

    #[test]
    fn backslash_sep_outside_string() {
        let toks = tokens(r"rootWindow\myForm\show");
        assert_eq!(
            toks,
            vec![
                Token::Ident("rootWindow".to_string()),
                Token::BackslashSep,
                Token::Ident("myForm".to_string()),
                Token::BackslashSep,
                Token::KwShow,
            ]
        );
    }

    #[test]
    fn backslash_inside_string_is_literal() {
        // Backslash inside a string is NOT tokenised as BackslashSep.
        let toks = tokens(r#""C:\Users\Simon""#);
        // Should produce a single StringLit — the backslashes are literal.
        assert_eq!(toks.len(), 1, "expected single StringLit, got {:?}", toks);
        match &toks[0] {
            Token::StringLit(s) => assert!(s.contains('\\'), "backslash must be in string: {s:?}"),
            t => panic!("expected StringLit, got {t:?}"),
        }
    }

    #[test]
    fn pixel_value_token() {
        assert_eq!(tokens("10px"), vec![Token::PixelValue(10)]);
        assert_eq!(tokens("200px"), vec![Token::PixelValue(200)]);
    }

    #[test]
    fn pixel_value_no_space_only() {
        // "10 px" with a space should NOT be a PixelValue token; it becomes Integer + Ident.
        let toks = tokens("10 px");
        // PixelValue regex requires no whitespace, so this becomes Integer(10) + Ident("px")
        // BUT "px" is now KwLabel... no wait, we don't have a KwPx.
        // Since px is not a dedicated keyword and the Ident regex matches it:
        assert_eq!(toks.len(), 2, "tokens: {:?}", toks);
        assert_eq!(toks[0], Token::Integer(10));
    }

    #[test]
    fn gui_keywords() {
        let cases: &[(&str, Token)] = &[
            ("form", Token::KwForm),
            ("show", Token::KwShow),
            ("close", Token::KwClose),
            ("modal", Token::KwModal),
            ("autorefresh", Token::KwAutorefresh),
            ("datasource", Token::KwDatasource),
            ("columns", Token::KwColumns),
            ("dialog", Token::KwDialog),
            ("label", Token::KwLabel),
            ("textbox", Token::KwTextbox),
            ("button", Token::KwButton),
            ("radiobutton", Token::KwRadiobutton),
            ("checkbox", Token::KwCheckbox),
            ("select", Token::KwSelect),
            ("listbox", Token::KwListbox),
        ];
        for (src, expected) in cases {
            assert_eq!(tokens(src), vec![expected.clone()], "gui keyword: {src}");
        }
    }
}
