use std::collections::HashMap;
use std::fs;
use crate::compile_source_code_to_ast;
use crate::env::LispyEnv;
use crate::types::LispyType;

pub struct LispyMachine {
    env: LispyEnv,
}

fn eval_ast(expression: &LispyType, env: &mut LispyEnv) -> Result<LispyType, LispyType> {
    match expression {
        LispyType::Symbol { .. } => {
            let result = env.get_item(expression.as_symbol().unwrap());
            return if result.is_some() {
                Ok(result.unwrap().clone())
            } else {
                Err(LispyType::Error {
                    message: format!(
                        "Symbol {} is not defined",
                        expression.as_symbol().unwrap()
                    ).to_string(),
                    error_type: "NOT_DEFINED".to_string(),
                    meta: HashMap::new(),
                })
            };
        }
        LispyType::List { .. } => {
            let items = expression.as_list().unwrap();
            let mut collection: Vec<LispyType> = vec![];

            for index in 0..items.len() {
                let evaluated = eval(
                    &items.get(index).unwrap().clone(),
                    env,
                );

                if evaluated.is_err() {
                    return evaluated;
                } else {
                    collection.push(evaluated.unwrap());
                }
            }

            Ok(LispyType::List {
                collection: Box::from(collection),
                meta: HashMap::new(),
            })
        }
        LispyType::Hash { .. } => {
            let mut collection = HashMap::new();
            for (key, value) in expression.as_hash().unwrap().iter() {
                let evaluated = eval(value, env);
                if evaluated.is_err() {
                    return evaluated;
                } else {
                    collection.insert(key.clone(), evaluated.unwrap());
                }
            }
            Ok(LispyType::Hash { collection: Box::from(collection), meta: HashMap::new() })
        }
        _ => {
            Ok(expression.clone())
        }
    }
}

pub fn eval(expression: &LispyType, env: &mut LispyEnv) -> Result<LispyType, LispyType> {
    match expression {
        LispyType::List { .. } => {
            if expression.as_list().unwrap().is_empty() {
                return Ok(expression.clone());
            }

            let first = expression.as_list().unwrap().first().unwrap();

            if first.is_symbol() {
                match first.as_symbol().unwrap().as_str() {
                    "def!" => {
                        let key = expression.as_list().unwrap().get(1).unwrap().clone();
                        let value = expression.as_list().unwrap().get(2).unwrap().clone();

                        if !key.is_symbol() {
                            return Err(LispyType::Error {
                                message: format!("def! first arg must be a symbol. Received: {}", key),
                                error_type: "INCORRECT_TYPE".to_string(),
                                meta: HashMap::new(),
                            });
                        }

                        let evaluated = eval(&value, env);
                        if evaluated.is_err() { return evaluated; }

                        env.set_item(key.as_symbol().unwrap().clone(), evaluated.as_ref().unwrap().clone());
                        return evaluated;
                    }
                    "let*" => {
                        let mut n_env = LispyEnv::child(env);
                        let bindings = expression.as_list().unwrap().get(1).unwrap().clone();
                        let to_eval = expression.as_list().unwrap().get(2).unwrap().clone();

                        if !bindings.is_list() || bindings.as_list().unwrap().len() % 2 != 0 {
                            return Err(LispyType::Error {
                                message: format!("let* first arg must be a list of key value pairs. Received: {}", bindings),
                                error_type: "INCORRECT_TYPE".to_string(),
                                meta: HashMap::new(),
                            });
                        }

                        for index in (0..bindings.as_list().unwrap().len()).step_by(2) {
                            let key = bindings.as_list().unwrap().get(index).unwrap().clone();
                            let value = bindings.as_list().unwrap().get(index + 1).unwrap().clone();
                            if !key.is_symbol() {
                                return Err(LispyType::Error {
                                    message: format!("let* bindings key must be a symbol. Received: {}", bindings),
                                    error_type: "INCORRECT_TYPE".to_string(),
                                    meta: HashMap::new(),
                                });
                            }
                            let evaluated = eval(&value, &mut n_env);
                            if evaluated.is_err() {
                                return evaluated;
                            }
                            n_env.set_item(key.as_symbol().unwrap().clone(), evaluated.unwrap());
                        }

                        return eval(&to_eval, &mut n_env);
                    }
                    "do" => {
                        for index in 1..expression.as_list().unwrap().len() {
                            let item = expression.as_list().unwrap().get(index).unwrap().clone();
                            let evaluated = eval(&item, env);
                            if evaluated.is_err() || index == expression.as_list().unwrap().len() - 1 {
                                return evaluated;
                            }
                        }
                    }
                    "if" => {
                        let cond = expression.as_list().unwrap().get(1).unwrap().clone();
                        let evaluated_condition = eval(&cond, env);
                        let to_eval = if evaluated_condition.unwrap().is_truthy() {
                            expression.as_list().unwrap().get(2).unwrap().clone()
                        } else {
                            expression.as_list().unwrap().get(3).unwrap().clone()
                        };

                        return eval(&to_eval, env);
                    }
                    "fn*" => {
                        let bindings = expression.as_list().unwrap().get(1).unwrap().clone().as_list().unwrap().clone();
                        let to_eval = expression.as_list().unwrap().get(2).unwrap().clone();
                        return Ok(LispyType::Lambda {
                            bindings,
                            to_eval: Box::new(to_eval),
                            env: Box::new(env.clone()),
                            meta: HashMap::new(),
                        });
                    }
                    _ => {}
                }
            }
            let evaluated = eval_ast(expression, env);
            if evaluated.is_err() {
                return evaluated;
            }
            let callee = evaluated.as_ref().unwrap().as_list().unwrap().get(0).unwrap().clone();
            let len = evaluated.as_ref().unwrap().as_list().unwrap().len();
            let arguments: Vec<LispyType> = evaluated.as_ref().unwrap().as_list().unwrap().get(1..len).unwrap().into();

            callee.apply_function(arguments)
        }
        _ => eval_ast(expression, env)
    }
}

impl LispyMachine {
    pub fn new() -> Self {
        let mut this = Self {
            env: LispyEnv::root()
        };

        this.evaluate_file("core.lispy");

        this
    }

    pub fn execute(&mut self, input_code: &str) {
        let ast = compile_source_code_to_ast(input_code);

        for expression in ast {
            let result = eval(&expression, &mut self.env);

            if result.is_err() {
                panic!("Error: {:?}", result.err().unwrap().as_error().unwrap().message);
            }
        }
    }

    pub fn evaluate_file(&mut self, filepath: &str) {
        let contents = fs::read_to_string(filepath)
            .expect(format!("File {} not found", filepath).as_str());
        self.execute(contents.as_str());
    }
}