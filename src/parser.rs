use nom::character::complete::{alpha1, alphanumeric1, digit1, space0, space1};
use nom::*;
use std::iter::FromIterator;

type AppErr<'a> = nom::Err<(&'a str, nom::error::ErrorKind)>;

#[derive(Clone, Debug, PartialEq)]
pub enum LispVal {
    Atom(String),
    List(Vec<LispVal>),
    DottedList(Vec<LispVal>, Box<LispVal>),
    Number(u64),
    String(String),
    Boolean(bool),
}

fn match_symbols(input: String) -> LispVal {
    match input.as_str() {
        "#t" => LispVal::Boolean(true),
        "#f" => LispVal::Boolean(false),
        _ => LispVal::Atom(input),
    }
}

named!(parse_atom<&str, LispVal>, do_parse!(
        first: alt!(alpha1 | is_a!("!#$%&|*+-/:<=>?@^_~")) >>
        rest: many0!(complete!(alt!(alphanumeric1 | is_a!("!#$%&|*+-/:<=>?@^_~")))) >>
        (match_symbols(format!("{}{}", String::from(first), (String::from_iter(rest)))))
));

named!(parse_number<&str, LispVal>, do_parse!(
        number: many1!(digit1) >>
        (LispVal::Number(number.join("").parse::<u64>().unwrap()))
));

named!(
    parse_string<&str, LispVal>,
    do_parse!(
        char!('\"') >>
        value: many0!(none_of!("\"")) >> 
        char!('\"') >>
        (LispVal::String(String::from_iter(value)))
    )
);

named!(try_parse_list<&str, LispVal>, do_parse!(
        char!('(') >>
        items: alt!(parse_dotted_list | parse_list) >>
        char!(')') >>
        (items)
));

named!(parse_list<&str, LispVal>, do_parse!(
        items: separated_list!(space1, parse_expr) >>
        (LispVal::List(items))
));

named!(parse_quoted<&str, LispVal>, do_parse!(
        char!('\'') >>
        expr: parse_expr >>
        (LispVal::List(vec![LispVal::Atom("quote".to_owned()), expr]))
));

named!(dotted<&str, &str>, do_parse!(space0 >> char!('.') >> space0 >> (".")));

named!(parse_dotted_list<&str, LispVal>, do_parse!(
        exprs: separated_pair!(parse_list, dotted, parse_expr) >>
        ({
            let head = match exprs.0 {
                LispVal::List(v) => v,
                _ => panic!("List parser returned a non-list value")
            };
            LispVal::DottedList(head, Box::new(exprs.1))
        })
));

named!(parse_expr<&str, LispVal>, alt!(parse_atom | parse_number | parse_string | parse_quoted | try_parse_list));

pub fn parse_lisp_expr(input: &str) -> Result<(&str, LispVal), AppErr> {
    parse_expr(input)
}

#[cfg(test)]
mod tests {

    use crate::parser::*;
    use std::iter::FromIterator;

    #[test]
    fn number_parser_test() {
        assert!(parse_number("j5").is_err());
        assert!(parse_number("jlsdf").is_err());
        assert_eq!(parse_number("23").unwrap(), ("", LispVal::Number(23)));
    }

    #[test]
    fn string_parser_test() {
        let output = parse_string("\"hello\"").unwrap();
        assert_eq!(
            output,
            ("", LispVal::String(String::from_iter("hello".chars())))
        );
    }

    #[test]
    fn atom_parser_test() {
        assert_eq!(
            parse_atom("$foo").unwrap(),
            ("", LispVal::Atom(String::from_iter("$foo".chars())))
        );
        assert_eq!(parse_atom("#f").unwrap(), ("", LispVal::Boolean(false)));
    }

    #[test]
    fn list_parser_test() {
        assert_eq!(
            parse_list("$foo 42 53").unwrap(),
            (
                "",
                LispVal::List(vec!(
                    LispVal::Atom("$foo".to_owned()),
                    LispVal::Number(42),
                    LispVal::Number(53)
                ))
            )
        );
        assert_eq!(
            parse_list("\"foo\" 42 53").unwrap(),
            (
                "",
                LispVal::List(vec!(
                    LispVal::String("foo".to_owned()),
                    LispVal::Number(42),
                    LispVal::Number(53)
                ))
            )
        );
    }

    #[test]
    fn quoted_parser_test() {
        let output = parse_quoted("'52").unwrap();
        assert_eq!(
            output,
            (
                "",
                LispVal::List(vec![LispVal::Atom("quote".to_owned()), LispVal::Number(52)])
            )
        )
    }
}
