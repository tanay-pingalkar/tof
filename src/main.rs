#[macro_use]
extern crate lazy_static;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use clap::{AppSettings, Clap};
use std::fs;

#[derive(Clap, Debug)]
#[clap(
    version = "1.0",
    author = "author - Tanay D. Pingalkar <tanaydpingalkar@gmail.com>",
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

    #[clap(long, about = "show generated tokens", short)]
    show_tokens: bool,
}

mod prelude;
mod runtime;
mod tokenizer;

use runtime::Runtime;

mod utils;
use tokenizer::Tokenizer;

fn main() {
    let matches: Opts = Opts::parse();

    match matches.subcommand {
        Subcommand::Run(Run { file, show_tokens }) => {
            let string = fs::read_to_string(format!("{}.tof", file)).expect("file not found");
            let mut tokenizer = Tokenizer::new(&string);
            tokenizer.start();
            if show_tokens {
                println!("{:#?}", tokenizer.tokens);
            }
            let mut runtime = Runtime::new();

            runtime.eval(tokenizer.tokens, 1, vec![], vec![], true);
        }
        Subcommand::Play => {
            let mut rl = Editor::<()>::new();

            println!("welcome to interactive mode \npress : Ctrl-C to exit");
            let mut runtime = Runtime::new();
            let mut i: usize = 1;
            loop {
                let readline = rl.readline("-> ");
                match readline {
                    Ok(line) => {
                        rl.add_history_entry(line.as_str());

                        let mut lexer = Tokenizer::new(&line);
                        lexer.start();

                        runtime.eval(lexer.tokens, i, vec![], vec![], false);

                        i = i + 1;
                    }
                    Err(ReadlineError::Interrupted) => {
                        println!("^C");
                        break;
                    }
                    Err(ReadlineError::Eof) => {
                        println!("^D");
                        break;
                    }
                    Err(err) => {
                        println!("Error: {:?}", err);
                        break;
                    }
                }
            }
        }
    }
}
