use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    SimpleStr(String),
    Error(String),
    Integer(i64),
    Array(Vec<Value>),
    BulkStr(String),
    Bool(bool),
    Map(HashMap<Value, Value>),
    Set(HashSet<Value>)
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::SimpleStr(s) => s.hash(state),
            Value::Error(e) => e.hash(state),
            Value::Integer(n) => n.hash(state),
            Value::Array(a) => a.hash(state),
            Value::BulkStr(s) => s.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Map(m) => {
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::Set(s) => {
                for e in s {
                    e.hash(state)
                }
            }
        }
    }
}

fn parse_str_to_num<T>(s: &str) -> Result<T, &'static str>
where
    T: FromStr,
{
    if let Some(n) = s.parse::<T>().ok() {
        Ok(n)
    } else {
        Err("parse str to number failed")
    }
}

#[allow(unused)]
fn parse(
    raw_msg: &str,
    begin_idx: usize,
    next_begin_idx: &mut usize,
) -> Result<Value, &'static str> {
    // todo need more specific error info
    let crlf = "\r\n";
    if let Some(msg_slice) = raw_msg.get(begin_idx..) {
        if let Some(body) = msg_slice
            .find(crlf)
            .and_then(|i| raw_msg.get(begin_idx..begin_idx + i))
        {
            *next_begin_idx = begin_idx + body.len() + crlf.len();
            if let Some((first_char, rest)) =
                body.chars().next().map(|c| (c, &body[c.len_utf8()..]))
            {
                return match first_char {
                    '+' => Ok(Value::SimpleStr(rest.to_string())),
                    '-' => Ok(Value::Error(rest.to_string())),
                    ':' => {
                        let n = parse_str_to_num::<i64>(rest)?;
                        Ok(Value::Integer(n))
                    }
                    '#' => match rest {
                        "t" => Ok(Value::Bool(true)),
                        "f" => Ok(Value::Bool(false)),
                        _ => Err("unknow type"),
                    },
                    '$' => {
                        let n = parse_str_to_num::<usize>(rest)?;
                        if let Some(s) = raw_msg.get(*next_begin_idx..*next_begin_idx + n) {
                            *next_begin_idx = *next_begin_idx + n + crlf.len();
                            Ok(Value::BulkStr(s.to_string()))
                        } else {
                            Err("not enough str")
                        }
                    }
                    '*' => {
                        let mut size = parse_str_to_num::<usize>(rest)?;
                        let mut res_arr: Vec<Value> = vec![];
                        while size > 0 {
                            let item = parse(raw_msg, *next_begin_idx, next_begin_idx)?;
                            res_arr.push(item);
                            size = size - 1;
                        }
                        Ok(Value::Array(res_arr))
                    }
                    '%' => {
                        let mut length = parse_str_to_num::<usize>(rest)?;
                        let mut res_map: HashMap<Value, Value> = HashMap::new();

                        while length > 0 {
                            let k = parse(raw_msg, *next_begin_idx, next_begin_idx)?;
                            let v = parse(raw_msg, *next_begin_idx, next_begin_idx)?;
                            res_map.insert(k, v);
                            length = length - 1;
                        }

                        Ok(Value::Map(res_map))
                    }
                    '~' => {
                        let mut length = parse_str_to_num::<usize>(rest)?;
                        let mut res_set: HashSet<Value> = HashSet::new();
                        while length > 0 {
                            res_set.insert(parse(raw_msg, *next_begin_idx, next_begin_idx)?);
                            length = length - 1;
                        }
                        Ok(Value::Set(res_set))
                    }
                    _ => Err("unknow type"),
                };
            } else {
                Err("empty str")
            }
        } else {
            Err("missing CRLF")
        }
    } else {
        Err("begin_idx out of range")
    }
}

#[allow(unused)]
pub fn unpack(msg: String) -> Result<Value, &'static str> {
    parse(&msg, 0, &mut 0)
}

#[allow(unused)]
pub fn pack(value: Value) -> String {
    match value {
        Value::SimpleStr(s) => format!("+{}\r\n", s),
        Value::Error(s) => format!("-{}\r\n", s),
        Value::BulkStr(s) => format!("${}\r\n{}\r\n", s.len(), s),
        Value::Bool(b) => format!("#{}\r\n", if b {"t"} else {"f"}),
        Value::Integer(n) => format!(":{}\r\n", n),
        Value::Array(a) => {
            let length = a.len();
            let mut str = String::from("");
            for item in a {
                str.push_str(&pack(item));
            }
            format!("*{}\r\n{}", length, str)
        }
        Value::Map(m) => {
            let length = m.len();
            let mut str = String::from("");
            for (k, v) in m {
                str.push_str(&pack(k));
                str.push_str(&pack(v));
            }
            format!("%{}\r\n{}", length, str)
        }
        Value::Set(s) => {
            let length = s.len();
            let mut str = String::from("");
            for e in s {
                str.push_str(&pack(e));
            }
            format!("~{}\r\n{}", length, str)
        }
    }
}
