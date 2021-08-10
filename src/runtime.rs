use crate::lexer::*;
use crate::prelude::*;
use std::collections::{BTreeMap, VecDeque};
use std::ops::Index;

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Lamda { args: Vec<String>, value: Vec<LEX> },
    Rusty(fn(args: Vec<Variable>) -> Variable),
    Int(f64),
    Str(String),
    Bool(bool),
    Void,
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

    pub fn get(&mut self, id: &str, mut scope: usize) -> (usize, &Variable) {
        let mut var = &Variable::Void;

        if let Some(val) = self.vars.back().unwrap().get(id) {
            var = val
        } else {
            while scope != 0 {
                if let Some(val) = self.vars.index(scope).get(id) {
                    var = val;
                    break;
                }
                scope = scope - 1;
                if scope == 0 {
                    if let Some(val) = self.vars.index(scope).get(id) {
                        var = val;
                        break;
                    } else {
                        panic!("variable not in scope : {} {}", id, scope);
                    }
                }
            }
        }

        (scope, var)
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
        lex: Vec<LEX>,
        scope: usize,
        args: Vec<Variable>,
        args_t_s: Vec<String>,
        drop: bool,
    ) -> Variable {
        let mut t = Variable::Void;
        self.data.push();
        for (pos, e) in args_t_s.iter().enumerate() {
            self.data.insert(e.to_string(), args[pos].clone());
        }
        for line in lex {
            match line {
                LEX::DEF(_ln, def) => {
                    self.eval_def(def, scope);
                }
                LEX::EXPR(_ln, expr) => {
                    self.eval_expr(expr, scope);
                }
                LEX::RETURN(_ln, expr) => {
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
        let val = self.eval_expr(def.value, scope);
        self.data.insert(def.name, val);
    }

    pub fn eval_expr(&mut self, expr: Expr, scope: usize) -> Variable {
        let mut v = Variable::Void;

        match *expr {
            NODE::INT(int) => v = Variable::Int(int),
            NODE::STR(string) => v = Variable::Str(string),
            NODE::LAMDA(Lamda { args, value }) => v = Variable::Lamda { args, value },
            NODE::FCCALL(FcCall { args, name }) => {
                let mut fc = self.data.clone();

                let (fc_scope, fc) = fc.get(&name, scope);

                let args_t_s = args
                    .iter()
                    .map(|node| self.eval_expr(node.clone(), scope))
                    .collect();

                match fc {
                    Variable::Rusty(fnc) => {
                        v = fnc(args_t_s);
                    }
                    Variable::Lamda { args, value } => {
                        v = self.eval(value.clone(), fc_scope, args_t_s, args.clone(), true);
                    }
                    _ => {
                        panic!("not callable");
                    }
                }
            }
            NODE::CALL(name) => {
                v = self.data.get(&name, scope).1.clone();
            }
            NODE::VOID => v = Variable::Void,
            NODE::OP { joint, lhs, rhs } => {
                let lhs = self.eval_expr(lhs, scope);
                let rhs = self.eval_expr(rhs, scope);
                match joint {
                    JOINT::ADD => match lhs {
                        Variable::Int(int) => match rhs {
                            Variable::Int(int2) => {
                                v = Variable::Int(int + int2);
                            }
                            _ => panic!("you can only add numbers and string"),
                        },
                        Variable::Str(string) => match rhs {
                            Variable::Str(string2) => {
                                v = Variable::Str(string + &string2);
                            }
                            _ => panic!("you can only add numbers and string"),
                        },
                        _ => panic!("you can only add numsbers and string"),
                    },
                    JOINT::SUB => match lhs {
                        Variable::Int(int) => match rhs {
                            Variable::Int(int2) => {
                                v = Variable::Int(int - int2);
                            }
                            _ => panic!("you can only subtract numbers"),
                        },
                        _ => panic!("you can only subtract numbers"),
                    },
                    JOINT::MULT => match lhs {
                        Variable::Int(int) => match rhs {
                            Variable::Int(int2) => {
                                v = Variable::Int(int * int2);
                            }
                            _ => panic!("you can only multiply numbers"),
                        },
                        _ => panic!("you can only multiply numbers"),
                    },
                    JOINT::DIV => match lhs {
                        Variable::Int(int) => match rhs {
                            Variable::Int(int2) => {
                                v = Variable::Int(int / int2);
                            }
                            _ => panic!("you can only divide numbers"),
                        },
                        _ => panic!("you can only divide numbers"),
                    },
                    JOINT::EQU => match lhs {
                        Variable::Int(int) => match rhs {
                            Variable::Int(int2) => {
                                v = Variable::Bool(int == int2);
                            }
                            _ => panic!("you can only compare string and number"),
                        },
                        Variable::Str(string) => match rhs {
                            Variable::Str(string2) => {
                                v = Variable::Bool(string == string2);
                            }
                            _ => panic!("you can only add numbers and string"),
                        },
                        _ => panic!("you can only compare string and number"),
                    },
                    JOINT::GREAT => todo!(),
                    JOINT::LESS => todo!(),
                    JOINT::NOT => match lhs {
                        Variable::Int(int) => match rhs {
                            Variable::Int(int2) => {
                                v = Variable::Bool(int != int2);
                            }
                            _ => panic!("you can only compare string and number"),
                        },
                        Variable::Str(string) => match rhs {
                            Variable::Str(string2) => {
                                v = Variable::Bool(string != string2);
                            }
                            _ => panic!("you can only add numbers and string"),
                        },
                        _ => panic!("you can only compare string and number"),
                    },
                }
            }

            NODE::MATCH(mat) => {
                v = self.eval_match(mat, scope);
            }
            NODE::BOOL(bool) => {
                v = Variable::Bool(bool);
            }
            NODE::SCOPE(s) => {
                v = self.eval(s, self.data.scopes_number(), vec![], vec![], true);
            }
        }
        v
    }

    pub fn eval_match(&mut self, m: Box<Match>, scope: usize) -> Variable {
        if self.eval_expr(m.cond, scope) == Variable::Bool(true) {
            self.eval(m.block, self.data.scopes_number(), vec![], vec![], true)
        } else {
            match m.next {
                Some(m) => self.eval_match(m, scope),
                None => Variable::Void,
            }
        }
    }
}
