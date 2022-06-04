extern crate core;

use crate::compiler::compile_source_code_to_ast;
use crate::lexer::LexerToken;
use crate::machine::LispyMachine;

mod compiler;
mod core_ns;
mod env;
mod lexer;
mod machine;
mod types;

fn main() {
    let mut lispy_machine = LispyMachine::new();
    lispy_machine.evaluate_file("demo.lispy");
}
