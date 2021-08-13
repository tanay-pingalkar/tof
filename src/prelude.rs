use crate::runtime::*;
use rand::prelude::*;
use std::io::{stdin, stdout, Write};

pub fn stdio(var: &Variable) {
    match var {
        Variable::Lamda { args, value } => print!("args:{:#?} , value:{:#?}", args, value),
        Variable::Rusty(_) => print!("a rusty function"),
        Variable::Int(int) => print!("{}", int),
        Variable::Str(string) => print!("{}", string),
        Variable::Bool(bool) => print!("{}", bool),
    }
    stdout().flush().unwrap();
}

pub fn prelude(data: &mut Vars) {
    data.insert(
        "print".to_string(),
        Variable::Rusty(|args| {
            for var in args {
                stdio(&var);
            }
            stdio(&Variable::Str("\n".to_string()));
            None
        }),
    );

    data.insert(
        "scan".to_string(),
        Variable::Rusty(|args| {
            let mut string = String::new();
            stdio(&args[0]);

            stdin().read_line(&mut string).unwrap();
            string.pop();
            Some(Variable::Str(string))
        }),
    );

    data.insert(
        "int".to_string(),
        Variable::Rusty(|args| {
            let int = match &args[0] {
                Variable::Int(int) => int.clone(),
                Variable::Str(string) => string.parse::<f64>().unwrap(),
                _ => panic!("cannot parse"),
            };
            Some(Variable::Int(int.clone()))
        }),
    );
    data.insert(
        "len".to_string(),
        Variable::Rusty(|args| {
            let len = match &args[0] {
                Variable::Str(str) => str,
                _ => panic!("only give len of string"),
            };
            let len = len.len();
            Some(Variable::Int(len as f64))
        }),
    );
    data.insert(
        "rand".to_string(),
        Variable::Rusty(|_args| Some(Variable::Int(random::<f64>()))),
    );
    data.insert(
        "round".to_string(),
        Variable::Rusty(|args| match args[0] {
            Variable::Int(i) => Some(Variable::Int(i.round() as f64)),
            _ => panic!("only numbers please"),
        }),
    );
    data.insert(
        "quit".to_string(),
        Variable::Rusty(|args| {
            if args.len() == 1 {
                match args[0] {
                    Variable::Int(i) => std::process::exit(i as i32),
                    _ => panic!("only numbers please"),
                }
            } else {
                std::process::exit(100)
            }
        }),
    );
}
