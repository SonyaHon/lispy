extern crate core;

use types::LispyType;

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
    lispy_machine
        .get_env_mut()
        .set("message-to-print", LispyType::create_string("Hello"));

    lispy_machine.evaluate_file("demo.lispy");

    println!(
        "{}",
        lispy_machine
            .get_env()
            .get_item(&"message-to-print".to_string())
            .unwrap()
            .as_string()
            .unwrap()
    );
}
