#[allow(unused)]
#[derive(Debug, PartialEq)]
enum RESPValue {
    SimpleStr(String),
    Error(String),
    Integer(i64),
    Array(Vec<RESPValue>),
    BulkStr(String)
}

fn parse_type(msg: &str) -> Result<RESPValue, &'static str> {
    let res = msg.find("\r\n").and_then(|i| -> Option<&str> {
        msg.get(..i)
    });

    if let Some(first) = res {
        if let Some((first_char, rest)) = 
            first.chars().next().map(|c| (c, &first[c.len_utf8()..])) {
            match first_char {
                '+' => Ok(RESPValue::SimpleStr(rest.to_string())),
                '$' => {
                    let n_r = rest.parse::<i32>();
                    if let Some(n) = n_r.ok() {
                        
                    } else {
                        return Err("invlid bulk string content");
                    }
                    return Ok(())
                },
                '*' => {
                    
                    Ok(())
                }
                _ => Err("unknow data type")
            }
        } else {
            return Err("missing content");
        }
    } else {
        return Err("missing CRLF");
    }
}

fn parse_content(ch: char, rest: &str) -> Result<RESPValue, &'static str> {
    match ch {
        '+' => Ok(RESPValue::SimpleStr(rest.to_string())),
        '$' => {
            let n_r = rest.parse::<i32>();
            if let Some(n) = n_r.ok() {

            } else {
                return Err("invlid bulk string content");
            }
            return Ok(())
        }
        _ => Err("unknow data type")
    }
}

#[cfg(test)]
mod tests {
    use crate::RESPValue;

    #[test]
    fn it_works() {
        let msg = "+OK\r\n";
        let res = msg.find("\r\n").and_then(|i: usize| ->Option<&str> {
            msg.get(0..i)
        });
        assert_eq!(res, Some("+OK"));
        
        if let Some(first) = res {
            if let Some((first_char, rest)) = 
                first.chars().next().map(|c| (c, &first[c.len_utf8()..])) {
                assert_eq!(first_char, '+');
                assert_eq!(rest, "OK");

                let t = match first_char {
                    '+' => RESPValue::SimpleStr(rest.to_string()),
                    '$' => 
                    _ => RESPValue::Error("unknow data type".to_string())
                };
                assert_eq!(t, RESPValue::SimpleStr("OK".to_string()));
            }
        }
    }
}
