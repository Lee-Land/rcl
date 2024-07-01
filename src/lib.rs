use std::{
    io::{Read, Write}, net::TcpStream, str::Utf8Error, vec
};

mod resp;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Utf8Err(Utf8Error),
    RespErr(resp::ParseError),
    ServerProto(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::RespErr(ref s) => write!(f, "parse resp protocol error: {}", s),
            Error::Io(ref e) => write!(f, "{}", e),
            Error::Utf8Err(ref e) => write!(f, "{}", e),
            Error::ServerProto(ref s) => write!(f, "expected to receivie type: Map, but received type: {}", s)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Io(ref e) => Some(e),
            Error::Utf8Err(ref e) => Some(e),
            Error::RespErr(ref e) => Some(e),
            Error::ServerProto(_) => None
        }
    }
}

fn recv(conn: &mut TcpStream) -> Result<resp::Value, Error> {
    let mut utf8_buffer = Vec::new();

    let resp_val: Result<resp::Value, Error>;
    loop {
        let mut buffer = [0; 1024];
        let read_ret = conn.read(&mut buffer);
        let read_n: usize;
        match read_ret {
            Ok(n) => read_n = n,
            Err(e) => return Err(Error::Io(e)),
        }
        if read_n == 0 {
            return Err(Error::Io(std::io::Error::new(std::io::ErrorKind::ConnectionReset, "connection was closed")));
        }
        utf8_buffer.extend_from_slice(&buffer[..read_n]);
        let back_ret = std::str::from_utf8(&utf8_buffer);
        match back_ret {
            Ok(str) => {
                match resp::unpack(str.to_string()) {
                    Ok(value) => resp_val = Ok(value),
                    Err(e) => {
                        match e {
                            resp::ParseError::Incomplete => continue,
                            _ => resp_val = Err(Error::RespErr(e))
                        }
                    }
                }
            },
            Err(_) => continue,
        };
        return resp_val;
    }
}

fn send(conn: &mut TcpStream, req: resp::Value) -> Result<(), Error> {
    match conn.write(resp::pack(req).as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Io(e)),
    }
}

pub struct Client {
    conn: TcpStream,
    srv_info: Server
}

struct Server {
    srv_name: String,
    version: String,
    proto_ver: i32,
    id: i32,
    mode: String,
    role: String
}

impl Client {
    pub fn build(addr: String) -> Result<Client, Error> {
        let stream_ret = TcpStream::connect(addr);
        let mut stream: TcpStream;
        match stream_ret {
            Ok(s) => stream = s,
            Err(e) => return Err(Error::Io(e)),
        }

        let command = resp::Value::Array(vec![resp::Value::BulkStr(String::from("hello"))]);
        match send(&mut stream, command) {
            Err(e) => return Err(e),
            Ok(_) => {}
        }

        match recv(&mut stream) {
            Ok(val) => {
                match val {
                    resp::Value::Map(mp) => {
                        Client{
                            conn: stream,
                            srv_info: Server{
                                
                            }
                        }
                    },
                    _ => {
                        Err(Error::ServerProto(val.to_string()))
                    }
                }
            },
            Err(e) => Err(e)
        }
    }

    pub fn get(&mut self, key: String) -> Result<resp::Value, Error> {
        let arr = resp::Value::Array(vec![
            resp::Value::BulkStr(String::from("get")),
            resp::Value::BulkStr(key),
        ]);
        
        match send(&mut self.conn, arr) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }

        match recv(&mut self.conn) {
            Ok(response) => Ok(response),
            Err(e) => Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::resp::{self, unpack};
    use std::{
        collections::{HashMap, HashSet}, fs, vec
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

    #[test]
    fn connect_and_ping() {
        let mut cli = crate::Client::build(String::from("127.0.0.1:6379")).unwrap();
        let ret = cli.get("hello".to_string()).unwrap();
        assert_eq!(ret, resp::Value::BulkStr("1".to_string()));
    }
}
