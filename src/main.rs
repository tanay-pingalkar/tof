#[macro_use]
extern crate lazy_static;

use clap::{AppSettings, Clap};
use std::fs;

#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    file: String,

    #[clap(long)]
    show_lex: bool,
}

mod lexer;
mod runtime;

use crate::lexer::Lexer;

use crate::runtime::start;

fn main() {
    let Opts { file, show_lex }: Opts = Opts::parse();
    let string = fs::read_to_string(format!("{}.tof", file)).expect("file not found");

    let mut lexer = Lexer::new(&string);
    lexer.start_lex();

    start(lexer.lex());

    if show_lex {
        fs::write(format!("{}.lex.tof", file), format!("{:#?}", lexer.lex())).unwrap();
    }
}
