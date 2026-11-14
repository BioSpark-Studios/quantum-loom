//! Parameter bus: the single address → value store every control writes to and
//! every screen, route and output adapter reads from. The core only ever sees
//! addresses and numbers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Num(f32),
    Steps(Vec<bool>),
    Curve(Vec<[f32; 2]>),
    Text(String),
}

impl Value {
    /// Numeric view used by routes and the core frame: booleans read as 0 / 1.
    pub fn as_num(&self) -> Option<f32> {
        match self {
            Value::Num(v) => Some(*v),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Bus {
    v: HashMap<String, Value>,
    /// Bumped on every write; lets observers skip work when nothing changed.
    pub rev: u64,
}

impl Bus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.v.get(key)
    }

    pub fn has(&self, key: &str) -> bool {
        self.v.contains_key(key)
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.rev += 1;
        self.v.insert(key.to_owned(), value);
    }

    /// Seed a default without overwriting an existing value.
    pub fn init(&mut self, key: &str, value: Value) {
        if !self.v.contains_key(key) {
            self.v.insert(key.to_owned(), value);
        }
    }

    pub fn num(&self, key: &str, default: f32) -> f32 {
        self.v.get(key).and_then(Value::as_num).unwrap_or(default)
    }

    pub fn flag(&self, key: &str, default: bool) -> bool {
        match self.v.get(key) {
            Some(Value::Bool(b)) => *b,
            Some(Value::Num(n)) => *n > 0.5,
            _ => default,
        }
    }

    pub fn set_num(&mut self, key: &str, v: f32) {
        self.set(key, Value::Num(v));
    }

    pub fn set_flag(&mut self, key: &str, v: bool) {
        self.set(key, Value::Bool(v));
    }

    pub fn toggle(&mut self, key: &str, default: bool) -> bool {
        let v = !self.flag(key, default);
        self.set_flag(key, v);
        v
    }

    pub fn steps(&self, key: &str) -> Option<&[bool]> {
        match self.v.get(key) {
            Some(Value::Steps(s)) => Some(s),
            _ => None,
        }
    }

    pub fn curve(&self, key: &str) -> Option<&[[f32; 2]]> {
        match self.v.get(key) {
            Some(Value::Curve(c)) => Some(c),
            _ => None,
        }
    }

    pub fn text(&self, key: &str) -> Option<&str> {
        match self.v.get(key) {
            Some(Value::Text(t)) => Some(t),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        self.v.len()
    }

    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }

    /// Snapshot of every numeric / boolean / pattern value, for persistence.
    pub fn snapshot(&self) -> HashMap<String, Value> {
        self.v
            .iter()
            .filter(|(k, _)| !k.starts_with("transport.") && !k.starts_with("engine."))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn restore(&mut self, snap: HashMap<String, Value>) {
        for (k, v) in snap {
            self.v.insert(k, v);
        }
        self.rev += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_does_not_overwrite() {
        let mut b = Bus::new();
        b.set_num("a", 0.2);
        b.init("a", Value::Num(0.9));
        assert_eq!(b.num("a", 0.0), 0.2);
    }

    #[test]
    fn bools_read_as_numbers() {
        let mut b = Bus::new();
        b.set_flag("g", true);
        assert_eq!(b.num("g", 0.0), 1.0);
        assert!(b.flag("g", false));
        assert_eq!(b.num("missing", 0.5), 0.5);
    }

    #[test]
    fn value_round_trips_as_untagged_json() {
        let v = vec![
            Value::Bool(true),
            Value::Num(0.5),
            Value::Steps(vec![true, false]),
        ];
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, "[true,0.5,[true,false]]");
        let back: Vec<Value> = serde_json::from_str(&s).unwrap();
        assert_eq!(back, v);
    }
}
