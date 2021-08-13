use super::utils::IsClosed;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct Tokens {
    pub line_number: usize,
    pub token: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Def(Def),
    Return(Box<Expr>),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lamda {
    pub args: Vec<String>,
    pub value: Vec<Tokens>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub cond: Box<Expr>,
    pub block: Vec<Tokens>,
    pub next: Option<Box<Decision>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FcCall {
    pub args: Vec<Box<Expr>>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Op {
    pub joint: JOINT,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Def {
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(f64),
    Str(String),
    Lamda(Lamda),
    Decision(Decision),
    FcCall(FcCall),
    Call(String),
    Scope(Vec<Tokens>),
    Bool(bool),
    Op(Op),
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

#[derive(Debug)]
pub struct Tokenizer {
    pub tokens: Vec<Tokens>,
    lines: Vec<String>,
    coverage: usize,
}

lazy_static! {
    static ref JOINS_REGEX: regex::Regex = Regex::new(r"([\+\-\\/\*><=!])").unwrap();
    static ref FUNC_REGEX: regex::Regex = Regex::new(r"(\s+)").unwrap();
    static ref CALL_REGEX: regex::Regex = Regex::new(r#"([^\+\-\\/\*><=!"\{\}])"#).unwrap();
    static ref WHITE_REGEX: regex::Regex = Regex::new(r#"([\w])"#).unwrap();
    static ref INT_REGEX: regex::Regex = Regex::new(r"([0-9.])").unwrap();
    static ref BOOL_REGEX: regex::Regex = Regex::new(r"(TRUE|FALSE)").unwrap();
}

impl Tokenizer {
    pub fn new(file: &str) -> Tokenizer {
        let lines: Vec<String> = file
            .split("\n")
            .collect::<Vec<&str>>()
            .iter()
            .map(|v| v.to_string())
            .collect();

        let tokens: Vec<Tokens> = Vec::new();
        Tokenizer {
            lines,
            tokens,
            coverage: 0,
        }
    }

    pub fn start(&mut self) {
        loop {
            match self.token_resolver(&self.lines[self.coverage].clone()) {
                None => {}
                Some(value) => match value {
                    _ => {
                        self.tokens.push(value);
                    }
                },
            }
            self.coverage = self.coverage + 1;
            if self.lines.len() <= self.coverage {
                break;
            }
        }
    }

    pub fn token_resolver(&mut self, line: &str) -> Option<Tokens> {
        let trimed_line = line.trim();

        if trimed_line == "" || trimed_line.starts_with("//") {
            None
        } else if line.contains(":") {
            self.def_resolver(&line)
        } else {
            self.return_expr_resolver(&line)
        }
    }

    pub fn def_resolver(&mut self, line: &str) -> Option<Tokens> {
        let (name, value) = line.split_once(":").unwrap();
        if CALL_REGEX.is_match(name.trim()) && !WHITE_REGEX.is_match(name.trim()) {
            Some(Tokens {
                line_number: self.coverage + 1,
                token: Token::Def(Def {
                    name: name.trim().to_string(),
                    value: self.expression_resolver(&value),
                }),
            })
        } else {
            self.return_expr_resolver(&line)
        }
    }

    pub fn return_expr_resolver(&mut self, mut line: &str) -> Option<Tokens> {
        if line.trim().starts_with("~") {
            line = &line[1..];
            Some(Tokens {
                line_number: self.coverage,
                token: Token::Return(self.expression_resolver(&line)),
            })
        } else {
            Some(Tokens {
                line_number: self.coverage,
                token: Token::Expr(self.expression_resolver(&self.lines[self.coverage].clone())),
            })
        }
    }

    pub fn args_resolver(&mut self, value: &str) -> Vec<Box<Expr>> {
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
                args.push(self.expression_resolver(&exp));
                arg = Vec::new();
            }

            is_closed.check(splited[i]);

            arg.push(splited[i]);
            i = i + 1;

            if splited.len() == i && is_closed.is() {
                args.push(self.node_resolver(&arg.join("")));
                break;
            } else if splited.len() == i {
            }
        }
        args
    }

    pub fn scope_lamda_resolver(&mut self, part: &str) -> Box<Expr> {
        let (args, steps) = part.split_once("->").unwrap();
        let steps = steps.trim();
        let value;

        if steps.starts_with("{") && steps.ends_with("}") {
            let mut tokenizer = Tokenizer::new(&steps[1..steps.len() - 1]);
            tokenizer.start();
            value = tokenizer.tokens;
        } else {
            value = vec![Tokens {
                line_number: self.coverage,
                token: Token::Return(self.expression_resolver(steps)),
            }];
        }
        if args.trim() == "" {
            Box::new(Expr::Scope(value))
        } else {
            let mut args: Vec<String> = args
                .split_whitespace()
                .collect::<Vec<&str>>()
                .iter()
                .map(|v| v.trim().to_string())
                .collect();

            if args[0] == "_" {
                args = Vec::new()
            }
            Box::new(Expr::Lamda(Lamda { args, value }))
        }
    }

    pub fn node_resolver(&mut self, part: &str) -> Box<Expr> {
        let part = part.trim();
        let n;

        if part.starts_with("\"") && part.ends_with("\"") {
            let part = part.trim_matches('\"').to_string();
            n = Box::new(Expr::Str(part));
        } else if part == "TRUE" {
            n = Box::new(Expr::Bool(true));
        } else if part == "FALSE" {
            n = Box::new(Expr::Bool(false));
        } else if part.starts_with("(") && part.ends_with(")") {
            n = self.expression_resolver(&part[1..part.len() - 1]);
        } else if FUNC_REGEX.is_match(&part) {
            let splitted = part.split_whitespace().collect::<Vec<&str>>();
            let name = splitted[0];

            n = Box::new(Expr::FcCall(FcCall {
                args: self.args_resolver(&part[name.len()..]),
                name: name.to_string(),
            }))
        } else if CALL_REGEX.is_match(&part) {
            n = Box::new(Expr::Call(part.to_string()))
        } else if INT_REGEX.is_match(&part) {
            n = Box::new(Expr::Int(part.parse().unwrap()));
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

    pub fn expression_resolver(&mut self, string: &str) -> Box<Expr> {
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
        let mut cond: Option<String> = None;
        loop {
            if JOINS_REGEX.is_match(&splited[i])
                && is_closed.is()
                && !is_closed.in_arrow
                && !is_closed.in_cond
            {
                let part = parts.join("");

                if splited[i] == "-" && splited[i + 1] == ">" {
                    is_closed.in_arrow = true;
                } else {
                    let spliced = splited
                        .splice(i + 1.., vec![])
                        .collect::<Vec<String>>()
                        .join("");

                    node = Box::new(Expr::Op(Op {
                        lhs: self.node_resolver(&part.as_str()),
                        rhs: self.expression_resolver(&spliced),
                        joint: Tokenizer::op(&splited[i]),
                    }));
                    break;
                }
            }

            if splited[i] == "|" && splited[i + 1] == "|" && is_closed.is() && !is_closed.in_arrow {
                node = self.decision_resolver(
                    &parts.join(""),
                    cond,
                    &splited
                        .splice(i + 2..splited.len() - 1, vec![])
                        .collect::<Vec<String>>()
                        .join(""),
                );
                break;
            }

            is_closed.check(&splited[i]);

            if splited[i] == "?" && !is_closed.in_arrow {
                is_closed.in_cond = true;
                cond = Some(parts.join("").to_string());
                parts = Vec::new();
            } else {
                parts.push(splited[i].clone());
            }
            i = i + 1;

            if splited.len() == i && is_closed.is() {
                if is_closed.in_cond {
                    let nl = self.lines[self.coverage + 1].clone();
                    if nl.trim().starts_with("||") && !self.if_lv_full() {
                        self.coverage = self.coverage + 1;
                        splited = [
                            splited,
                            nl.trim()
                                .split("")
                                .collect::<Vec<&str>>()
                                .iter()
                                .map(|v| v.to_string())
                                .collect(),
                        ]
                        .concat();
                    } else {
                        node = self.decision_resolver(&parts.join(""), cond, "");
                        break;
                    }
                } else {
                    if is_closed.in_arrow {
                        node = self.scope_lamda_resolver(&parts.join(""));
                        break;
                    } else {
                        node = self.node_resolver(&parts.join(""));
                        break;
                    }
                }
            } else if splited.len() == i {
                self.coverage = self.coverage + 1;
                if self.if_lv_full() {
                    panic!("you forget to close {:?} ", is_closed.unclosed());
                }
                let nl = self.lines[self.coverage].clone();
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

    pub fn decision_resolver(
        &mut self,
        block: &str,
        cond: Option<String>,
        next_s: &str,
    ) -> Box<Expr> {
        let mut block = Tokenizer::new(&format!("~{}", block.to_string()));
        block.start();
        let block = block.tokens;

        let mut is_closed = IsClosed::new();

        let mut next_s = next_s
            .split("")
            .collect::<Vec<&str>>()
            .iter()
            .map(|v| {
                is_closed.check(&v);
                v.to_string()
            })
            .collect();

        if !is_closed.is() {
            loop {
                self.coverage = self.coverage + 1;
                let nl = self.lines[self.coverage].clone();
                let nl = nl
                    .trim()
                    .split("")
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|v| {
                        is_closed.check(v);
                        v.to_string()
                    })
                    .collect::<Vec<String>>()
                    .join("");
                next_s = [next_s, "\n".to_string(), nl].concat();

                if is_closed.is() {
                    break;
                }
            }
        }

        let lex;
        if next_s != "" {
            lex = self.expression_resolver(&next_s);
        } else {
            panic!(
                "else condition should be specified at line {}",
                self.coverage
            );
        }

        let next;

        match *lex {
            Expr::Decision(d) => {
                next = Some(Box::new(Decision {
                    cond: d.cond,
                    block: d.block,
                    next: d.next,
                }));
            }
            _ => {
                next = Some(Box::new(Decision {
                    cond: Box::new(Expr::Bool(true)),
                    block: vec![Tokens {
                        line_number: self.coverage,
                        token: Token::Return(lex),
                    }],
                    next: None,
                }));
            }
        }

        Box::new(Expr::Decision(Decision {
            cond: self.expression_resolver(&match cond {
                Some(s) => s,
                None => {
                    panic!("please specify condition {}", self.coverage);
                }
            }),
            block,
            next,
        }))
    }

    pub fn if_lv_full(&self) -> bool {
        self.lines.len() == self.coverage
    }
}
