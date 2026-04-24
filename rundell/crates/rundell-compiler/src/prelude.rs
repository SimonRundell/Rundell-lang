//! The Rundell runtime prelude — embedded verbatim in every compiled output.

pub const PRELUDE: &str = r###"
// ============================================================
// Rundell Runtime Prelude  (auto-generated — do not edit)
// ============================================================
#![allow(
    unused_variables, unused_mut, dead_code, unused_imports,
    clippy::all, non_snake_case
)]

use serde_json::{json, Value as JsonValue};
use chrono::{
    DateTime, FixedOffset, Local, Utc,
    Datelike, Timelike,
    Duration as ChronoDuration,
};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;

// ── Value enum ──────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum RVal {
    Int(i64),
    Float(f64),
    Str(String),
    Currency(i64),
    Bool(bool),
    Json(JsonValue),
    DateTime(DateTime<FixedOffset>),
    Null,
}

impl RVal {
    fn type_name(&self) -> &'static str {
        match self {
            RVal::Int(_)      => "integer",
            RVal::Float(_)    => "float",
            RVal::Str(_)      => "string",
            RVal::Currency(_) => "currency",
            RVal::Bool(_)     => "boolean",
            RVal::Json(_)     => "json",
            RVal::DateTime(_) => "datetime",
            RVal::Null        => "null",
        }
    }

    pub fn to_display(&self) -> String {
        match self {
            RVal::Int(n) => n.to_string(),
            RVal::Float(f) => {
                let s = format!("{f}");
                if s.contains('.') || s.contains('e') || s.contains('E') { s }
                else { format!("{s}.0") }
            }
            RVal::Str(s)      => s.clone(),
            RVal::Currency(c) => {
                let whole = c / 100;
                let frac  = (c % 100).unsigned_abs();
                format!("{whole}.{frac:02}")
            }
            RVal::Bool(b)     => if *b { "true" } else { "false" }.to_string(),
            RVal::Json(v)     => v.to_string(),
            RVal::DateTime(dt)=> dt.to_rfc3339(),
            RVal::Null        => "null".to_string(),
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            RVal::Int(n)      => *n != 0,
            RVal::Float(f)    => *f != 0.0,
            RVal::Bool(b)     => *b,
            RVal::Currency(c) => *c != 0,
            RVal::Str(s)      => !s.is_empty(),
            RVal::Json(_)     => true,
            RVal::DateTime(_) => true,
            RVal::Null        => false,
        }
    }

    fn is_null(&self) -> bool { matches!(self, RVal::Null) }
}

// ── Arithmetic ───────────────────────────────────────────────
fn rval_add(a: RVal, b: RVal) -> RVal {
    match (a, b) {
        (RVal::Int(x),      RVal::Int(y))      => RVal::Int(x + y),
        (RVal::Float(x),    RVal::Float(y))    => RVal::Float(x + y),
        (RVal::Float(x),    RVal::Int(y))      => RVal::Float(x + y as f64),
        (RVal::Int(x),      RVal::Float(y))    => RVal::Float(x as f64 + y),
        (RVal::Currency(x), RVal::Currency(y)) => RVal::Currency(x + y),
        (RVal::DateTime(dt),RVal::Int(ms))     => RVal::DateTime(dt + ChronoDuration::milliseconds(ms)),
        (RVal::Str(a), b) => RVal::Str(a + &b.to_display()),
        (a, RVal::Str(b)) => RVal::Str(a.to_display() + &b),
        (a, b) => panic!("type error in +: {} + {}", a.type_name(), b.type_name()),
    }
}

fn rval_sub(a: RVal, b: RVal) -> RVal {
    match (a, b) {
        (RVal::Int(x),      RVal::Int(y))      => RVal::Int(x - y),
        (RVal::Float(x),    RVal::Float(y))    => RVal::Float(x - y),
        (RVal::Float(x),    RVal::Int(y))      => RVal::Float(x - y as f64),
        (RVal::Int(x),      RVal::Float(y))    => RVal::Float(x as f64 - y),
        (RVal::Currency(x), RVal::Currency(y)) => RVal::Currency(x - y),
        (RVal::DateTime(a), RVal::DateTime(b)) => RVal::Int(a.timestamp_millis() - b.timestamp_millis()),
        (RVal::DateTime(dt),RVal::Int(ms))     => RVal::DateTime(dt - ChronoDuration::milliseconds(ms)),
        (a, b) => panic!("type error in -: {} - {}", a.type_name(), b.type_name()),
    }
}

fn rval_mul(a: RVal, b: RVal) -> RVal {
    match (a, b) {
        (RVal::Int(x),      RVal::Int(y))   => RVal::Int(x * y),
        (RVal::Float(x),    RVal::Float(y)) => RVal::Float(x * y),
        (RVal::Float(x),    RVal::Int(y))   => RVal::Float(x * y as f64),
        (RVal::Int(x),      RVal::Float(y)) => RVal::Float(x as f64 * y),
        (RVal::Currency(x), RVal::Int(y))   => RVal::Currency(x * y),
        (RVal::Int(x),      RVal::Currency(y)) => RVal::Currency(x * y),
        (a, b) => panic!("type error in *: {} * {}", a.type_name(), b.type_name()),
    }
}

fn rval_div(a: RVal, b: RVal) -> RVal {
    match (a, b) {
        (RVal::Int(x), RVal::Int(y))       => { if y == 0 { panic!("division by zero") } RVal::Int(x / y) }
        (RVal::Float(x), RVal::Float(y))   => RVal::Float(x / y),
        (RVal::Float(x), RVal::Int(y))     => RVal::Float(x / y as f64),
        (RVal::Int(x),   RVal::Float(y))   => RVal::Float(x as f64 / y),
        (RVal::Currency(x), RVal::Int(y))  => { if y == 0 { panic!("division by zero") } RVal::Currency(x / y) }
        (a, b) => panic!("type error in /: {} / {}", a.type_name(), b.type_name()),
    }
}

fn rval_mod(a: RVal, b: RVal) -> RVal {
    match (a, b) {
        (RVal::Int(x), RVal::Int(y))     => { if y == 0 { panic!("modulo by zero") } RVal::Int(x % y) }
        (RVal::Float(x), RVal::Float(y)) => RVal::Float(x % y),
        (RVal::Float(x), RVal::Int(y))   => RVal::Float(x % y as f64),
        (RVal::Int(x),   RVal::Float(y)) => RVal::Float(x as f64 % y),
        (a, b) => panic!("type error in %: {} % {}", a.type_name(), b.type_name()),
    }
}

fn rval_pow(a: RVal, b: RVal) -> RVal {
    match (a, b) {
        (RVal::Int(x),   RVal::Int(y)) if y >= 0  => RVal::Int(x.pow(y as u32)),
        (RVal::Int(x),   RVal::Int(y))             => RVal::Float((x as f64).powi(y as i32)),
        (RVal::Float(x), RVal::Float(y))           => RVal::Float(x.powf(y)),
        (RVal::Float(x), RVal::Int(y))             => RVal::Float(x.powi(y as i32)),
        (RVal::Int(x),   RVal::Float(y))           => RVal::Float((x as f64).powf(y)),
        (a, b) => panic!("type error in **: {} ** {}", a.type_name(), b.type_name()),
    }
}

// ── Comparison ───────────────────────────────────────────────
fn rval_eq(a: &RVal, b: &RVal) -> bool {
    match (a, b) {
        (RVal::Int(x),      RVal::Int(y))      => x == y,
        (RVal::Float(x),    RVal::Float(y))    => x == y,
        (RVal::Float(x),    RVal::Int(y))      => *x == *y as f64,
        (RVal::Int(x),      RVal::Float(y))    => *x as f64 == *y,
        (RVal::Str(x),      RVal::Str(y))      => x == y,
        (RVal::Bool(x),     RVal::Bool(y))     => x == y,
        (RVal::Currency(x), RVal::Currency(y)) => x == y,
        (RVal::Null,        RVal::Null)        => true,
        _ => false,
    }
}

fn rval_lt(a: &RVal, b: &RVal) -> bool {
    match (a, b) {
        (RVal::Int(x),      RVal::Int(y))      => x < y,
        (RVal::Float(x),    RVal::Float(y))    => x < y,
        (RVal::Float(x),    RVal::Int(y))      => *x < *y as f64,
        (RVal::Int(x),      RVal::Float(y))    => (*x as f64) < *y,
        (RVal::Str(x),      RVal::Str(y))      => x < y,
        (RVal::Currency(x), RVal::Currency(y)) => x < y,
        (a, b) => panic!("type error in comparison: {} < {}", a.type_name(), b.type_name()),
    }
}

fn rval_lteq(a: &RVal, b: &RVal) -> bool { rval_lt(a, b) || rval_eq(a, b) }
fn rval_gt(a: &RVal, b: &RVal)   -> bool { !rval_lteq(a, b) }
fn rval_gteq(a: &RVal, b: &RVal) -> bool { !rval_lt(a, b) }
fn rval_neq(a: &RVal, b: &RVal)  -> bool { !rval_eq(a, b) }

// ── Collection access ─────────────────────────────────────────
fn json_to_rval(v: &JsonValue) -> RVal {
    match v {
        JsonValue::Null      => RVal::Null,
        JsonValue::Bool(b)   => RVal::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() { RVal::Int(i) }
            else { RVal::Float(n.as_f64().unwrap_or(0.0)) }
        }
        JsonValue::String(s) => RVal::Str(s.clone()),
        other                => RVal::Json(other.clone()),
    }
}

fn rval_to_json(v: RVal) -> JsonValue {
    match v {
        RVal::Null        => JsonValue::Null,
        RVal::Bool(b)     => JsonValue::Bool(b),
        RVal::Int(n)      => JsonValue::Number(n.into()),
        RVal::Float(f)    => serde_json::Number::from_f64(f)
                               .map(JsonValue::Number).unwrap_or(JsonValue::Null),
        RVal::Str(s)      => JsonValue::String(s),
        RVal::Json(v)     => v,
        RVal::Currency(c) => {
            let whole = c / 100; let frac = (c % 100).unsigned_abs();
            JsonValue::String(format!("{whole}.{frac:02}"))
        }
        RVal::DateTime(dt) => JsonValue::String(dt.to_rfc3339()),
    }
}

fn rval_index(col: RVal, key: RVal) -> RVal {
    match (col, key) {
        (RVal::Json(JsonValue::Array(arr)), RVal::Int(i)) => {
            let idx = if i < 0 { arr.len() as i64 + i } else { i } as usize;
            arr.get(idx).map(json_to_rval).unwrap_or(RVal::Null)
        }
        (RVal::Json(JsonValue::Object(map)), RVal::Str(k)) =>
            map.get(&k).map(json_to_rval).unwrap_or(RVal::Null),
        (RVal::Json(JsonValue::Array(arr)), RVal::Str(k)) => {
            if let Ok(i) = k.parse::<usize>() { arr.get(i).map(json_to_rval).unwrap_or(RVal::Null) }
            else { RVal::Null }
        }
        _ => panic!("index requires json collection"),
    }
}

fn rval_set_index(col: &mut RVal, key: RVal, val: RVal) {
    match col {
        RVal::Json(JsonValue::Object(map)) => {
            let k = match key { RVal::Str(s) => s, _ => panic!("object key must be string") };
            map.insert(k, rval_to_json(val));
        }
        RVal::Json(JsonValue::Array(arr)) => {
            let i = match key { RVal::Int(n) if n >= 0 => n as usize, _ => panic!("array index must be non-negative integer") };
            if i < arr.len() { arr[i] = rval_to_json(val); }
            else { panic!("array index {i} out of bounds"); }
        }
        _ => panic!("index assignment requires json collection"),
    }
}

// ── Built-in functions ────────────────────────────────────────
fn rnd_newline()  -> RVal { RVal::Str("\n".to_string()) }
fn rnd_string(v: RVal)  -> RVal { RVal::Str(v.to_display()) }
fn rnd_integer(v: RVal) -> RVal {
    match v {
        RVal::Int(n)      => RVal::Int(n),
        RVal::Float(f)    => RVal::Int(f as i64),
        RVal::Str(s)      => RVal::Int(s.trim().parse::<i64>().unwrap_or_else(|_| panic!("cannot convert '{}' to integer", s))),
        RVal::Bool(b)     => RVal::Int(if b { 1 } else { 0 }),
        RVal::Currency(c) => RVal::Int(c / 100),
        other => panic!("cannot convert {} to integer", other.type_name()),
    }
}
fn rnd_float(v: RVal) -> RVal {
    match v {
        RVal::Int(n)   => RVal::Float(n as f64),
        RVal::Float(f) => RVal::Float(f),
        RVal::Str(s)   => RVal::Float(s.trim().parse::<f64>().unwrap_or_else(|_| panic!("cannot convert '{}' to float", s))),
        RVal::Bool(b)  => RVal::Float(if b { 1.0 } else { 0.0 }),
        other => panic!("cannot convert {} to float", other.type_name()),
    }
}
fn rnd_boolean(v: RVal) -> RVal {
    match v {
        RVal::Bool(b) => RVal::Bool(b),
        RVal::Int(n)  => RVal::Bool(n != 0),
        RVal::Str(s)  => RVal::Bool(s == "true"),
        other => panic!("cannot convert {} to boolean", other.type_name()),
    }
}
fn rnd_cast(v: RVal, target: RVal) -> RVal {
    let t = match target { RVal::Str(s) => s, _ => panic!("cast() target must be string") };
    match t.as_str() {
        "integer"  => rnd_integer(v),
        "float"    => rnd_float(v),
        "string"   => rnd_string(v),
        "boolean"  => rnd_boolean(v),
        other => panic!("cast(): unknown type '{other}'"),
    }
}

fn rnd_length(v: RVal) -> RVal {
    match v {
        RVal::Str(s)                          => RVal::Int(s.chars().count() as i64),
        RVal::Json(JsonValue::Array(arr))     => RVal::Int(arr.len() as i64),
        RVal::Json(JsonValue::Object(map))    => RVal::Int(map.len() as i64),
        other => panic!("length() requires string or json, got {}", other.type_name()),
    }
}
fn rnd_abs(v: RVal) -> RVal {
    match v {
        RVal::Int(n)      => RVal::Int(n.abs()),
        RVal::Float(f)    => RVal::Float(f.abs()),
        RVal::Currency(c) => RVal::Currency(c.abs()),
        other => panic!("abs() requires numeric, got {}", other.type_name()),
    }
}
fn rnd_floor(v: RVal) -> RVal {
    match v {
        RVal::Float(f) => RVal::Int(f.floor() as i64),
        RVal::Int(n)   => RVal::Int(n),
        other => panic!("floor() requires numeric, got {}", other.type_name()),
    }
}
fn rnd_ceil(v: RVal) -> RVal {
    match v {
        RVal::Float(f) => RVal::Int(f.ceil() as i64),
        RVal::Int(n)   => RVal::Int(n),
        other => panic!("ceil() requires numeric, got {}", other.type_name()),
    }
}
fn rnd_round(v: RVal, dp: RVal) -> RVal {
    let f = match v { RVal::Float(f) => f, RVal::Int(n) => n as f64, other => panic!("round() first arg numeric, got {}", other.type_name()) };
    let d = match dp { RVal::Int(n) => n, other => panic!("round() second arg integer, got {}", other.type_name()) };
    let factor = 10_f64.powi(d as i32);
    RVal::Float((f * factor).round() / factor)
}
fn rnd_sqrt(v: RVal) -> RVal {
    let f = match v { RVal::Float(f) => f, RVal::Int(n) => n as f64, other => panic!("sqrt() requires numeric, got {}", other.type_name()) };
    if f < 0.0 { panic!("sqrt() requires non-negative number"); }
    RVal::Float(f.sqrt())
}
fn rnd_min(a: RVal, b: RVal) -> RVal { if rval_lt(&a, &b) { a } else { b } }
fn rnd_max(a: RVal, b: RVal) -> RVal { if rval_gt(&a, &b) { a } else { b } }
fn rnd_clamp(v: RVal, lo: RVal, hi: RVal) -> RVal {
    if rval_lt(&v, &lo) { lo } else if rval_gt(&v, &hi) { hi } else { v }
}
fn rnd_pow(a: RVal, b: RVal) -> RVal { rval_pow(a, b) }

fn rnd_upper(v: RVal) -> RVal {
    match v { RVal::Str(s) => RVal::Str(s.to_uppercase()), other => panic!("upper() requires string, got {}", other.type_name()) }
}
fn rnd_lower(v: RVal) -> RVal {
    match v { RVal::Str(s) => RVal::Str(s.to_lowercase()), other => panic!("lower() requires string, got {}", other.type_name()) }
}
fn rnd_trim(v: RVal) -> RVal {
    match v { RVal::Str(s) => RVal::Str(s.trim().to_string()), other => panic!("trim() requires string, got {}", other.type_name()) }
}
fn rnd_substr(s: RVal, start: RVal, len: RVal) -> RVal {
    let s = match s { RVal::Str(s) => s, other => panic!("substr() string arg, got {}", other.type_name()) };
    let st = match start { RVal::Int(n) => n as usize, other => panic!("substr() start integer, got {}", other.type_name()) };
    let ln = match len   { RVal::Int(n) => n as usize, other => panic!("substr() len integer, got {}", other.type_name()) };
    let chars: Vec<char> = s.chars().collect();
    let end = (st + ln).min(chars.len());
    RVal::Str(chars[st.min(chars.len())..end].iter().collect())
}
fn rnd_replace(s: RVal, find: RVal, repl: RVal) -> RVal {
    let s = match s    { RVal::Str(s) => s, other => panic!("replace() arg 1 string, got {}", other.type_name()) };
    let f = match find { RVal::Str(s) => s, other => panic!("replace() arg 2 string, got {}", other.type_name()) };
    let r = match repl { RVal::Str(s) => s, other => panic!("replace() arg 3 string, got {}", other.type_name()) };
    RVal::Str(s.replace(&f, &r))
}
fn rnd_split(s: RVal, delim: RVal) -> RVal {
    let s = match s     { RVal::Str(s) => s, other => panic!("split() arg 1 string, got {}", other.type_name()) };
    let d = match delim { RVal::Str(s) => s, other => panic!("split() arg 2 string, got {}", other.type_name()) };
    let parts: Vec<JsonValue> = s.split(&d).map(|p| JsonValue::String(p.to_string())).collect();
    RVal::Json(JsonValue::Array(parts))
}
fn rnd_join(arr: RVal, delim: RVal) -> RVal {
    let d = match delim { RVal::Str(s) => s, other => panic!("join() delimiter string, got {}", other.type_name()) };
    match arr {
        RVal::Json(JsonValue::Array(parts)) => {
            let strings: Vec<String> = parts.iter().map(|p| match p {
                JsonValue::String(s) => s.clone(), other => other.to_string(),
            }).collect();
            RVal::Str(strings.join(&d))
        }
        other => panic!("join() requires json array, got {}", other.type_name()),
    }
}
fn rnd_startswith(s: RVal, prefix: RVal) -> RVal {
    let s = match s      { RVal::Str(s) => s, other => panic!("startswith() arg 1 string, got {}", other.type_name()) };
    let p = match prefix { RVal::Str(s) => s, other => panic!("startswith() arg 2 string, got {}", other.type_name()) };
    RVal::Bool(s.starts_with(p.as_str()))
}
fn rnd_endswith(s: RVal, suffix: RVal) -> RVal {
    let s  = match s      { RVal::Str(s) => s, other => panic!("endswith() arg 1 string, got {}", other.type_name()) };
    let sf = match suffix { RVal::Str(s) => s, other => panic!("endswith() arg 2 string, got {}", other.type_name()) };
    RVal::Bool(s.ends_with(sf.as_str()))
}
fn rnd_contains(s: RVal, needle: RVal) -> RVal {
    let s = match s      { RVal::Str(s) => s, other => panic!("contains() arg 1 string, got {}", other.type_name()) };
    let n = match needle { RVal::Str(s) => s, other => panic!("contains() arg 2 string, got {}", other.type_name()) };
    RVal::Bool(s.contains(n.as_str()))
}
fn rnd_keys(v: RVal) -> RVal {
    match v {
        RVal::Json(JsonValue::Object(map)) => {
            let keys: Vec<JsonValue> = map.keys().map(|k| JsonValue::String(k.clone())).collect();
            RVal::Json(JsonValue::Array(keys))
        }
        other => panic!("keys() requires json object, got {}", other.type_name()),
    }
}
fn rnd_values(v: RVal) -> RVal {
    match v {
        RVal::Json(JsonValue::Object(map)) => {
            RVal::Json(JsonValue::Array(map.values().cloned().collect()))
        }
        other => panic!("values() requires json object, got {}", other.type_name()),
    }
}
fn rnd_has_key(v: RVal, key: RVal) -> RVal {
    let k = match key { RVal::Str(s) => s, other => panic!("has_key() key must be string, got {}", other.type_name()) };
    match v {
        RVal::Json(JsonValue::Object(map)) => RVal::Bool(map.contains_key(&k)),
        other => panic!("has_key() requires json object, got {}", other.type_name()),
    }
}
fn rnd_type_of(v: &RVal) -> RVal  { RVal::Str(v.type_name().to_string()) }
fn rnd_isnull(v: &RVal)  -> RVal  { RVal::Bool(v.is_null()) }

fn rnd_exists(path: RVal) -> RVal {
    let p = match path { RVal::Str(s) => s, _ => panic!("exists() requires string path") };
    RVal::Bool(Path::new(&p).exists())
}
fn rnd_delete(path: RVal) -> RVal {
    let p = match path { RVal::Str(s) => s, _ => panic!("delete() requires string path") };
    let path = Path::new(&p);
    if path.is_dir() { std::fs::remove_dir_all(path).unwrap_or_else(|e| panic!("delete() failed: {e}")); }
    else { std::fs::remove_file(path).unwrap_or_else(|e| panic!("delete() failed: {e}")); }
    RVal::Null
}
fn rnd_mkdir(path: RVal) -> RVal {
    let p = match path { RVal::Str(s) => s, _ => panic!("mkdir() requires string path") };
    std::fs::create_dir_all(&p).unwrap_or_else(|e| panic!("mkdir() failed: {e}"));
    RVal::Null
}
fn rnd_sleep(ms: RVal) -> RVal {
    let n = match ms { RVal::Int(n) if n >= 0 => n as u64, _ => panic!("sleep() requires non-negative integer") };
    std::thread::sleep(std::time::Duration::from_millis(n));
    RVal::Null
}
fn rnd_read_text(path: RVal) -> RVal {
    let p = match path { RVal::Str(s) => s, _ => panic!("read_text() requires string path") };
    RVal::Str(std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read_text() failed: {e}")))
}
fn rnd_write_text(path: RVal, content: RVal) -> RVal {
    let p = match path    { RVal::Str(s) => s, _ => panic!("write_text() path must be string") };
    let c = match content { RVal::Str(s) => s, _ => panic!("write_text() content must be string") };
    std::fs::write(&p, c.as_bytes()).unwrap_or_else(|e| panic!("write_text() failed: {e}"));
    RVal::Null
}
fn rnd_read_json(path: RVal) -> RVal {
    let p = match path { RVal::Str(s) => s, _ => panic!("read_json() requires string path") };
    let s = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read_json() failed: {e}"));
    RVal::Json(serde_json::from_str(&s).unwrap_or_else(|e| panic!("read_json() parse error: {e}")))
}
fn rnd_write_json(path: RVal, val: RVal) -> RVal {
    let p = match path { RVal::Str(s) => s, _ => panic!("write_json() path must be string") };
    let v = match val  { RVal::Json(v) => v, _ => panic!("write_json() value must be json") };
    let s = serde_json::to_string_pretty(&v).unwrap_or_else(|e| panic!("write_json() failed: {e}"));
    std::fs::write(&p, s.as_bytes()).unwrap_or_else(|e| panic!("write_json() write failed: {e}"));
    RVal::Null
}
fn rnd_read_csv(path: RVal, has_headers: RVal) -> RVal {
    let p  = match path        { RVal::Str(s)  => s, _ => panic!("read_csv() path must be string") };
    let hh = match has_headers { RVal::Bool(b) => b, _ => panic!("read_csv() has_headers must be boolean") };
    let mut reader = csv::ReaderBuilder::new().has_headers(hh).from_path(&p)
        .unwrap_or_else(|e| panic!("read_csv() failed: {e}"));
    let mut rows = Vec::new();
    if hh {
        let headers = reader.headers().unwrap_or_else(|e| panic!("read_csv() header read failed: {e}")).clone();
        for record in reader.records() {
            let record = record.unwrap_or_else(|e| panic!("read_csv() record read failed: {e}"));
            let mut obj = serde_json::Map::new();
            for (i, h) in headers.iter().enumerate() {
                obj.insert(h.to_string(), JsonValue::String(record.get(i).unwrap_or("").to_string()));
            }
            rows.push(JsonValue::Object(obj));
        }
    } else {
        for record in reader.records() {
            let record = record.unwrap_or_else(|e| panic!("read_csv() record read failed: {e}"));
            let arr: Vec<JsonValue> = record.iter().map(|v| JsonValue::String(v.to_string())).collect();
            rows.push(JsonValue::Array(arr));
        }
    }
    RVal::Json(JsonValue::Array(rows))
}
fn rnd_write_csv(path: RVal, rows: RVal, include_headers: RVal) -> RVal {
    let p  = match path            { RVal::Str(s)  => s, _ => panic!("write_csv() path must be string") };
    let ih = match include_headers { RVal::Bool(b) => b, _ => panic!("write_csv() include_headers must be boolean") };
    let rows = match rows { RVal::Json(JsonValue::Array(a)) => a, _ => panic!("write_csv() rows must be json array") };
    let mut writer = csv::WriterBuilder::new().has_headers(ih).from_path(&p)
        .unwrap_or_else(|e| panic!("write_csv() failed: {e}"));
    if ih {
        if let Some(JsonValue::Object(first)) = rows.first() {
            let headers: Vec<String> = first.keys().cloned().collect();
            writer.write_record(headers.iter()).unwrap();
            for row in &rows {
                if let JsonValue::Object(obj) = row {
                    let record: Vec<String> = headers.iter().map(|k| match obj.get(k) {
                        Some(JsonValue::String(s)) => s.clone(),
                        Some(v) => v.to_string(),
                        None => String::new(),
                    }).collect();
                    writer.write_record(record.iter()).unwrap();
                }
            }
        }
    } else {
        for row in &rows {
            if let JsonValue::Array(arr) = row {
                let record: Vec<String> = arr.iter().map(|v| match v {
                    JsonValue::String(s) => s.clone(), v => v.to_string(),
                }).collect();
                writer.write_record(record.iter()).unwrap();
            }
        }
    }
    writer.flush().unwrap();
    RVal::Null
}

// ── Datetime builtins ─────────────────────────────────────────
fn rnd_now() -> RVal {
    let now = Local::now();
    RVal::DateTime(now.with_timezone(now.offset()))
}
fn rnd_day(v: RVal)    -> RVal { match v { RVal::DateTime(dt) => RVal::Int(dt.day()    as i64), other => panic!("day() requires datetime, got {}", other.type_name()) } }
fn rnd_month(v: RVal)  -> RVal { match v { RVal::DateTime(dt) => RVal::Int(dt.month()  as i64), other => panic!("month() requires datetime, got {}", other.type_name()) } }
fn rnd_year(v: RVal)   -> RVal { match v { RVal::DateTime(dt) => RVal::Int(dt.year()   as i64), other => panic!("year() requires datetime, got {}", other.type_name()) } }
fn rnd_hour(v: RVal)   -> RVal { match v { RVal::DateTime(dt) => RVal::Int(dt.hour()   as i64), other => panic!("hour() requires datetime, got {}", other.type_name()) } }
fn rnd_minute(v: RVal) -> RVal { match v { RVal::DateTime(dt) => RVal::Int(dt.minute() as i64), other => panic!("minute() requires datetime, got {}", other.type_name()) } }
fn rnd_second(v: RVal) -> RVal { match v { RVal::DateTime(dt) => RVal::Int(dt.second() as i64), other => panic!("second() requires datetime, got {}", other.type_name()) } }
fn rnd_dateformat(fmt: RVal, dt: RVal) -> RVal {
    let fmt = match fmt { RVal::Str(s) => s, other => panic!("dateformat() arg 1 string, got {}", other.type_name()) };
    let dt  = match dt  { RVal::DateTime(dt) => dt, other => panic!("dateformat() arg 2 datetime, got {}", other.type_name()) };
    let pat = fmt
        .replace("YYYY", "%Y").replace("MM", "%m").replace("DD", "%d")
        .replace("HH", "%H").replace("mm", "%M").replace("SS", "%S")
        .replace("ZZ", "%:z").replace("Z", "%z");
    RVal::Str(dt.format(&pat).to_string())
}
fn rnd_timestamp(v: RVal) -> RVal {
    match v { RVal::DateTime(dt) => RVal::Int(dt.timestamp_millis()), other => panic!("timestamp() requires datetime, got {}", other.type_name()) }
}
fn rnd_fromtimestamp(v: RVal) -> RVal {
    let ms = match v { RVal::Int(n) => n, other => panic!("fromtimestamp() requires integer, got {}", other.type_name()) };
    let secs = ms.div_euclid(1_000);
    let nsec = (ms.rem_euclid(1_000) as u32) * 1_000_000;
    #[allow(deprecated)]
    let utc = DateTime::<Utc>::from_timestamp(secs, nsec).expect("fromtimestamp(): invalid timestamp");
    let offset = FixedOffset::east_opt(0).unwrap();
    RVal::DateTime(utc.with_timezone(&offset))
}
fn rnd_dayofweek(v: RVal) -> RVal {
    use chrono::Weekday;
    match v { RVal::DateTime(dt) => RVal::Int(dt.weekday().number_from_monday() as i64), other => panic!("dayofweek() requires datetime, got {}", other.type_name()) }
}
fn rnd_adddays(dt: RVal, days: RVal) -> RVal {
    let dt = match dt   { RVal::DateTime(dt) => dt, other => panic!("adddays() arg 1 datetime, got {}", other.type_name()) };
    let d  = match days { RVal::Int(n) => n,         other => panic!("adddays() arg 2 integer, got {}", other.type_name()) };
    RVal::DateTime(dt + ChronoDuration::days(d))
}
fn rnd_addhours(dt: RVal, hours: RVal) -> RVal {
    let dt = match dt    { RVal::DateTime(dt) => dt, other => panic!("addhours() arg 1 datetime, got {}", other.type_name()) };
    let h  = match hours { RVal::Int(n) => n,         other => panic!("addhours() arg 2 integer, got {}", other.type_name()) };
    RVal::DateTime(dt + ChronoDuration::hours(h))
}
fn rnd_diffdays(a: RVal, b: RVal) -> RVal {
    let a = match a { RVal::DateTime(dt) => dt, other => panic!("diffdays() arg 1 datetime, got {}", other.type_name()) };
    let b = match b { RVal::DateTime(dt) => dt, other => panic!("diffdays() arg 2 datetime, got {}", other.type_name()) };
    RVal::Int((a - b).num_days())
}
fn rnd_timezone(v: RVal) -> RVal {
    match v { RVal::DateTime(dt) => RVal::Str(dt.offset().to_string()), other => panic!("timezone() requires datetime, got {}", other.type_name()) }
}

// ── System ────────────────────────────────────────────────────
fn rnd_execute(path: RVal) -> RVal {
    let p = match path { RVal::Str(s) => s, _ => panic!("execute() requires string path") };
    let out = Command::new(&p).output().unwrap_or_else(|e| panic!("execute() failed for '{p}': {e}"));
    if !out.stdout.is_empty() { print!("{}", String::from_utf8_lossy(&out.stdout)); }
    if !out.stderr.is_empty() { eprint!("{}", String::from_utf8_lossy(&out.stderr)); }
    RVal::Null
}
fn rnd_os() -> RVal {
    let name = if cfg!(windows) { "windows" }
               else if cfg!(target_os = "macos") { "macos" }
               else if cfg!(target_os = "linux") { "linux" }
               else { "unknown" };
    RVal::Str(name.to_string())
}

// ── Debug output ──────────────────────────────────────────────
fn rnd_debug_stdout(msg: &str) {
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    print!("{ts}> {msg}");
    let _ = io::stdout().flush();
}
fn rnd_debug_file(path: &str, msg: &str) {
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let entry = format!("{ts}> {msg}");
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    std::fs::write(path, format!("{entry}{existing}")).unwrap_or_else(|e| panic!("debug file write failed: {e}"));
}

// ── Collection mutation ───────────────────────────────────────
fn rnd_append(col: &mut RVal, val: RVal) {
    match col {
        RVal::Json(JsonValue::Array(arr)) => arr.push(rval_to_json(val)),
        other => panic!("append() requires json array, got {}", other.type_name()),
    }
}
fn rnd_remove_key(col: &mut RVal, key: RVal) {
    match col {
        RVal::Json(JsonValue::Object(map)) => {
            let k = match key { RVal::Str(s) => s, _ => panic!("remove key must be string") };
            map.remove(&k);
        }
        RVal::Json(JsonValue::Array(arr)) => {
            let i = match key { RVal::Int(n) if n >= 0 => n as usize, _ => panic!("remove index must be non-negative integer") };
            if i < arr.len() { arr.remove(i); } else { panic!("remove: index {i} out of bounds"); }
        }
        other => panic!("remove requires json collection, got {}", other.type_name()),
    }
}
fn rnd_remove_at(col: &mut RVal, idx: RVal) {
    let i = match idx { RVal::Int(n) if n >= 0 => n as usize, _ => panic!("remove_at() index must be non-negative integer") };
    match col {
        RVal::Json(JsonValue::Array(arr)) => {
            if i < arr.len() { arr.remove(i); } else { panic!("remove_at(): index {i} out of bounds"); }
        }
        other => panic!("remove_at() requires json array, got {}", other.type_name()),
    }
}
"###;
