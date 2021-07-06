use regex::Regex;

#[derive(Debug, Clone)]
pub enum LEX {
    DEF(Def),
    EXPR(Expr),
    RETURN(Expr),
}

#[derive(Debug, Clone)]
pub enum Node {
    Int(i32),
    Str(String),
    Lamda {
        args: Vec<String>,
        value: Vec<LEX>,
    },
    FCCALL {
        args: Vec<Box<Node>>,
        name: String,
    },
    CALL(String),
    VOID,
    OP {
        joint: JOINT,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub line_number: usize,
    pub node: Box<Node>,
}

#[derive(Debug, Clone)]
pub enum JOINT {
    ADD,
    SUB,
    MULT,
    DIV,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub line_number: usize,
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    pub file: &'a str,
    pub lex_vec: Vec<&'a str>,
    pub lex: Vec<LEX>,
    pub coverage: usize,
}

lazy_static! {
    static ref JOINS_REGEX: regex::Regex = Regex::new(r"([\+\-\\/\*])").unwrap();
    static ref FUNC_REGEX: regex::Regex = Regex::new(r"([\(\)])").unwrap();
    static ref CALL_REGEX: regex::Regex = Regex::new(r"([a-zA-Z])").unwrap();
    static ref INT_REGEX: regex::Regex = Regex::new(r"([0-9])").unwrap();
}

impl<'a> Lexer<'a> {
    pub fn new(file: &'a str) -> Lexer<'a> {
        let lex_vec: Vec<&'a str> = file.split("\n").collect();
        let lex: Vec<LEX> = Vec::new();
        Lexer {
            file,
            lex_vec,
            lex,
            coverage: 0,
        }
    }

    pub fn start_lex(&mut self) {
        if self.coverage != self.lex_vec.len() {
            let line = self.lex_vec[self.coverage];
            if line.trim() == "" || line.trim().starts_with("//") {
            } else if line.contains(":") {
                let (name, value_str) = line.split_once(":").unwrap();

                if !value_str.ends_with("{") {
                    self.lex.push(LEX::DEF(Def {
                        name: name.trim().to_string(),
                        line_number: self.coverage,
                        value: Expr {
                            line_number: self.coverage,
                            node: self.equation_resolver(&value_str),
                        },
                    }));
                } else {
                    self.coverage = self.coverage + 1;
                    let mut value: Vec<LEX> = Vec::new();
                    let mut args = value_str
                        .split_once("(")
                        .unwrap()
                        .1
                        .split_once(")")
                        .unwrap()
                        .0
                        .split(",")
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<String>>();

                    if args[0].trim() == "" {
                        args.pop();
                    }

                    self.deep_lex(&mut value);
                    self.lex.push(LEX::DEF(Def {
                        name: name.trim().to_string(),
                        line_number: self.coverage,
                        value: Expr {
                            line_number: self.coverage,
                            node: Box::new(Node::Lamda { args, value }),
                        },
                    }));
                }
            } else if line.trim().starts_with("return") {
                panic!(
                    "cannot return on top level ,line number:{}",
                    self.coverage + 1
                );
            } else {
                self.lex.push(LEX::EXPR(Expr {
                    line_number: self.coverage,
                    node: self.equation_resolver(&self.lex_vec[self.coverage]),
                }));
            }
            self.coverage = self.coverage + 1;
            self.start_lex();
        }
    }

    pub fn deep_lex(&mut self, mut sub_lex: &mut Vec<LEX>) {
        if !self.lex_vec[self.coverage].contains("}") {
            let line = self.lex_vec[self.coverage];
            if line.trim() == "" || line.trim().starts_with("//") {
            } else if line.contains(":") {
                let (name, value_str) = line.split_once(":").unwrap();
                if !value_str.ends_with("{") {
                    sub_lex.push(LEX::DEF(Def {
                        name: name.trim().to_string(),
                        line_number: self.coverage,
                        value: Expr {
                            line_number: self.coverage,
                            node: self.equation_resolver(&value_str),
                        },
                    }));
                } else {
                    self.coverage = self.coverage + 1;
                    let mut value: Vec<LEX> = Vec::new();
                    let mut args = value_str
                        .split_once("(")
                        .unwrap()
                        .1
                        .split_once(")")
                        .unwrap()
                        .0
                        .split(",")
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<String>>();

                    if args[0].trim() == "" {
                        args.pop();
                    }

                    self.deep_lex(&mut value);

                    sub_lex.push(LEX::DEF(Def {
                        name: name.trim().to_string(),
                        line_number: self.coverage,
                        value: Expr {
                            line_number: self.coverage,
                            node: Box::new(Node::Lamda { args, value }),
                        },
                    }));
                }
            } else if line.trim().starts_with("return") {
                let line = line.replace("return", "");
                sub_lex.push(LEX::RETURN(Expr {
                    line_number: self.coverage,
                    node: self.equation_resolver(&line),
                }))
            } else {
                sub_lex.push(LEX::EXPR(Expr {
                    line_number: self.coverage,
                    node: self.equation_resolver(&line),
                }))
            }
            self.coverage = self.coverage + 1;
            self.deep_lex(&mut sub_lex);
        }
    }

    pub fn equation_resolver(&self, string: &'a str) -> Box<Node> {
        let mut splited: Vec<&str> = string.split("").collect::<Vec<&str>>();
        let mut node = Box::new(Node::VOID);

        let mut foo = 0;
        let mut bracks = 0;

        // while foo < splited.len() {
        let mut part = Vec::new();
        loop {
            if JOINS_REGEX.is_match(splited[foo]) && bracks == 0 {
                let word = part.join("").trim().to_string();

                if word.starts_with("\"") && word.ends_with("\"") {
                    let word = word.trim_matches('\"').to_string();
                    node = Box::new(Node::Str(word));
                } else if word.contains("(") && word.contains(")") {
                    let (name, args_str) = word.split_once("(").unwrap();
                    let name = name.to_string();
                    let mut args_str = args_str.split("").collect::<Vec<&str>>();
                    args_str.pop();
                    args_str.pop();

                    let args_str = args_str.join("");
                    let args_str = args_str.split("").collect::<Vec<&str>>();
                    let mut args = Vec::new();
                    let mut part = Vec::new();
                    let mut bracks = 0;
                    let mut i = 0;
                    loop {
                        if args_str[i] == "," && bracks == 0 {
                            args.push(self.equation_resolver(part.join("").as_str()));
                            part = Vec::new();
                        } else {
                            part.push(args_str[i]);
                        }

                        if args_str[i] == "(" {
                            bracks = bracks + 1;
                        }
                        if args_str[i] == ")" {
                            bracks = bracks - 1;
                        }

                        i = i + 1;

                        if i == args_str.len() && bracks == 0 {
                            if part.join("").trim() != "" {
                                args.push(self.equation_resolver(part.join("").as_str()));
                            }

                            break;
                        } else if i == args_str.len() {
                            panic!("close the brackets")
                        }
                    }

                    node = Box::new(Node::FCCALL { args, name })
                } else if CALL_REGEX.is_match(&word) {
                    node = Box::new(Node::CALL(word))
                } else if INT_REGEX.is_match(&word) {
                    node = Box::new(Node::Int(word.parse().unwrap()));
                } else {
                    panic!("not a type : {}", word);
                }

                let joint = splited[foo].trim();

                let joint = match joint {
                    "+" => JOINT::ADD,
                    "-" => JOINT::SUB,
                    "/" => JOINT::DIV,
                    "*" => JOINT::MULT,
                    _ => panic!("operator not found : {}", joint),
                };

                let spliced = splited.splice(foo + 1.., vec![]).collect::<Vec<&str>>();

                let spliced = spliced.join("");

                node = Box::new(Node::OP {
                    joint,
                    lhs: node,
                    rhs: self.equation_resolver(spliced.as_str()),
                });
                break;
            }
            if splited[foo] == "(" {
                bracks = bracks + 1;
            }

            if splited[foo] == ")" {
                bracks = bracks - 1
            }

            part.push(splited[foo]);

            foo = foo + 1;

            if foo == splited.len() && bracks == 0 {
                let word = part.join("").trim().to_string();

                if word.starts_with("\"") && word.ends_with("\"") {
                    let word = word.trim_matches('\"').to_string();
                    node = Box::new(Node::Str(word));
                } else if word.contains("(") && word.contains(")") {
                    let (name, args_str) = word.split_once("(").unwrap();
                    let name = name.to_string();
                    let mut args_str = args_str.split("").collect::<Vec<&str>>();
                    args_str.pop();
                    args_str.pop();

                    let args_str = args_str.join("");
                    let args_str = args_str.split("").collect::<Vec<&str>>();
                    let mut args = Vec::new();
                    let mut part = Vec::new();
                    let mut bracks = 0;
                    let mut i = 0;
                    loop {
                        if args_str[i] == "," && bracks == 0 {
                            args.push(self.equation_resolver(part.join("").as_str()));
                            part = Vec::new();
                        } else {
                            part.push(args_str[i]);
                        }

                        if args_str[i] == "(" {
                            bracks = bracks + 1;
                        }
                        if args_str[i] == ")" {
                            bracks = bracks - 1;
                        }

                        i = i + 1;

                        if i == args_str.len() && bracks == 0 {
                            if part.join("").trim() != "" {
                                args.push(self.equation_resolver(part.join("").as_str()));
                            }

                            break;
                        } else if i == args_str.len() {
                            panic!("close the brackets")
                        }
                    }

                    node = Box::new(Node::FCCALL { args, name })
                } else if CALL_REGEX.is_match(&word) {
                    node = Box::new(Node::CALL(word))
                } else if INT_REGEX.is_match(&word) {
                    node = Box::new(Node::Int(word.parse().unwrap()));
                } else {
                    panic!("type not found");
                }
                break;
            } else if foo == splited.len() {
                panic!("bracket not closed {}", bracks);
            }
        }

        node
    }

    pub fn lex(&self) -> Vec<LEX> {
        self.lex.clone()
    }
}
