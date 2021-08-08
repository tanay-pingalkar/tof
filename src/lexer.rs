use super::utils::IsClosed;
use regex::Regex;

pub type Expr = Box<NODE>;

#[derive(Debug, Clone, PartialEq)]
pub enum LEX {
    DEF(usize, Def),
    EXPR(usize, Expr),
    RETURN(usize, Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub cond: Box<NODE>,
    pub block: Vec<LEX>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lamda {
    pub args: Vec<String>,
    pub value: Vec<LEX>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FcCall {
    pub args: Vec<Box<NODE>>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NODE {
    INT(f64),
    STR(String),
    LAMDA(Lamda),
    MATCH(Vec<Match>),
    FCCALL(FcCall),
    CALL(String),
    VOID,
    BOOL(bool),
    OP {
        joint: JOINT,
        lhs: Box<NODE>,
        rhs: Box<NODE>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum JOINT {
    ADD,
    SUB,
    MULT,
    DIV,
    GREAT,
    LESS,
    EQU,
    NOT,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Def {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    pub file: String,
    pub lex_vec: Vec<String>,
    pub lex: Vec<LEX>,
    pub coverage: usize,
}

lazy_static! {
    static ref JOINS_REGEX: regex::Regex = Regex::new(r"([\+\-\\/\*><=!])").unwrap();
    static ref FUNC_REGEX: regex::Regex = Regex::new(r"(\s+)").unwrap();
    static ref CALL_REGEX: regex::Regex = Regex::new(r"([a-zA-Z])").unwrap();
    static ref INT_REGEX: regex::Regex = Regex::new(r"([0-9.])").unwrap();
    static ref BOOL_REGEX: regex::Regex = Regex::new(r"(true|false)").unwrap();
}

impl Lexer {
    pub fn new(file: String) -> Lexer {
        let lex_vec: Vec<String> = file
            .split("\n")
            .collect::<Vec<&str>>()
            .iter()
            .map(|v| v.to_string())
            .collect();

        let lex: Vec<LEX> = Vec::new();
        Lexer {
            file,
            lex_vec,
            lex,
            coverage: 0,
        }
    }

    pub fn start(&mut self) {
        loop {
            match self.lexise(&self.lex_vec[self.coverage].clone()) {
                None => {}
                Some(value) => match value {
                    _ => {
                        self.lex.push(value);
                    }
                },
            }
            self.coverage = self.coverage + 1;
            if self.lex_vec.len() <= self.coverage {
                break;
            }
        }
    }

    pub fn lexise(&mut self, line: &str) -> Option<LEX> {
        let line = line.trim();

        let lex;

        if line == "" || line.starts_with("//") {
            lex = None;
        } else if line.contains(":") {
            let (name, value) = line.split_once(":").unwrap();

            lex = Some(LEX::DEF(
                self.coverage + 1,
                Def {
                    name: name.trim().to_string(),
                    value: self.expression(&value),
                },
            ));
        } else if line.trim().starts_with("~") {
            let line = line[1..].to_string();
            lex = Some(LEX::RETURN(
                self.coverage + 1,
                self.expression(&line.clone()),
            ));
        } else {
            lex = Some(LEX::EXPR(
                self.coverage + 1,
                self.expression(&self.lex_vec[self.coverage].clone()),
            ));
        }
        lex
    }

    pub fn args(&mut self, value: &str) -> Vec<Box<NODE>> {
        if value.trim() == "_" {
            return Vec::new();
        }
        let splited = value.trim().split("").collect::<Vec<&str>>();
        let mut args = Vec::new();
        let mut arg = Vec::new();

        let mut is_closed = IsClosed::new();

        let mut i = 0;
        loop {
            if FUNC_REGEX.is_match(splited[i]) && is_closed.is() {
                let exp = arg.join("");
                args.push(self.expression(&exp));
                arg = Vec::new();
            }

            is_closed.check(splited[i]);

            arg.push(splited[i]);
            i = i + 1;

            if splited.len() == i && is_closed.is() {
                args.push(self.node(&arg.join("")));
                break;
            } else if splited.len() == i {
            }
        }
        args
    }

    pub fn node(&mut self, part: &str) -> Expr {
        let part = part.trim();
        let n;

        if part.starts_with("\"") && part.ends_with("\"") {
            let part = part.trim_matches('\"').to_string();
            n = Box::new(NODE::STR(part));
        } else if BOOL_REGEX.is_match(&part) {
            n = Box::new(NODE::BOOL(match part {
                "true" => true,
                "false" => false,
                _ => panic!("bug at bool regex"),
            }));
        } else if part == "TRUE" {
            n = Box::new(NODE::BOOL(true));
        } else if part == "FALSE" {
            n = Box::new(NODE::BOOL(false));
        } else if part.starts_with("(") && part.ends_with(")") {
            n = self.expression(&part[1..part.len() - 1]);
        } else if part.contains("->") {
            let (args, steps) = part.split_once("->").unwrap();
            let steps = steps.trim();
            let value;

            if steps.starts_with("{") && steps.ends_with("}") {
                let mut lex = Lexer::new(steps[1..steps.len() - 1].to_string());
                lex.start();
                value = lex.lex();
            } else {
                let mut lex = Lexer::new(format!("~{}", steps.to_string()));
                lex.start();
                value = lex.lex();
            }

            let mut args: Vec<String> = args
                .split_whitespace()
                .collect::<Vec<&str>>()
                .iter()
                .map(|v| v.trim().to_string())
                .collect();

            if args[0] == "_" {
                args = Vec::new()
            }
            n = Box::new(NODE::LAMDA(Lamda { args, value }))
        } else if FUNC_REGEX.is_match(&part) {
            let splitted = part.split_whitespace().collect::<Vec<&str>>();
            let name = splitted[0];

            n = Box::new(NODE::FCCALL(FcCall {
                args: self.args(&part[name.len()..]),
                name: name.to_string(),
            }))
        } else if CALL_REGEX.is_match(&part) {
            n = Box::new(NODE::CALL(part.to_string()))
        } else if INT_REGEX.is_match(&part) {
            n = Box::new(NODE::INT(part.parse().unwrap()));
        } else {
            panic!("not a type : {} on line {}", part, self.coverage);
        }
        n
    }

    pub fn op(join: &str) -> JOINT {
        match join {
            "+" => JOINT::ADD,
            "-" => JOINT::SUB,
            "/" => JOINT::DIV,
            "*" => JOINT::MULT,
            ">" => JOINT::GREAT,
            "<" => JOINT::LESS,
            "=" => JOINT::EQU,
            "!" => JOINT::NOT,
            _ => panic!("operator not found : {}", join),
        }
    }

    pub fn expression(&mut self, string: &str) -> Box<NODE> {
        let mut splited: Vec<String> = string
            .trim()
            .split("")
            .collect::<Vec<&str>>()
            .iter()
            .map(|v| v.to_string())
            .collect();
        let node;
        let mut i = 0;

        let mut parts = Vec::new();
        let mut is_closed = IsClosed::new();
        loop {
            if JOINS_REGEX.is_match(&splited[i]) && is_closed.is() && !is_closed.in_arrow {
                let part = parts.join("");

                if splited[i] == "-" && splited[i + 1] == ">" {
                    is_closed.in_arrow = true;
                } else {
                    let spliced = splited
                        .splice(i + 1.., vec![])
                        .collect::<Vec<String>>()
                        .join("");

                    node = Box::new(NODE::OP {
                        lhs: self.node(&part.as_str()),
                        rhs: self.expression(&spliced),
                        joint: Lexer::op(&splited[i]),
                    });
                    break;
                }
            }

            is_closed.check(&splited[i]);

            parts.push(splited[i].clone());
            i = i + 1;

            if splited.len() == i && is_closed.is() {
                node = self.node(&parts.join(""));
                break;
            } else if splited.len() == i {
                self.coverage = self.coverage + 1;
                let nl = self.lex_vec[self.coverage].clone();
                splited = [
                    splited,
                    vec!["\n".to_string()],
                    nl.trim()
                        .split("")
                        .collect::<Vec<&str>>()
                        .iter()
                        .map(|v| v.to_string())
                        .collect(),
                ]
                .concat();
            }
        }

        node
    }

    pub fn lex(&self) -> Vec<LEX> {
        self.lex.clone()
    }
}
