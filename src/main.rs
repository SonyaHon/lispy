use crate::lexer::LexerToken;
use crate::compiler::compile_source_code_to_ast;
use crate::machine::LispyMachine;

mod lexer;
mod compiler;
mod types;
mod machine;
mod env;
mod core_ns;

fn main() {
    let input = r#"
        (println
            (not (empty? (list))))
    "#;

    let mut lispy_machine = LispyMachine::new();
    lispy_machine.execute(input);
}