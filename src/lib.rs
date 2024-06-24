use std::{collections::HashMap, str::FromStr};

mod resp;

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
fn parse_content(
    raw_msg: &str,
    begin_idx: usize,
    next_begin_idx: &mut usize,
) -> Result<resp::Value, &'static str> {    // todo need more specific error info
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
                    '+' => Ok(resp::Value::SimpleStr(rest.to_string())),
                    '-' => Ok(resp::Value::Error(rest.to_string())),
                    ':' => {
                        let n = parse_str_to_num::<i64>(rest)?;
                        Ok(resp::Value::Integer(n))
                    }
                    '#' => match rest {
                        "t" => Ok(resp::Value::Bool(true)),
                        "f" => Ok(resp::Value::Bool(false)),
                        _ => Err("unknow type"),
                    },
                    '$' => {
                        let n = parse_str_to_num::<usize>(rest)?;
                        if let Some(s) = raw_msg.get(*next_begin_idx..*next_begin_idx + n) {
                            *next_begin_idx = *next_begin_idx + n + crlf.len();
                            Ok(resp::Value::BulkStr(s.to_string()))
                        } else {
                            Err("not enough str")
                        }
                    }
                    '*' => {
                        let mut size = parse_str_to_num::<usize>(rest)?;
                        let mut res_arr: Vec<resp::Value> = vec![];
                        while size > 0 {
                            let item =
                                parse_content(raw_msg, *next_begin_idx, next_begin_idx)?;
                            res_arr.push(item);
                            size = size - 1;
                        }
                        Ok(resp::Value::Array(res_arr))
                    },
                    '%' => {
                        let mut length = parse_str_to_num::<usize>(rest)?;
                        let mut res_map: HashMap<resp::Value, resp::Value> = HashMap::new();

                        while length > 0 {
                            let k = parse_content(raw_msg, *next_begin_idx, next_begin_idx)?;
                            let v = parse_content(raw_msg, *next_begin_idx, next_begin_idx)?;
                            res_map.insert(k, v);
                            length = length - 1;
                        }

                        Ok(resp::Value::Map(res_map))
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
fn parse(msg: &str) -> Result<resp::Value, &'static str> {
    parse_content(msg, 0, &mut 0)
}

#[cfg(test)]
mod tests {
    use crate::{resp, parse};
    use std::{collections::HashMap, fs, vec};

    #[test]
    fn it_works() {
        let msg = "+OK\r\n";
        let res = msg
            .find("\r\n")
            .and_then(|i: usize| -> Option<&str> { msg.get(0..i) });
        assert_eq!(res, Some("+OK"));
    }

    #[test]
    fn parse_resp() {
        let bulk_str = "$5\r\nHello\r\n";

        assert_eq!(
            parse(bulk_str),
            Ok(resp::Value::BulkStr(String::from("Hello")))
        );

        let utf8_str = String::from("一杯🧊\r\n美式");
        let bulk_str_utf8 = format!("${}\r\n{}\r\n", utf8_str.len(), utf8_str);
        assert_eq!(
            parse(&bulk_str_utf8),
            Ok(resp::Value::BulkStr(String::from("一杯🧊\r\n美式")))
        );

        let arr_str = "*2\r\n$5\r\nHello\r\n$5\r\nWorld\r\n";
        assert_eq!(
            parse(arr_str),
            Ok(resp::Value::Array(vec![
                resp::Value::BulkStr(String::from("Hello")),
                resp::Value::BulkStr(String::from("World"))
            ]))
        );

        let array_str2 = "*2\r\n*2\r\n$5\r\nHello\r\n$5\r\nWorld\r\n$5\r\nRust!\r\n";
        assert_eq!(
            parse(array_str2),
            Ok(resp::Value::Array(vec![
                resp::Value::Array(vec![
                    resp::Value::BulkStr(String::from("Hello")),
                    resp::Value::BulkStr(String::from("World"))
                ]),
                resp::Value::BulkStr(String::from("Rust!"))
            ]))
        );

        let number_str = ":100\r\n";
        assert_eq!(parse(number_str), Ok(resp::Value::Integer(100)));

        let number_str = ":-99\r\n";
        assert_eq!(parse(number_str), Ok(resp::Value::Integer(-99)));

        let number_str = ":+98\r\n";
        assert_eq!(parse(number_str), Ok(resp::Value::Integer(98)));

        let map_str = "%5\r\n+name\r\n+xiamingjie\r\n$5\r\nhello\r\n$5\r\nworld\r\n:1\r\n+1\r\n+array\r\n*1\r\n+item1\r\n+map\r\n%1\r\n+map_k\r\n+map_v\r\n";
        assert_eq!(parse(map_str), Ok(resp::Value::Map(HashMap::from([
            (resp::Value::SimpleStr(String::from("name")), resp::Value::SimpleStr(String::from("xiamingjie"))),
            (resp::Value::BulkStr(String::from("hello")), resp::Value::BulkStr(String::from("world"))),
            (resp::Value::Integer(1), resp::Value::SimpleStr(String::from("1"))),
            (resp::Value::SimpleStr(String::from("array")), resp::Value::Array(vec![resp::Value::SimpleStr(String::from("item1"))])),
            (resp::Value::SimpleStr(String::from("map")), resp::Value::Map(HashMap::from([(resp::Value::SimpleStr(String::from("map_k")), resp::Value::SimpleStr(String::from("map_v")))]))),
        ]))));
    }

    #[test]
    fn parse_long_resp_example() {
        let content = fs::read_to_string("resp_example").expect("open the file failed");
        let r: usize = match parse(&content.to_string()) {
            Ok(res) => {
                if let resp::Value::Array(d) = res {
                    d.len()
                } else {
                    0
                }
            },
            Err(_) => 0
        };
        assert_eq!(r, 240);
    }
}
