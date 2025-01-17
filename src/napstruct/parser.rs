
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::io::BufRead;

use crate::napstruct;

// TODO move this in another file
pub trait ResponseExt {
    fn display_body(&mut self);
    fn display_headers(&self);
}

// TODO move this in another file
impl ResponseExt for reqwest::Response {
    fn display_body(&mut self) {
        println!("{}", self.text().unwrap());
    }

    fn display_headers(&self) {
        println!("// {:?} {}", self.version(), self.status());

        for (key, value) in self.headers().iter() {
            println!("// {}: {}", key, value.to_str().unwrap());
        }
    }
}

#[derive(Debug)]
enum LineType {
    Comment,
    //    Target,
    Header,
    //    Empty,
    Body,
    Param,
    DynParam,
}

#[derive(Debug)]
pub struct Parser {
    in_request: bool,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            in_request: false,
        }
    }

    fn new_request(&mut self, line: &str) -> napstruct::Request {
        let tmp = line.split(' ').collect::<Vec<&str>>();
        let request = napstruct::Request::new(tmp[0].to_string(), tmp[1..].join(" "));
        self.in_request = true;
        request
    }

    fn type_line(&self, line: &str) -> LineType {
        if line.starts_with('#') {
            LineType::Comment
        } else if self.is_header(line) {
            LineType::Header
        } else if self.is_param(line) {
            LineType::Param
        } else if self.is_dyn_param(line) {
            LineType::DynParam
        } else {
            LineType::Body
        }
    }

    fn is_in_request(&self) -> bool {
        self.in_request
    }

    fn set_in_request(&mut self, state: bool) {
        self.in_request = state;
    }

    fn is_header(&self, line: &str) -> bool {
        // TODO refactor this
        lazy_static! {
            static ref R : Regex = Regex::new(r"(?x) ^[\w-]+: \s .+").unwrap();
        }
        R.is_match(line)
    }

    fn is_param(&self, line: &str) -> bool {
        // TODO refactor this
        lazy_static! {
            static ref R : Regex = Regex::new(r"^:[^=]+?\s*[^:]=.+").unwrap();
        }
        R.is_match(line)
    }

    fn is_dyn_param(&self, line: &str) -> bool {
        // TODO refactor this
        lazy_static! {
            static ref R : Regex = Regex::new(r":\w+ := .*$").unwrap();
        }
        R.is_match(line)
    }

    fn process_param(line: &String, params: &mut HashMap<String, String>) {
        let tmp = &line.split('=').collect::<Vec<&str>>();
        let key = tmp[0][1..].trim().to_string();

        let value = tmp[1].trim().to_string();

        params.insert(key, value);
    }

    pub fn run<R: BufRead>(&self,
                   input: &mut R,
		   params: &mut HashMap<String, String>) -> Vec::<Result<napstruct::Request,String>> {
        let mut tmp = Vec::<String>::new();
        let mut reqs = Vec::<Result<napstruct::Request,String>>::new();

        for line in input.lines() {
            let current = line.unwrap();
            match self.type_line(current.as_str()) {
                LineType::Comment => {
                    if !tmp.is_empty() {
                        reqs.push(Ok(napstruct::Request::from_vec(tmp.to_vec())));
                        tmp = Vec::<String>::new();
                    }
                }
                LineType::Param => {
                    Parser::process_param(&current, params);
                }
                _ => {
                    tmp.push(current);
                }
            }
        }

        return reqs;
    }
}
#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_new() {
        let p = Parser::new();
        assert_eq!(p.in_request, false);
    }

    #[test]
    fn test_new_request() {
        let mut p = Parser::new();
        assert_eq!(p.in_request, false);

        let line = "POST https://some.url";
        let _r = p.new_request(line);
        assert_eq!(p.in_request, true);
    }

    #[test]
    fn test_is_header() {
        let p = Parser::new();

        assert_eq!(p.is_header("Content: application/json"), true);
        assert_eq!(p.is_header("Content :application/json"), false);
        assert_eq!(p.is_header("http://some.url:80"), false);
    }

    #[test]
    fn test_is_param() {
        let p = Parser::new();

        let line = ":some = param";
        assert_eq!(p.is_param(line), true);

        let line = ":some := dyn param";
        assert_eq!(p.is_param(line), false);

        let line = "POST http://some.url";
        assert_eq!(p.is_param(line), false);
    }

    #[test]
    fn test_is_dyn_param() {
        let p = Parser::new();

        let line = ":some = param";
        assert_eq!(p.is_dyn_param(line), false);

        let line = ":some := dyn param";
        assert_eq!(p.is_dyn_param(line), true);

        let line = "POST http://some.url";
        assert_eq!(p.is_dyn_param(line), false);
    }

    #[test]
    fn test_in_request() {
        let mut p = Parser::new();

        assert_eq!(p.in_request, false);
        assert_eq!(p.is_in_request(), false);

        p.in_request = true;
        assert_eq!(p.is_in_request(), true);

        p.set_in_request(false);
        assert_eq!(p.is_in_request(), false);

        p.set_in_request(true);
        assert_eq!(p.is_in_request(), true);
    }

    #[test]
    fn test_process_param() {
        let mut params: HashMap<String, String> = HashMap::new();

        Parser::process_param(&":some = param".to_string(), &mut params);
        assert_eq!(params["some"], "param");

        Parser::process_param(&":some_other=foo".to_string(), &mut params);
        assert_eq!(params["some_other"], "foo");
    }

    #[test]
    fn test_run_online() {
        let mut params: HashMap<String, String> = HashMap::new();
        // Parser::process_param(&":some = param".to_string(), &mut params);
        let input = "# Localhost bug test
GET http://localhost:3000";

        let parser = napstruct::parser::Parser::new();
        parser.run(&mut input.as_bytes(), &mut params);
        assert!(true);
    }
}
