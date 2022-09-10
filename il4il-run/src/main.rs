//! An IL4IL bytecode interpreter.

use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[clap(author, about, version)]
struct Arguments {
    #[clap(short, long)]
    program: Option<PathBuf>,
    #[clap(short, long)]
    reference: Vec<PathBuf>,
}

fn main() {
    let mut program_arguments = Vec::new();
    let interpreter_options = {
        let mut environment_arguments = std::env::args();
        let mut interpreter_arguments = Vec::new();
        if let Some(environment_path) = environment_arguments.next() {
            interpreter_arguments.push(environment_path);
        }

        let mut has_program_arguments = false;
        for argument in environment_arguments {
            if has_program_arguments {
                program_arguments.push(argument);
            } else if argument == "--" {
                has_program_arguments = true;
            } else {
                interpreter_arguments.push(argument);
            }
        }

        <Arguments as clap::Parser>::parse_from(interpreter_arguments)
    };

    println!("{interpreter_options:?}");
    println!("{program_arguments:?}");
}
