use std::{collections::HashMap, hash::{Hash, Hasher}};

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    SimpleStr(String),
    Error(String),
    Integer(i64),
    Array(Vec<Value>),
    BulkStr(String),
    Bool(bool),
    Map(HashMap<Value, Value>)
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // todo 正确性？
        match self {
            Value::SimpleStr(_) => 0.hash(state),
            Value::Error(_) => 1.hash(state),
            Value::Integer(_) => 2.hash(state),
            Value::Array(_) => 3.hash(state),
            Value::BulkStr(_) => 4.hash(state),
            Value::Bool(_) => 5.hash(state),
            Value::Map(_) => 6.hash(state),
        }
    }
}
