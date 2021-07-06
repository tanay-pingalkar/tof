#[macro_use]
extern crate lazy_static;

use clap::{AppSettings, Clap};
use std::fs;
use std::io::{stdin, stdout, Write};

#[derive(Clap, Debug)]
#[clap(
    version = "1.0",
    author = "author - Tanay D. Pingalkar <tanaydpingalkar@Gmail.com>",
    about = "a functional programming language"
)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
enum Subcommand {
    #[clap(about = "to run file, example : `tof run filename`")]
    Run(Run),

    #[clap(about = "to enter interactive mode")]
    Play,
}

#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Run {
    #[clap(about = "file name of .tof extension , example `tof run filename`")]
    file: String,
}

mod lexer;
mod prelude;
mod runtime;

use crate::lexer::Lexer;

use crate::runtime::start;

fn main() {
    let matches: Opts = Opts::parse();

    match matches.subcommand {
        Subcommand::Run(Run { file }) => {
            let string = fs::read_to_string(format!("{}.tof", file)).expect("file not found");
            let mut lexer = Lexer::new(&string);
            lexer.start_lex();
            start(lexer.lex());
        }
        Subcommand::Play => {
            println!("welcome to interactive mode \npress : Ctrl-C to exit");
            loop {
                let mut string = String::new();
                print!("-> ");
                stdout().flush().unwrap();

                stdin().read_line(&mut string).unwrap();

                let mut lexer = Lexer::new(&string);
                lexer.start_lex();

                start(lexer.lex());
            }
        }
    }
}
