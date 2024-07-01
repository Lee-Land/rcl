use std::{
    collections::{HashMap, HashSet}, hash::{Hash, Hasher}, num::ParseIntError
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
    Set(HashSet<Value>),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        self.to_string()
    }
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

const CRLF: &str = "\r\n";
const CRLF_LEN: usize = 2;

#[allow(unused)]
fn parse(raw_msg: &str, begin_idx: usize, next_begin_idx: &mut usize) -> Result<Value, ParseError> {
    if let Some(msg_slice) = raw_msg.get(begin_idx..) {
        if let Some(body) = msg_slice
            .find(CRLF)
            .and_then(|i| raw_msg.get(begin_idx..begin_idx + i))
        {
            *next_begin_idx = begin_idx + body.len() + CRLF_LEN;
            if let Some((first_char, rest)) =
                body.chars().next().map(|c| (c, &body[c.len_utf8()..]))
            {
                return match first_char {
                    '+' => Ok(Value::SimpleStr(rest.to_string())),
                    '-' => Ok(Value::Error(rest.to_string())),
                    ':' => {
                        match rest.parse::<i64>() {
                            Ok(n) => Ok(Value::Integer(n)),
                            Err(e) => Err(ParseError::ParseInt(e))
                        }
                    }
                    '#' => match rest {
                        "t" => Ok(Value::Bool(true)),
                        "f" => Ok(Value::Bool(false)),
                        _ => Err(ParseError::UnknowType(rest.to_string())),
                    },
                    '$' => {
                        match rest.parse::<usize>() {
                            Ok(n) => {
                                if let Some(s) = raw_msg.get(*next_begin_idx..*next_begin_idx + n) {
                                    *next_begin_idx = *next_begin_idx + n + CRLF_LEN;
                                    Ok(Value::BulkStr(s.to_string()))
                                } else {
                                    Err(ParseError::Incomplete)
                                }
                            },
                            Err(e) => Err(ParseError::ParseInt(e))
                        }
                    }
                    '*' => {
                        match rest.parse::<usize>() {
                            Ok(mut size) => {
                                let mut res_arr: Vec<Value> = vec![];
                                while size > 0 {
                                    let item = parse(raw_msg, *next_begin_idx, next_begin_idx)?;
                                    res_arr.push(item);
                                    size = size - 1;
                                }
                                Ok(Value::Array(res_arr))        
                            },
                            Err(e) => Err(ParseError::ParseInt(e))
                        }
                    }
                    '%' => {
                        match rest.parse::<usize>() {
                            Ok(mut size) => {
                                let mut res_map: HashMap<Value, Value> = HashMap::new();
                                while size > 0 {
                                    let k = parse(raw_msg, *next_begin_idx, next_begin_idx)?;
                                    let v = parse(raw_msg, *next_begin_idx, next_begin_idx)?;
                                    res_map.insert(k, v);
                                    size = size - 1;
                                }
                                Ok(Value::Map(res_map))
                            },
                        Err(e) => Err(ParseError::ParseInt(e))
                        }
                    }
                    '~' => {
                        match rest.parse::<usize>() {
                            Ok(mut size) => {
                                let mut res_set: HashSet<Value> = HashSet::new();
                        while size > 0 {
                            res_set.insert(parse(raw_msg, *next_begin_idx, next_begin_idx)?);
                            size = size - 1;
                        }
                        Ok(Value::Set(res_set))
                            },
                            Err(e) => Err(ParseError::ParseInt(e))
                        }
                    }
                    _ => Err(ParseError::UnknowType(first_char.to_string())),
                };
            } else {
                Err(ParseError::UnknowType(body.to_string()))
            }
        } else {
            Err(ParseError::Incomplete)
        }
    } else {
        Err(ParseError::Incomplete)
    }
}

#[allow(unused)]
pub fn unpack(msg: String) -> Result<Value, ParseError> {
    parse(&msg, 0, &mut 0)
}

#[allow(unused)]
pub fn pack(value: Value) -> String {
    match value {
        Value::SimpleStr(s) => format!("+{}\r\n", s),
        Value::Error(s) => format!("-{}\r\n", s),
        Value::BulkStr(s) => format!("${}\r\n{}\r\n", s.len(), s),
        Value::Bool(b) => format!("#{}\r\n", if b { "t" } else { "f" }),
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

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    UnknowType(String),
    ParseInt(std::num::ParseIntError),
    Incomplete,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ParseError::UnknowType(ref e) => write!(f, "unknow type: {}", e),
            ParseError::ParseInt(ref e) => write!(f, "{}", e),
            ParseError::Incomplete => write!(f, "received data is incomplete"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ParseError::UnknowType(_) => None,
            ParseError::ParseInt(ref e) => Some(e),
            ParseError::Incomplete => None,
        }
    }
}

impl From<ParseIntError> for ParseError {
    fn from(e: ParseIntError) -> Self {
        ParseError::ParseInt(e)
    }
}
