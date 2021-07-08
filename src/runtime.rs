use crate::lexer::*;
use crate::prelude::*;
use std::collections::{BTreeMap, VecDeque};
use std::ops::Index;

#[derive(Debug, Clone)]
pub enum Variable {
    Lamda { args: Vec<String>, value: Vec<LEX> },
    Rusty(fn(args: Vec<Variable>) -> Variable),
    Int(i32),
    Str(String),
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
        // for (i, m) in self.vars.iter().enumerate().rev() {
        //     if let Some(val) = m.get(id) {
        //         return Some(val);
        //     }
        // }

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
                        panic!("variable not in scope")
                    }
                }
            }
        }

        (scope, var)
    }
}

pub fn start(lex: Vec<LEX>) -> Vars {
    let mut data: Vars = Vars::new();
    prelude(&mut data);
    eval(lex, &mut data, 1, vec![], vec![]);

    data
}

pub fn eval(
    lex: Vec<LEX>,
    data: &mut Vars,
    scope: usize,
    args: Vec<Variable>,
    args_t_s: Vec<String>,
) -> Variable {
    let mut t = Variable::Void;
    data.push();
    for (pos, e) in args_t_s.iter().enumerate() {
        data.insert(e.to_string(), args[pos].clone());
    }
    for line in lex {
        match line {
            LEX::DEF(def) => {
                eval_def(def, data, scope);
            }
            LEX::EXPR(expr) => {
                eval_expr(expr, data, scope);
            }
            LEX::RETURN(expr) => {
                t = eval_expr(expr, data, scope);
            }
            LEX::MATCH => {}
        }
    }
    data.pop();
    t
}

pub fn eval_def(def: Def, data: &mut Vars, scope: usize) {
    let val = eval_expr(def.value, data, scope);
    data.insert(def.name, val);
}

pub fn eval_expr(expr: Expr, mut data: &mut Vars, scope: usize) -> Variable {
    let v;

    match *expr.node {
        Node::Int(int) => v = Variable::Int(int),
        Node::Str(string) => v = Variable::Str(string),
        Node::Lamda { args, value } => v = Variable::Lamda { args, value },
        Node::FCCALL { ref args, ref name } => {
            let mut fc = data.clone();

            let (fc_scope, fc) = fc.get(&name, scope);

            let args_t_s = args
                .iter()
                .map(|node| {
                    eval_expr(
                        Expr {
                            line_number: expr.line_number.clone(),
                            node: node.clone(),
                        },
                        data,
                        scope,
                    )
                })
                .collect();

            match fc {
                Variable::Rusty(fnc) => {
                    v = fnc(args_t_s);
                }
                Variable::Lamda { args, value } => {
                    v = eval(value.clone(), data, fc_scope, args_t_s, args.clone());
                }
                _ => {
                    panic!("not callable");
                }
            }
        }
        Node::CALL(name) => {
            v = data.get(&name, scope).1.clone();
        }
        Node::VOID => v = Variable::Void,
        Node::OP { joint, lhs, rhs } => {
            let lhs = eval_expr(
                Expr {
                    line_number: expr.line_number,
                    node: lhs,
                },
                &mut data,
                scope,
            );
            let rhs = eval_expr(
                Expr {
                    line_number: expr.line_number,
                    node: rhs,
                },
                &mut data,
                scope,
            );
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
            }
        }
    }
    v
}
