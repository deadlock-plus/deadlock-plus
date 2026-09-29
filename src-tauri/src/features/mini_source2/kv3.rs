//! Minimal text KV3 (`.vdata`) reader and writer.
//!
//! Strings have no escape sequences: a plain string ends at the next `"`, and `"""` opens a
//! multiline string. The writer picks `"""` for any string holding a quote or newline, so a
//! string containing `"""` itself cannot be written back faithfully.

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Kv3Error {
    #[error("{message} at byte {at}")]
    Syntax { message: &'static str, at: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    /// Ordered, and may repeat a key.
    Object(Vec<(String, Value)>),
    /// A value prefixed with a type flag, such as `resource:"path"`.
    Flagged(String, Box<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub header: Option<String>,
    pub root: Value,
}

pub const GENERIC_HEADER: &str = "<!-- kv3 encoding:text:version{e21c7f3c-8a33-41c5-9977-a76d3a32aa0d} format:generic:version{7412167c-06e9-4698-aff2-e63eb59037e7} -->";

impl Value {
    /// The first value stored under `key`, if this is an object.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

type Result<T> = std::result::Result<T, Kv3Error>;

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')
}

impl<'a> Parser<'a> {
    fn err<T>(&self, message: &'static str) -> Result<T> {
        Err(Kv3Error::Syntax { message, at: self.pos })
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    /// Whitespace, commas and both comment styles.
    fn skip(&mut self) -> Result<()> {
        loop {
            let rest = self.rest();
            let trimmed = rest.trim_start_matches(|c: char| c.is_whitespace() || c == ',');
            self.pos += rest.len() - trimmed.len();
            if trimmed.starts_with("//") {
                self.pos += trimmed.find('\n').unwrap_or(trimmed.len());
            } else if trimmed.starts_with("/*") {
                match trimmed.find("*/") {
                    Some(end) => self.pos += end + 2,
                    None => return self.err("unterminated comment"),
                }
            } else {
                return Ok(());
            }
        }
    }

    fn eat(&mut self, prefix: &str) -> bool {
        let hit = self.rest().starts_with(prefix);
        if hit {
            self.pos += prefix.len();
        }
        hit
    }

    fn word(&mut self) -> &'a str {
        let rest = self.rest();
        let len = rest.find(|c: char| !is_word(c)).unwrap_or(rest.len());
        self.pos += len;
        &rest[..len]
    }

    fn string(&mut self) -> Result<String> {
        let multiline = self.eat("\"\"\"");
        if !multiline && !self.eat("\"") {
            return self.err("expected a string");
        }
        let close = if multiline { "\"\"\"" } else { "\"" };
        let Some(mut len) = self.rest().find(close) else {
            return self.err("unterminated string");
        };
        if multiline {
            // A body ending in `"` puts four or more quotes in a row; the last three close it.
            while self.rest()[len + 3..].starts_with('"') {
                len += 1;
            }
        }
        let mut body = &self.rest()[..len];
        self.pos += len + close.len();
        if multiline {
            body = body.strip_prefix("\r\n").or_else(|| body.strip_prefix('\n')).unwrap_or(body);
        }
        Ok(body.to_string())
    }

    fn value(&mut self) -> Result<Value> {
        self.skip()?;
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => self.string().map(Value::String),
            Some(c) if c.is_ascii_digit() || matches!(c, '-' | '+') => self.number(),
            Some(c) if is_word(c) => {
                let word = self.word();
                if self.eat(":") {
                    return Ok(Value::Flagged(word.to_string(), Box::new(self.value()?)));
                }
                match word {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    "null" => Ok(Value::Null),
                    _ => self.err("unexpected bare word"),
                }
            }
            _ => self.err("expected a value"),
        }
    }

    fn number(&mut self) -> Result<Value> {
        let rest = self.rest();
        let len = rest
            .find(|c: char| !(c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')))
            .unwrap_or(rest.len());
        let text = &rest[..len];
        let parsed = if text.contains(['.', 'e', 'E']) {
            text.parse().map(Value::Float).ok()
        } else {
            text.parse().map(Value::Int).ok()
        };
        match parsed {
            Some(v) => {
                self.pos += len;
                Ok(v)
            }
            None => self.err("invalid number"),
        }
    }

    fn array(&mut self) -> Result<Value> {
        self.pos += 1;
        let mut items = Vec::new();
        loop {
            self.skip()?;
            if self.eat("]") {
                return Ok(Value::Array(items));
            }
            if self.peek().is_none() {
                return self.err("unterminated array");
            }
            items.push(self.value()?);
        }
    }

    fn object(&mut self) -> Result<Value> {
        self.pos += 1;
        let mut pairs = Vec::new();
        loop {
            self.skip()?;
            if self.eat("}") {
                return Ok(Value::Object(pairs));
            }
            let key = if self.peek() == Some('"') {
                self.string()?
            } else {
                match self.word() {
                    "" => return self.err("expected a key"),
                    word => word.to_string(),
                }
            };
            self.skip()?;
            if !self.eat("=") {
                return self.err("expected `=`");
            }
            pairs.push((key, self.value()?));
        }
    }
}

pub fn parse(src: &str) -> Result<Document> {
    let mut p = Parser { src, pos: 0 };
    p.skip()?;
    let mut header = None;
    if p.rest().starts_with("<!--") {
        let Some(end) = p.rest().find("-->") else {
            return p.err("unterminated header");
        };
        header = Some(p.rest()[..end + 3].to_string());
        p.pos += end + 3;
    }
    let root = p.value()?;
    p.skip()?;
    if p.pos != src.len() {
        return p.err("unexpected trailing content");
    }
    Ok(Document { header, root })
}

pub fn write(doc: &Document) -> String {
    let mut out = String::new();
    if let Some(header) = &doc.header {
        out.push_str(header);
        out.push('\n');
    }
    write_value(&doc.root, 0, &mut out);
    out.push('\n');
    out
}

fn indent(depth: usize, out: &mut String) {
    out.extend(std::iter::repeat('\t').take(depth));
}

fn write_string(s: &str, out: &mut String) {
    if s.contains(['"', '\n']) {
        out.push_str("\"\"\"");
        out.push_str(s);
        out.push_str("\"\"\"");
    } else {
        out.push('"');
        out.push_str(s);
        out.push('"');
    }
}

fn write_value(value: &Value, depth: usize, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(i) => out.push_str(&i.to_string()),
        Value::Float(f) => out.push_str(&format!("{f:?}")),
        Value::String(s) => write_string(s, out),
        Value::Flagged(flag, inner) => {
            out.push_str(flag);
            out.push(':');
            write_value(inner, depth, out);
        }
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            out.push_str("[\n");
            for item in items {
                indent(depth + 1, out);
                write_value(item, depth + 1, out);
                out.push_str(",\n");
            }
            indent(depth, out);
            out.push(']');
        }
        Value::Object(pairs) => {
            out.push_str("{\n");
            for (key, item) in pairs {
                indent(depth + 1, out);
                if !key.is_empty() && key.chars().all(is_word) {
                    out.push_str(key);
                } else {
                    write_string(key, out);
                }
                out.push_str(" = ");
                write_value(item, depth + 1, out);
                out.push('\n');
            }
            indent(depth, out);
            out.push('}');
        }
    }
}

#[cfg(test)]
mod tests;
