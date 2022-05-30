use std::collections::HashMap;
use crate::LexerToken;
use crate::types::LispyType;
use logos::Logos;

struct TokenReader {
    index: usize,
    data: Vec<LexerToken>,
}

impl TokenReader {
    pub fn new(data: Vec<LexerToken>) -> Self {
        Self {
            index: 0,
            data,
        }
    }

    pub fn peek(&self) -> LexerToken {
        self.data[self.index].clone()
    }

    pub fn grab(&mut self) -> LexerToken {
        let token = self.peek().clone();
        self.index += 1;
        token
    }

    pub fn is_empty(&self) -> bool {
        self.index >= self.data.len()
    }
}

fn build_any_form(reader: &mut TokenReader) -> LispyType {
    match reader.peek() {
        LexerToken::Nil => {
            reader.grab();
            LispyType::Nil { meta: HashMap::new() }
        }

        LexerToken::Boolean(val) => {
            reader.grab();
            LispyType::Bool { value: val.clone(), meta: HashMap::new() }
        }
        LexerToken::String(val) => {
            reader.grab();
            LispyType::String { value: val.clone()[1..val.len() - 1].to_string().clone(), meta: HashMap::new() }
        }
        LexerToken::Number(val) => {
            reader.grab();
            LispyType::Number { value: val.clone(), meta: HashMap::new() }
        }
        LexerToken::Keyword(val) => {
            reader.grab();
            LispyType::Keyword { value: val.clone(), meta: HashMap::new() }
        }
        LexerToken::Symbol(val) => {
            reader.grab();
            LispyType::Symbol { value: val.clone(), meta: HashMap::new() }
        }

        LexerToken::ListStart => {
            reader.grab();
            let mut collection = vec![];

            while reader.peek() != LexerToken::ListEnd {
                collection.push(build_any_form(reader));
            }

            reader.grab();
            LispyType::List { collection: Box::from(collection), meta: HashMap::new() }
        }
        LexerToken::HashStart => {
            reader.grab();
            let mut collection = HashMap::new();

            while reader.peek() != LexerToken::HashEnd {
                let key = build_any_form(reader);
                let value = build_any_form(reader);
                collection.insert(key, value);
            }

            reader.grab();
            LispyType::Hash { collection: Box::from(collection), meta: HashMap::new() }
        }

        _ => panic!("Unknown token {:?}", reader.peek())
    }
}

fn build_from_tokens(reader: &mut TokenReader) -> Vec<LispyType> {
    let mut ast = vec![];

    while !reader.is_empty() {
        ast.push(build_any_form(reader));
    }

    ast
}


pub fn compile_source_code_to_ast(source_code: &str) -> Vec<LispyType> {
    let tokens: Vec<LexerToken> = LexerToken::lexer(source_code).collect();
    let mut reader = TokenReader::new(tokens);
    build_from_tokens(&mut reader)
}