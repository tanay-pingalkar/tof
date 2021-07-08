use crate::runtime::*;
use std::io::{stdin, stdout, Write};

pub fn stdio(var: &Variable) {
    match var {
        Variable::Lamda { args, value } => print!("args:{:#?} , value:{:#?}", args, value),
        Variable::Rusty(_) => print!("a rusty function"),
        Variable::Int(int) => print!("{}", int),
        Variable::Str(string) => print!("{}", string),
        Variable::Void => print!("VOID"),
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
            Variable::Void
        }),
    );

    data.insert(
        "scan".to_string(),
        Variable::Rusty(|args| {
            let mut string = String::new();
            stdio(&args[0]);

            stdin().read_line(&mut string).unwrap();
            string.pop();
            Variable::Str(string)
        }),
    );

    data.insert(
        "int".to_string(),
        Variable::Rusty(|args| {
            let int = match args[0].clone() {
                Variable::Int(int) => int,
                Variable::Str(string) => string.parse().unwrap(),
                _ => panic!("cannot parse"),
            };
            Variable::Int(int.clone())
        }),
    );
    data.insert(
        "len".to_string(),
        Variable::Rusty(|args| {
            let len = match args[0].clone() {
                Variable::Str(str) => str,
                _ => panic!("only give len of string"),
            };
            let len = len.len();
            Variable::Int(len as i32)
        }),
    );
}
