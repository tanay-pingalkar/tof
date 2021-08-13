use crate::prelude::*;
use crate::tokenizer::*;
use std::collections::{BTreeMap, VecDeque};
use std::ops::Index;

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Lamda {
        args: Vec<String>,
        value: Vec<Tokens>,
    },
    Rusty(fn(args: Vec<Variable>) -> Option<Variable>),
    Int(f64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct Vars {
    vars: VecDeque<BTreeMap<String, Variable>>,
}

impl Vars {
    pub fn new() -> Vars {
        let mut list = VecDeque::new();
        list.push_back(BTreeMap::new());

        Vars { vars: list }
    }

    pub fn push(&mut self) {
        self.vars.push_back(BTreeMap::new());
    }

    pub fn pop(&mut self) {
        self.vars.pop_back();
    }

    pub fn insert(&mut self, id: String, data: Variable) {
        let m = self.vars.back_mut().unwrap();
        m.insert(id, data);
    }

    pub fn get(&mut self, id: &str, mut scope: usize) -> Option<(usize, &Variable)> {
        let mut var = None;

        if let Some(val) = self.vars.back().unwrap().get(id) {
            var = Some((scope, val))
        } else {
            while scope != 0 {
                if let Some(val) = self.vars.index(scope).get(id) {
                    var = Some((scope, val));
                    break;
                }
                scope = scope - 1;
                if scope == 0 {
                    if let Some(val) = self.vars.index(scope).get(id) {
                        var = Some((scope, val));
                    }
                    break;
                }
            }
        }

        var
    }

    pub fn scopes_number(&self) -> usize {
        self.vars.len()
    }
}

pub struct Runtime {
    data: Vars,
}

impl Runtime {
    pub fn new() -> Runtime {
        let mut data: Vars = Vars::new();
        prelude(&mut data);
        Runtime { data }
    }

    // pub fn start(lex: Vec<LEX>) -> Vars {
    //     let mut data: Vars = Vars::new();
    //     prelude(&mut data);
    //     eval(lex, &mut data, 1, vec![], vec![]);

    //     data
    // }

    pub fn eval(
        &mut self,
        tokens: Vec<Tokens>,
        scope: usize,
        args: Vec<Variable>,
        args_t_s: Vec<String>,
        drop: bool,
    ) -> Option<Variable> {
        let mut t = None;
        self.data.push();
        for (pos, e) in args_t_s.iter().enumerate() {
            self.data.insert(e.to_string(), args[pos].clone());
        }
        for line in tokens {
            match line.token {
                Token::Def(def) => {
                    self.eval_def(def, scope);
                }
                Token::Expr(expr) => {
                    self.eval_expr(expr, scope);
                }
                Token::Return(expr) => {
                    t = self.eval_expr(expr, scope);
                }
            }
        }
        if drop {
            self.data.pop();
        }

        t
    }

    pub fn eval_def(&mut self, def: Def, scope: usize) {
        let val = match self.eval_expr(def.value, scope) {
            Some(val) => val,
            None => {
                panic!("you cannot store void value");
            }
        };

        self.data.insert(def.name, val);
    }

    pub fn eval_expr(&mut self, expr: Box<Expr>, scope: usize) -> Option<Variable> {
        let mut v = None;

        match *expr {
            Expr::Int(int) => v = Some(Variable::Int(int)),
            Expr::Str(string) => v = Some(Variable::Str(string.to_string())),
            Expr::Lamda(Lamda { args, value }) => {
                v = Some(Variable::Lamda {
                    args: args.to_vec(),
                    value: value.to_vec(),
                })
            }
            Expr::FcCall(FcCall { args, name }) => {
                let mut tempd = self.data.clone();
                let (fc_scope, fc) = match tempd.get(&name, scope) {
                    Some(v) => v,
                    None => {
                        panic!("varible not found {}", name);
                    }
                };

                let args_t_s = args
                    .iter()
                    .map(|node| match self.eval_expr(node.clone(), scope) {
                        Some(val) => val,
                        None => {
                            panic!("sorry you cannot pass void value");
                        }
                    })
                    .collect();

                match fc {
                    Variable::Rusty(fnc) => v = fnc(args_t_s),
                    Variable::Lamda { args, value } => {
                        v = self.eval(value.to_vec(), fc_scope, args_t_s, args.clone(), true)
                    }
                    _ => {
                        panic!("not callable");
                    }
                }
            }
            Expr::Call(name) => {
                v = match self.data.get(&name, scope) {
                    Some(value) => Some(value.1.clone()),
                    None => {
                        panic!("variable not in scope {}", name);
                    }
                };
            }
            Expr::Op(Op { joint, lhs, rhs }) => {
                if let Some(lhs) = self.eval_expr(lhs, scope) {
                    if let Some(rhs) = self.eval_expr(rhs, scope) {
                        match joint {
                            JOINT::ADD => match lhs {
                                Variable::Int(int) => match rhs {
                                    Variable::Int(int2) => {
                                        v = Some(Variable::Int(int + int2));
                                    }
                                    _ => panic!("you can only add numbers and string"),
                                },
                                Variable::Str(string) => match rhs {
                                    Variable::Str(string2) => {
                                        v = Some(Variable::Str(string + &string2));
                                    }
                                    _ => panic!("you can only add numbers and string"),
                                },
                                _ => panic!("you can only add numsbers and string"),
                            },
                            JOINT::SUB => match lhs {
                                Variable::Int(int) => match rhs {
                                    Variable::Int(int2) => {
                                        v = Some(Variable::Int(int - int2));
                                    }
                                    _ => panic!("you can only subtract numbers"),
                                },
                                _ => panic!("you can only subtract numbers"),
                            },
                            JOINT::MULT => match lhs {
                                Variable::Int(int) => match rhs {
                                    Variable::Int(int2) => {
                                        v = Some(Variable::Int(int * int2));
                                    }
                                    _ => panic!("you can only multiply numbers"),
                                },
                                _ => panic!("you can only multiply numbers"),
                            },
                            JOINT::DIV => match lhs {
                                Variable::Int(int) => match rhs {
                                    Variable::Int(int2) => {
                                        v = Some(Variable::Int(int / int2));
                                    }
                                    _ => panic!("you can only divide numbers"),
                                },
                                _ => panic!("you can only divide numbers"),
                            },
                            JOINT::EQU => match lhs {
                                Variable::Int(int) => match rhs {
                                    Variable::Int(int2) => {
                                        v = Some(Variable::Bool(int == int2));
                                    }
                                    _ => panic!("you can only compare string and number"),
                                },
                                Variable::Str(string) => match rhs {
                                    Variable::Str(string2) => {
                                        v = Some(Variable::Bool(string == string2));
                                    }
                                    _ => panic!("you can only add numbers and string"),
                                },
                                _ => panic!("you can only compare string and number"),
                            },
                            JOINT::GREAT => match lhs {
                                Variable::Int(int) => match rhs {
                                    Variable::Int(int2) => {
                                        v = Some(Variable::Bool(int > int2));
                                    }
                                    _ => panic!("you can only compare string and number"),
                                },
                                _ => {
                                    panic!("only numbers are allowed");
                                }
                            },
                            JOINT::LESS => match lhs {
                                Variable::Int(int) => match rhs {
                                    Variable::Int(int2) => {
                                        v = Some(Variable::Bool(int < int2));
                                    }
                                    _ => panic!("you can only compare string and number"),
                                },
                                _ => {
                                    panic!("only numbers are allowed");
                                }
                            },
                            JOINT::NOT => match lhs {
                                Variable::Int(int) => match rhs {
                                    Variable::Int(int2) => {
                                        v = Some(Variable::Bool(int != int2));
                                    }
                                    _ => panic!("you can only compare string and number"),
                                },
                                Variable::Str(string) => match rhs {
                                    Variable::Str(string2) => {
                                        v = Some(Variable::Bool(string != string2));
                                    }
                                    _ => panic!("you can only add numbers and string"),
                                },
                                _ => panic!("you can only compare string and number"),
                            },
                        }
                    }
                };
            }

            Expr::Decision(mat) => {
                v = self.eval_match(mat, scope);
            }
            Expr::Bool(bool) => {
                v = Some(Variable::Bool(bool));
            }
            Expr::Scope(s) => {
                v = self.eval(s, self.data.scopes_number(), vec![], vec![], true);
            }
        }
        v
    }

    pub fn eval_match(&mut self, m: Decision, scope: usize) -> Option<Variable> {
        if self.eval_expr(m.cond, scope) == Some(Variable::Bool(true)) {
            self.eval(m.block, self.data.scopes_number(), vec![], vec![], true)
        } else {
            match m.next {
                Some(m) => self.eval_match(*m, scope),
                None => None,
            }
        }
    }
}
