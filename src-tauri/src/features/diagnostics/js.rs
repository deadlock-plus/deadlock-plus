//! Just enough JavaScript lexing for heuristic scans: no parsing, no execution.

use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    /// Offset of the name in the stripped text.
    pub start: usize,
    /// From the opening `{` to just past the matching `}`.
    pub body: Range<usize>,
}

/// Blanks comments, string and regex contents with spaces. Length, offsets and newlines are
/// preserved so positions map straight back to the original.
pub fn strip(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = bytes.to_vec();
    let blank = |out: &mut [u8], range: Range<usize>| {
        for b in &mut out[range] {
            if *b != b'\n' && *b != b'\r' {
                *b = b' ';
            }
        }
    };

    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                let end = bytes[i..].iter().position(|&b| b == b'\n').map_or(bytes.len(), |p| i + p);
                blank(&mut out, i..end);
                i = end;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let end = find(bytes, i + 2, b"*/").map_or(bytes.len(), |p| p + 2);
                blank(&mut out, i..end);
                i = end;
            }
            q @ (b'"' | b'\'' | b'`') => {
                let end = string_end(bytes, i + 1, q);
                blank(&mut out, i + 1..end.saturating_sub(1).max(i + 1));
                i = end;
            }
            b'/' if regex_allowed(&out, i) => {
                let end = regex_end(bytes, i + 1);
                blank(&mut out, i + 1..end.saturating_sub(1).max(i + 1));
                i = end;
            }
            _ => i += 1,
        }
    }
    String::from_utf8(out).expect("only ASCII-delimited runs are replaced")
}

fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    bytes.get(from..)?.windows(needle.len()).position(|w| w == needle).map(|p| p + from)
}

/// Offset just past the closing quote, or the end of input if unterminated.
fn string_end(bytes: &[u8], mut i: usize, quote: u8) -> usize {
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'\n' if quote != b'`' => return i,
            b if b == quote => return i + 1,
            _ => i += 1,
        }
    }
    bytes.len()
}

fn regex_end(bytes: &[u8], mut i: usize) -> usize {
    let mut in_class = false;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'[' => in_class = true,
            b']' => in_class = false,
            b'/' if !in_class => return i + 1,
            b'\n' => return i,
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

/// A `/` starts a regex when the previous token cannot end an expression.
fn regex_allowed(stripped: &[u8], at: usize) -> bool {
    let before = &stripped[..at];
    let Some(last) = before.iter().rposition(|b| !b.is_ascii_whitespace()) else {
        return true;
    };
    if b"(,=:[!&|?{};+-*%<>~^".contains(&before[last]) {
        return true;
    }
    let word_start = before[..=last]
        .iter()
        .rposition(|b| !(b.is_ascii_alphanumeric() || *b == b'_' || *b == b'$'))
        .map_or(0, |p| p + 1);
    matches!(&before[word_start..=last], b"return" | b"typeof")
}

static DECLARATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfunction\s+([A-Za-z_$][\w$]*)\s*\(").unwrap());
static ASSIGNED_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Za-z_$][\w$]*)\s*[=:]\s*(?:async\s+)?function\b\s*(?:[A-Za-z_$][\w$]*)?\s*\(").unwrap()
});
static ARROW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Za-z_$][\w$]*)\s*=\s*(?:async\s+)?(?:\([^()]*\)|[A-Za-z_$][\w$]*)\s*=>\s*\{").unwrap()
});

/// Functions with a block body, sorted by position. Run on [`strip`] output. Nested functions
/// are reported too, with nested ranges. Arrows with an expression body are skipped.
pub fn functions(stripped: &str) -> Vec<Function> {
    let bytes = stripped.as_bytes();
    let mut found = Vec::new();

    for re in [&*DECLARATION, &*ASSIGNED_FUNCTION] {
        for caps in re.captures_iter(stripped) {
            let (name, whole) = (caps.get(1).unwrap(), caps.get(0).unwrap());
            let Some(close) = matching(bytes, whole.end() - 1, b'(', b')') else { continue };
            let Some(open) = next_non_space(bytes, close + 1).filter(|&p| bytes[p] == b'{') else {
                continue;
            };
            push(&mut found, bytes, name.as_str(), name.start(), open);
        }
    }
    for caps in ARROW.captures_iter(stripped) {
        let (name, whole) = (caps.get(1).unwrap(), caps.get(0).unwrap());
        push(&mut found, bytes, name.as_str(), name.start(), whole.end() - 1);
    }

    found.sort_by_key(|f| f.start);
    found.dedup_by_key(|f| f.body.start);
    found
}

fn push(found: &mut Vec<Function>, bytes: &[u8], name: &str, start: usize, open: usize) {
    if let Some(close) = matching(bytes, open, b'{', b'}') {
        found.push(Function { name: name.to_string(), start, body: open..close + 1 });
    }
}

fn next_non_space(bytes: &[u8], from: usize) -> Option<usize> {
    bytes.get(from..)?.iter().position(|b| !b.is_ascii_whitespace()).map(|p| p + from)
}

/// Index of the delimiter closing the one at `open_at`.
pub fn matching(bytes: &[u8], open_at: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0usize;
    for (i, &b) in bytes.iter().enumerate().skip(open_at) {
        if b == open {
            depth += 1;
        } else if b == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// 1-based line of a byte offset.
pub fn line_of(source: &str, offset: usize) -> usize {
    source.as_bytes()[..offset.min(source.len())].iter().filter(|&&b| b == b'\n').count() + 1
}

/// Trimmed text of the line containing a byte offset.
pub fn line_text(source: &str, offset: usize) -> String {
    let offset = offset.min(source.len());
    let start = source[..offset].rfind('\n').map_or(0, |i| i + 1);
    let end = source[offset..].find('\n').map_or(source.len(), |i| offset + i);
    source[start..end].trim().to_string()
}

#[cfg(test)]
mod tests;
