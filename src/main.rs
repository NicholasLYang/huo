mod ast;
mod compiler;
mod parser;
mod type_checker;

use crate::parser::parse;
use crate::type_checker::TypeChecker;
use compiler::Compiler;
use miette::Report;
use std::env;
use std::sync::Arc;

fn main() -> Result<(), anyhow::Error> {
    let file_path = env::args().nth(1).unwrap();
    let code = std::fs::read_to_string(&file_path)?;

    let program = parse(&code)?;
    let mut type_checker = TypeChecker::default();
    type_checker.check_program(&program);

    let code = Arc::new(code);
    for error in type_checker.into_errors() {
        println!("{:?}", Report::new(error).with_source_code(code.clone()));
    }

    let mut compiler = Compiler::default();
    compiler.compile_program(program);
    println!("==================");
    compiler.print()?;
    println!("==================");

    Ok(())
}
