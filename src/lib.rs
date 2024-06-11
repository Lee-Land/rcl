mod model;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[allow(unused)]
fn parse(msg: &str) -> Result<model::Type, &'static str> {
    let mut result: Result<model::Type, &'static str> = Err("msg is empty");;
    msg.split("\r\n").collect::<Vec<&str>>().split_first().and_then(|(first, rest)|{
        let mut first_iter = first.chars();
        result = match first_iter.next() {
            Some(c) => match c {
                '+' => Ok(model::Type::SimpleStr(first_iter.collect::<String>().to_string())),
                _ => Err("unknow type")
            },
            None => Err("parse error")
        };
        Some(())
    }).or_else(||{
        result = Err("wrong syntax");
        Some(())
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_print_chars() {
        let s = String::from("我是夏铭杰");
        let s_arr: Vec<char> = s.chars().collect();
        assert_eq!(s_arr.len(), 5);
        assert_eq!(s_arr[0], '我');
        assert_eq!(s_arr[2], '夏');
        assert_eq!(s_arr[4], '杰');
    }

    #[test]
    fn parse_msg() {
        let mut res = parse("+123\r\n");
        assert_eq!(res, Ok(model::Type::SimpleStr("123".to_string())));

        res = parse("msg");
        assert_eq!(res, Err("wrong syntax"));

        res = parse("-123\r\n");
        assert_eq!(res, Err("unknow type"));
    }

    #[test]
    fn test_iter_next() {
        let v: Vec<i32> = vec![1,2,3,4,5];
        let mut i = v.iter();
        assert_eq!(i.next(), Some(&1));

        let new_v: Vec<i32> = i.cloned().collect();
        assert_eq!(new_v, vec![2,3,4,5]);
    }
}
