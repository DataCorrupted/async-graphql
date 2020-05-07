use crate::parser::span::Spanned;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Variable(String),
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Enum(String),
    List(Vec<Spanned<Value>>),
    Object(BTreeMap<Spanned<String>, Spanned<Value>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;

        match (self, other) {
            (Variable(a), Variable(b)) => a.eq(b),
            (Int(a), Int(b)) => a.eq(b),
            (Float(a), Float(b)) => a.eq(b),
            (String(a), String(b)) => a.eq(b),
            (Boolean(a), Boolean(b)) => a.eq(b),
            (Null, Null) => true,
            (Enum(a), Enum(b)) => a.eq(b),
            (List(a), List(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for i in 0..a.len() {
                    if !a[i].eq(&b[i]) {
                        return false;
                    }
                }
                true
            }
            (Object(a), Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (key, a_value) in a.iter() {
                    if let Some(b_value) = b.get(key) {
                        if !a_value.eq(b_value) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}
