use std::{
    io::{Read, Write},
    net::TcpStream,
    str::Utf8Error,
    vec,
};

mod resp;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Utf8Err(Utf8Error),
    RespErr(String)
}

pub struct Client {
    conn: TcpStream
}

impl Client {
    pub fn build(addr: String) -> Result<Client, Error> {
        let stream_ret = TcpStream::connect(addr);
        let mut stream: TcpStream;
        match stream_ret {
            Ok(s) => stream = s,
            Err(e) => return Err(Error::Io(e)),
        }
    
        let command = resp::pack(resp::Value::Array(vec![resp::Value::BulkStr(
            String::from("COMMAND"),
        )]));
        let write_ret = stream.write(command.as_bytes());
        match write_ret {
            Err(e) => return Err(Error::Io(e)),
            Ok(_) => {}
        }
    
        let mut buffer = [0; 1024];
        let mut utf8_buffer = Vec::new();
        loop {
            let read_ret = stream.read(&mut buffer);
            let read_n: usize;
            match read_ret {
                Ok(n) => read_n = n,
                Err(e) => return Err(Error::Io(e)),
            }
            if read_n == 0 {
                break;
            }
            utf8_buffer.extend_from_slice(&buffer[..read_n]);
        }
    
        let back_ret = std::str::from_utf8(&utf8_buffer);
        let back_str: String;
        match back_ret {
            Ok(ret) => back_str = ret.to_string(),
            Err(e) => return Err(Error::Utf8Err(e)),
        }
    
        match resp::unpack(back_str) {
            Ok(_) => {
                Ok(Client{conn: stream})
            },
            Err(e) => Err(Error::RespErr(e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{resp, resp::unpack};
    use std::{
        collections::{HashMap, HashSet},
        fs, vec,
    };

    #[test]
    fn parse_resp() {
        let bulk_str = "$5\r\nHello\r\n";

        assert_eq!(
            unpack(bulk_str.to_string()),
            Ok(resp::Value::BulkStr(String::from("Hello")))
        );

        let utf8_str = String::from("一杯🧊\r\n美式");
        let bulk_str_utf8 = format!("${}\r\n{}\r\n", utf8_str.len(), utf8_str);
        assert_eq!(
            unpack(bulk_str_utf8.to_string()),
            Ok(resp::Value::BulkStr(String::from("一杯🧊\r\n美式")))
        );

        let arr_str = "*2\r\n$5\r\nHello\r\n$5\r\nWorld\r\n";
        assert_eq!(
            unpack(arr_str.to_string()),
            Ok(resp::Value::Array(vec![
                resp::Value::BulkStr(String::from("Hello")),
                resp::Value::BulkStr(String::from("World"))
            ]))
        );

        let array_str2 = "*2\r\n*2\r\n$5\r\nHello\r\n$5\r\nWorld\r\n$5\r\nRust!\r\n";
        assert_eq!(
            unpack(array_str2.to_string()),
            Ok(resp::Value::Array(vec![
                resp::Value::Array(vec![
                    resp::Value::BulkStr(String::from("Hello")),
                    resp::Value::BulkStr(String::from("World"))
                ]),
                resp::Value::BulkStr(String::from("Rust!"))
            ]))
        );

        let number_str = ":100\r\n";
        assert_eq!(
            unpack(number_str.to_string()),
            Ok(resp::Value::Integer(100))
        );

        let number_str = ":-99\r\n";
        assert_eq!(
            unpack(number_str.to_string()),
            Ok(resp::Value::Integer(-99))
        );

        let number_str = ":+98\r\n";
        assert_eq!(unpack(number_str.to_string()), Ok(resp::Value::Integer(98)));

        let map_str = "%5\r\n+name\r\n+xiamingjie\r\n$5\r\nhello\r\n$5\r\nworld\r\n:1\r\n+1\r\n+array\r\n*1\r\n+item1\r\n+map\r\n%1\r\n+map_k\r\n+map_v\r\n";
        assert_eq!(
            unpack(map_str.to_string()),
            Ok(resp::Value::Map(HashMap::from([
                (
                    resp::Value::SimpleStr(String::from("name")),
                    resp::Value::SimpleStr(String::from("xiamingjie"))
                ),
                (
                    resp::Value::BulkStr(String::from("hello")),
                    resp::Value::BulkStr(String::from("world"))
                ),
                (
                    resp::Value::Integer(1),
                    resp::Value::SimpleStr(String::from("1"))
                ),
                (
                    resp::Value::SimpleStr(String::from("array")),
                    resp::Value::Array(vec![resp::Value::SimpleStr(String::from("item1"))])
                ),
                (
                    resp::Value::SimpleStr(String::from("map")),
                    resp::Value::Map(HashMap::from([(
                        resp::Value::SimpleStr(String::from("map_k")),
                        resp::Value::SimpleStr(String::from("map_v"))
                    )]))
                ),
            ])))
        );

        let bool_str = "#t\r\n";
        assert_eq!(unpack(bool_str.to_string()), Ok(resp::Value::Bool(true)));
        let bool_str = "#f\r\n";
        assert_eq!(unpack(bool_str.to_string()), Ok(resp::Value::Bool(false)));

        assert_eq!(
            unpack("~2\r\n:1\r\n+xiamingjie\r\n".to_string()),
            Ok(resp::Value::Set(HashSet::from([
                resp::Value::Integer(1),
                resp::Value::SimpleStr("xiamingjie".to_string())
            ])))
        );
    }

    #[test]
    fn parse_long_resp_example() {
        let content = fs::read_to_string("resp_example").expect("open the file failed");
        let r: usize = match unpack(content) {
            Ok(res) => {
                if let resp::Value::Array(d) = res {
                    d.len()
                } else {
                    0
                }
            }
            Err(_) => 0,
        };
        assert_eq!(r, 240);
    }

    #[test]
    fn pack() {
        assert_eq!(
            resp::pack(resp::Value::SimpleStr(String::from("hello"))),
            "+hello\r\n"
        );
        assert_eq!(
            resp::pack(resp::Value::Error(String::from("Error: redis nil"))),
            "-Error: redis nil\r\n"
        );
        assert_eq!(
            resp::pack(resp::Value::BulkStr(String::from("hello"))),
            "$5\r\nhello\r\n"
        );
        assert_eq!(resp::pack(resp::Value::Integer(99999)), ":99999\r\n");
        assert_eq!(resp::pack(resp::Value::Bool(true)), "#t\r\n");
        assert_eq!(
            resp::pack(resp::Value::Array(vec![
                resp::Value::BulkStr(String::from("hello")),
                resp::Value::Integer(123456)
            ])),
            "*2\r\n$5\r\nhello\r\n:123456\r\n"
        );
        assert_eq!(
            resp::pack(resp::Value::Map(HashMap::from([(
                resp::Value::SimpleStr(String::from("abc")),
                resp::Value::Array(vec![
                    resp::Value::BulkStr(String::from("hello")),
                    resp::Value::Integer(123456)
                ])
            )]))),
            "%1\r\n+abc\r\n*2\r\n$5\r\nhello\r\n:123456\r\n"
        );
        // assert_eq!(resp::pack(resp::Value::Set(HashSet::from([resp::Value::Integer(1), resp::Value::SimpleStr(String::from("xiamingjie"))]))), "~2\r\n:1\r\n+xiamingjie\r\n");
    }
}
