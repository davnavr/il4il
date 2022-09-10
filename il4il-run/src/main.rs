//! An IL4IL bytecode interpreter.

use il4il_vm::model::error_stack::{self, IntoReport, ResultExt};
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[clap(author, about, version)]
struct Arguments {
    #[clap(short, long)]
    program: Option<PathBuf>,
    #[clap(short, long)]
    reference: Vec<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
#[error("interpreter error")]
struct Error;

type Result<T> = error_stack::Result<T, Error>;

fn load_module<'env>(path: &std::path::Path) -> Result<il4il_vm::model::validation::ValidModule<'env>> {
    match il4il_vm::model::validation::ValidModule::read_from_path(path) {
        Ok(Ok(Ok(module))) => Ok(module),
        Ok(Ok(Err(validation_error))) => Err(validation_error).change_context(Error),
        Ok(Err(parser_error)) => Err(parser_error).change_context(Error),
        Err(bad_path) => Err(bad_path).report().change_context(Error),
    }
    .attach_printable_lazy(|| format!("could not load module {path:?}"))
}

fn main() -> Result<()> {
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

    let program_path = if let Some(path) = &interpreter_options.program {
        path
    } else {
        todo!("check current directory for a program")
    };

    let configuration = il4il_vm::runtime::configuration::Configuration::HOST;
    let runtime = il4il_vm::runtime::Runtime::with_configuration(configuration);

    std::thread::scope(|scope| {
        let host = il4il_vm::host::Host::with_runtime(&runtime, scope);
        let main_module = host.load_module(load_module(program_path)?);
        dbg!(main_module);
        Ok(())
    })
}
