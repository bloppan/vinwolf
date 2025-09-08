#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

pub trait Serialize {
    fn to_value(&self) -> Value;
}

pub trait Deserialize: Sized {
    fn from_value(v: &Value) -> Result<Self, String>;
}

impl Serialize for Value {
    fn to_value(&self) -> Value { self.clone() }
}

impl Deserialize for Value {
    fn from_value(v: &Value) -> Result<Self, String> { Ok(v.clone()) }
}

impl Serialize for () {
    fn to_value(&self) -> Value { Value::Null }
}

impl Deserialize for () {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v { Value::Null => Ok(()), _ => Err("expected null".into()) }
    }
}

impl Serialize for bool {
    fn to_value(&self) -> Value { Value::Bool(*self) }
}

impl Deserialize for bool {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v { Value::Bool(b) => Ok(*b), _ => Err("expected bool".into()) }
    }
}

impl Serialize for String {
    fn to_value(&self) -> Value { Value::String(self.clone()) }
}

impl Serialize for &str {
    fn to_value(&self) -> Value { Value::String(self.to_string()) }
}

impl Deserialize for String {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v { Value::String(s) => Ok(s.clone()), _ => Err("expected string".into()) }
    }
}

macro_rules! num_impls {
    ($t:ty) => {
        impl Serialize for $t {
            fn to_value(&self) -> Value { Value::Number(*self as f64) }
        }
        impl Deserialize for $t {
            fn from_value(v: &Value) -> Result<Self, String> {
                match v {
                    Value::Number(n) => Ok(*n as $t),
                    _ => Err("expected number".into())
                }
            }
        }
    };
}
num_impls!(i64);
num_impls!(i32);
num_impls!(i16);
num_impls!(i8);
num_impls!(u64);
num_impls!(u32);
num_impls!(u16);
num_impls!(u8);
num_impls!(f64);
num_impls!(f32);
num_impls!(isize);
num_impls!(usize);

impl<T: Serialize> Serialize for Option<T> {
    fn to_value(&self) -> Value {
        match self {
            Some(x) => x.to_value(),
            None => Value::Null,
        }
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Null => Ok(None),
            _ => Ok(Some(T::from_value(v)?)),
        }
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn to_value(&self) -> Value {
        Value::Array(self.iter().map(|x| x.to_value()).collect())
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Array(a) => {
                let mut out = Vec::with_capacity(a.len());
                for el in a {
                    out.push(T::from_value(el)?);
                }
                Ok(out)
            }
            _ => Err("expected array".into()),
        }
    }
}

impl<T: Serialize> Serialize for HashMap<String, T> {
    fn to_value(&self) -> Value {
        let mut m = HashMap::with_capacity(self.len());
        for (k, v) in self {
            m.insert(k.clone(), v.to_value());
        }
        Value::Object(m)
    }
}

impl<T: Deserialize> Deserialize for HashMap<String, T> {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Object(o) => {
                let mut m = HashMap::with_capacity(o.len());
                for (k, vv) in o {
                    m.insert(k.clone(), T::from_value(vv)?);
                }
                Ok(m)
            }
            _ => Err("expected object".into()),
        }
    }
}

pub fn to_json_string<T: Serialize>(value: &T) -> String {
    stringify(&value.to_value())
}

pub fn from_json_str<T: Deserialize>(s: &str) -> Result<T, String> {
    let v = parse(s)?;
    T::from_value(&v)
}

pub fn stringify(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => if *b { "true".into() } else { "false".into() },
        Value::Number(n) => {
            if n.is_finite() {
                let s = format!("{}", n);
                if s.contains('.') || s.contains('e') || s.contains('E') { s } else { format!("{}.0", s) }
            } else { "null".into() }
        }
        Value::String(s) => quote_str(s),
        Value::Array(a) => {
            let mut out = String::from("[");
            let mut first = true;
            for el in a {
                if !first { out.push(','); }
                first = false;
                out.push_str(&stringify(el));
            }
            out.push(']');
            out
        }
        Value::Object(o) => {
            let mut out = String::from("{");
            let mut first = true;
            let mut entries: Vec<(&String, &Value)> = o.iter().collect();
            entries.sort_by(|a,b| a.0.cmp(b.0));
            for (k, vv) in entries {
                if !first { out.push(','); }
                first = false;
                out.push_str(&quote_str(k));
                out.push(':');
                out.push_str(&stringify(vv));
            }
            out.push('}');
            out
        }
    }
}

fn quote_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if c.is_control() => {
                let code = c as u32;
                out.push_str(&format!("\\u{:04X}", code));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn parse(s: &str) -> Result<Value, String> {
    let mut p = Parser { s: s.as_bytes(), i: 0, len: s.len() };
    let v = p.parse_value()?;
    p.skip_ws();
    if p.i != p.len { return Err("trailing characters".into()); }
    Ok(v)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    len: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> { if self.i < self.len { Some(self.s[self.i]) } else { None } }
    fn bump(&mut self) -> Option<u8> { let c = self.peek()?; self.i += 1; Some(c) }
    fn skip_ws(&mut self) { while let Some(c) = self.peek() { if c == b' ' || c == b'\n' || c == b'\r' || c == b'\t' { self.i += 1; } else { break; } } }

    fn parse_value(&mut self) -> Result<Value, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'n') => self.parse_null(),
            Some(b't') | Some(b'f') => self.parse_bool(),
            Some(b'-') | Some(b'0'..=b'9') => self.parse_number(),
            Some(b'"') => self.parse_string().map(Value::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            _ => Err("unexpected token".into()),
        }
    }

    fn parse_null(&mut self) -> Result<Value, String> {
        if self.take_bytes(b"null") { Ok(Value::Null) } else { Err("invalid null".into()) }
    }

    fn parse_bool(&mut self) -> Result<Value, String> {
        if self.take_bytes(b"true") { Ok(Value::Bool(true)) }
        else if self.take_bytes(b"false") { Ok(Value::Bool(false)) }
        else { Err("invalid bool".into()) }
    }

    fn parse_number(&mut self) -> Result<Value, String> {
        let start = self.i;
        if self.peek() == Some(b'-') { self.i += 1; }
        match self.peek() {
            Some(b'0') => { self.i += 1; }
            Some(b'1'..=b'9') => { self.i += 1; while matches!(self.peek(), Some(b'0'..=b'9')) { self.i += 1; } }
            _ => return Err("invalid number".into()),
        }
        if self.peek() == Some(b'.') {
            self.i += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) { return Err("invalid number".into()); }
            while matches!(self.peek(), Some(b'0'..=b'9')) { self.i += 1; }
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            self.i += 1;
            if matches!(self.peek(), Some(b'+') | Some(b'-')) { self.i += 1; }
            if !matches!(self.peek(), Some(b'0'..=b'9')) { return Err("invalid number".into()); }
            while matches!(self.peek(), Some(b'0'..=b'9')) { self.i += 1; }
        }
        let s = std::str::from_utf8(&self.s[start..self.i]).map_err(|_| "utf8".to_string())?;
        let n: f64 = s.parse().map_err(|_| "invalid number".to_string())?;
        Ok(Value::Number(n))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        if self.bump() != Some(b'"') { return Err("expected string".into()); }
        let mut out = String::new();
        while let Some(c) = self.bump() {
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = self.bump().ok_or("eof in escape")?;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let cp = self.parse_hex4()? as u32;
                            if 0xD800 <= cp && cp <= 0xDBFF {
                                if self.bump() != Some(b'\\') || self.bump() != Some(b'u') { return Err("invalid surrogate pair".into()); }
                                let cp2 = self.parse_hex4()? as u32;
                                if cp2 < 0xDC00 || cp2 > 0xDFFF { return Err("invalid surrogate pair".into()); }
                                let combined = 0x10000 + (((cp - 0xD800) << 10) | (cp2 - 0xDC00));
                                if let Some(ch) = std::char::from_u32(combined) { out.push(ch) } else { return Err("invalid codepoint".into()); }
                            } else {
                                if let Some(ch) = std::char::from_u32(cp) { out.push(ch) } else { return Err("invalid codepoint".into()); }
                            }
                        }
                        _ => return Err("invalid escape".into()),
                    }
                }
                _ => {
                    if c < 0x20 { return Err("control in string".into()); }
                    out.push(c as char)
                }
            }
        }
        Err("eof in string".into())
    }

    fn parse_hex4(&mut self) -> Result<u16, String> {
        let mut v: u16 = 0;
        for _ in 0..4 {
            let c = self.bump().ok_or("eof in hex")?;
            v <<= 4;
            v |= match c {
                b'0'..=b'9' => (c - b'0') as u16,
                b'a'..=b'f' => (c - b'a' + 10) as u16,
                b'A'..=b'F' => (c - b'A' + 10) as u16,
                _ => return Err("invalid hex".into()),
            };
        }
        Ok(v)
    }

    fn parse_array(&mut self) -> Result<Value, String> {
        if self.bump() != Some(b'[') { return Err("expected [".into()) }
        self.skip_ws();
        let mut arr = Vec::new();
        if self.peek() == Some(b']') { self.i += 1; return Ok(Value::Array(arr)); }
        loop {
            let v = self.parse_value()?;
            arr.push(v);
            self.skip_ws();
            match self.bump() {
                Some(b',') => { self.skip_ws(); }
                Some(b']') => break,
                _ => return Err("expected , or ]".into()),
            }
        }
        Ok(Value::Array(arr))
    }

    fn parse_object(&mut self) -> Result<Value, String> {
        if self.bump() != Some(b'{') { return Err("expected {".into()) }
        self.skip_ws();
        let mut obj = std::collections::HashMap::new();
        if self.peek() == Some(b'}') { self.i += 1; return Ok(Value::Object(obj)); }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            if self.bump() != Some(b':') { return Err("expected :".into()) }
            self.skip_ws();
            let val = self.parse_value()?;
            obj.insert(key, val);
            self.skip_ws();
            match self.bump() {
                Some(b',') => { self.skip_ws(); }
                Some(b'}') => break,
                _ => return Err("expected , or }".into()),
            }
        }
        Ok(Value::Object(obj))
    }

    fn take_bytes(&mut self, bytes: &[u8]) -> bool {
        if self.i + bytes.len() > self.len { return false; }
        if &self.s[self.i..self.i + bytes.len()] == bytes {
            self.i += bytes.len();
            true
        } else { false }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, PartialEq)]
    struct Person { name: String, age: u8, tags: Vec<String> }

    impl Serialize for Person {
        fn to_value(&self) -> Value {
            let mut m = HashMap::new();
            m.insert("name".into(), self.name.to_value());
            m.insert("age".into(), (self.age as u64).to_value());
            m.insert("tags".into(), self.tags.to_value());
            Value::Object(m)
        }
    }
    impl Deserialize for Person {
        fn from_value(v: &Value) -> Result<Self, String> {
            let o = match v { Value::Object(o) => o, _ => return Err("expected object".into()) };
            let name = String::from_value(o.get("name").ok_or("missing field name")?)?;
            let age: u8 = u8::from_value(o.get("age").ok_or("missing field age")?)?;
            let tags: Vec<String> = Vec::from_value(o.get("tags").ok_or("missing field tags")?)?;
            Ok(Person { name, age, tags })
        }
    }

    #[test]
    fn roundtrip() {
        let p = Person { name: "Ada".into(), age: 36, tags: vec!["math".into(), "cs".into()] };
        let s = to_json_string(&p);
        let p2: Person = from_json_str(&s).unwrap();
        println!("s: {:?}, p: {:?}", s, p);
        assert_eq!(p, p2);
    }
}
