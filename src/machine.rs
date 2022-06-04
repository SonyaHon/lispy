use crate::compile_source_code_to_ast;
use crate::env::LispyEnv;
use crate::types::LispyType;
use std::collections::HashMap;
use std::fs;

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
                    message: format!("Symbol {} is not defined", expression.as_symbol().unwrap())
                        .to_string(),
                    error_type: "NOT_DEFINED".to_string(),
                    meta: HashMap::new(),
                })
            };
        }
        LispyType::List { .. } => {
            let items = expression.as_list().unwrap();
            let mut collection: Vec<LispyType> = vec![];

            for index in 0..items.len() {
                let evaluated = eval(&items.get(index).unwrap().clone(), env);

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
            Ok(LispyType::Hash {
                collection: Box::from(collection),
                meta: HashMap::new(),
            })
        }
        _ => Ok(expression.clone()),
    }
}

pub fn quasi_quote(ast: &LispyType) -> LispyType {
    if ast.is_list()
        && ast
            .as_list()
            .unwrap()
            .get(0)
            .unwrap()
            .is_symbol_containing("unquote")
    {
        return ast.as_list().unwrap().get(1).unwrap().clone();
    }
    if ast.is_list() {
        let mut result = vec![];
        for elt in ast.as_list().unwrap().iter().rev() {
            if elt.is_list()
                && elt
                    .as_list()
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .is_symbol_containing("splice-unquote")
            {
                result = vec![
                    LispyType::create_symbol("concat"),
                    elt.as_list().unwrap().get(1).unwrap().clone(),
                    LispyType::create_list(result),
                ];
            } else {
                result = vec![
                    LispyType::create_symbol("cons"),
                    quasi_quote(elt),
                    LispyType::create_list(result),
                ];
            }
        }
        return LispyType::create_list(result);
    }
    if ast.is_hash() || ast.is_symbol() {
        return LispyType::create_list(vec![LispyType::create_symbol("quote"), ast.clone()]);
    }
    return ast.clone();
}

pub fn is_macro_call(ast: &LispyType, env: &LispyEnv) -> bool {
    if !ast.is_list() {
        return false;
    }
    if !ast.as_list().unwrap().first().is_some() {
        return false;
    }

    let item = ast.as_list().unwrap().first().unwrap().clone();
    if !item.is_symbol() {
        return false;
    }
    let in_env = env.get_item(item.as_symbol().unwrap());
    if in_env.is_none() {
        return false;
    }
    in_env.unwrap().is_macro()
}

pub fn macro_expand(ast: &LispyType, passed_env: &LispyEnv) -> Result<LispyType, LispyType> {
    let mut ast = ast.clone();
    let env = passed_env.clone();
    while is_macro_call(&ast, &env) {
        let callee_symbol = ast.as_list().unwrap().first().unwrap().clone();
        let callee = env.get_item(callee_symbol.as_symbol().unwrap()).unwrap();
        let len = ast.as_list().unwrap().len();
        let args: Vec<LispyType> = ast.as_list().unwrap().get(1..len).unwrap().into();
        let result = callee.apply_lambda(args);
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        let mut unwrapped = result.unwrap();
        let result = eval(&unwrapped.0, &mut unwrapped.1);
        if result.is_err() {
            return result;
        }
        ast = result.unwrap();
    }

    Ok(ast)
}

pub fn eval(
    passed_expression: &LispyType,
    passed_env: &mut LispyEnv,
) -> Result<LispyType, LispyType> {
    let mut env = passed_env.clone();
    let mut expression = passed_expression.clone();

    loop {
        match expression {
            LispyType::List { .. } => {
                let macro_expand_result = macro_expand(&expression, &mut env);

                if macro_expand_result.is_err() {
                    return macro_expand_result;
                }

                if !macro_expand_result.as_ref().unwrap().is_list() {
                    return eval_ast(macro_expand_result.as_ref().unwrap(), &mut env);
                }
                expression = macro_expand_result.unwrap();

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
                                    message: format!(
                                        "def! first arg must be a symbol. Received: {}",
                                        key
                                    ),
                                    error_type: "INCORRECT_TYPE".to_string(),
                                    meta: HashMap::new(),
                                });
                            }

                            let evaluated = eval(&value, &mut env);
                            if evaluated.is_err() {
                                return evaluated;
                            }

                            env.set_item(
                                key.as_symbol().unwrap().clone(),
                                evaluated.as_ref().unwrap().clone(),
                            );
                            passed_env.modify_with(&env);
                            return evaluated;
                        }
                        "defmacro!" => {
                            let key = expression.as_list().unwrap().get(1).unwrap().clone();
                            let value = expression.as_list().unwrap().get(2).unwrap().clone();

                            if !key.is_symbol() {
                                return Err(LispyType::Error {
                                    message: format!(
                                        "defmacro! first arg must be a symbol. Received: {}",
                                        key
                                    ),
                                    error_type: "INCORRECT_TYPE".to_string(),
                                    meta: HashMap::new(),
                                });
                            }

                            let evaluated = eval(&value, &mut env);
                            if evaluated.is_err() {
                                return evaluated;
                            }
                            if !evaluated.as_ref().unwrap().is_lambda() {
                                return Err(LispyType::Error {
                                    message: format!(
                                        "defmacro! second arg must be a lambda. Received: {}",
                                        key
                                    ),
                                    error_type: "INCORRECT_TYPE".to_string(),
                                    meta: HashMap::new(),
                                });
                            }

                            let evaluated = evaluated.unwrap().convert_to_macro();

                            env.set_item(key.as_symbol().unwrap().clone(), evaluated.clone());
                            passed_env.modify_with(&env);
                            return Ok(evaluated);
                        }
                        "deferror!" => {
                            let symbol = expression
                                .as_list()
                                .unwrap()
                                .get(1)
                                .unwrap()
                                .as_symbol()
                                .unwrap();
                            let error_type = expression
                                .as_list()
                                .unwrap()
                                .get(2)
                                .unwrap()
                                .as_string()
                                .unwrap();

                            env.set_item(
                                symbol.clone(),
                                LispyType::create_error(
                                    &error_type.as_str(),
                                    &symbol.clone().as_str(),
                                ),
                            );
                            passed_env.modify_with(&env);
                            return Ok(LispyType::create_nil());
                        }
                        "let*" => {
                            let mut n_env = LispyEnv::child(&mut env);
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
                                let value =
                                    bindings.as_list().unwrap().get(index + 1).unwrap().clone();
                                if !key.is_symbol() {
                                    return Err(LispyType::Error {
                                        message: format!(
                                            "let* bindings key must be a symbol. Received: {}",
                                            bindings
                                        ),
                                        error_type: "INCORRECT_TYPE".to_string(),
                                        meta: HashMap::new(),
                                    });
                                }
                                let evaluated = eval(&value, &mut n_env);
                                if evaluated.is_err() {
                                    return evaluated;
                                }
                                n_env
                                    .set_item(key.as_symbol().unwrap().clone(), evaluated.unwrap());
                            }

                            env = n_env;
                            expression = to_eval;
                            continue;
                        }
                        "do" => {
                            if expression.as_list().unwrap().len() == 1 {
                                return Ok(LispyType::create_nil());
                            }
                            if expression.as_list().unwrap().len() == 2 {
                                let last_expr = expression
                                    .as_list()
                                    .unwrap()
                                    .get(expression.as_list().unwrap().len() - 1)
                                    .unwrap()
                                    .clone();
                                expression = last_expr;
                                continue;
                            }
                            for index in 1..expression.as_list().unwrap().len() - 1 {
                                let item =
                                    expression.as_list().unwrap().get(index).unwrap().clone();
                                let evaluated = eval(&item, &mut env);
                                if evaluated.is_err() {
                                    return evaluated;
                                }
                            }
                            let last_expr = expression
                                .as_list()
                                .unwrap()
                                .get(expression.as_list().unwrap().len() - 1)
                                .unwrap()
                                .clone();
                            expression = last_expr;
                            continue;
                        }
                        "if" => {
                            let cond = expression.as_list().unwrap().get(1).unwrap().clone();
                            let evaluated_condition = eval(&cond, &mut env);
                            let to_eval = if evaluated_condition.unwrap().is_truthy() {
                                expression.as_list().unwrap().get(2).unwrap().clone()
                            } else {
                                expression.as_list().unwrap().get(3).unwrap().clone()
                            };

                            expression = to_eval;
                            continue;
                        }
                        "fn*" => {
                            let bindings = expression
                                .as_list()
                                .unwrap()
                                .get(1)
                                .unwrap()
                                .clone()
                                .as_list()
                                .unwrap()
                                .clone();
                            let to_eval = expression.as_list().unwrap().get(2).unwrap().clone();
                            return Ok(LispyType::Lambda {
                                bindings,
                                to_eval: Box::new(to_eval),
                                env: Box::new(env.clone()),
                                meta: HashMap::new(),
                                is_macro: false,
                            });
                        }
                        "eval" => {
                            let unevaluated_expr =
                                expression.as_list().unwrap().get(1).unwrap().clone();
                            let evaluated_expr = eval(&unevaluated_expr, &mut env);
                            if evaluated_expr.is_err() {
                                return evaluated_expr;
                            }
                            expression = evaluated_expr.unwrap();
                            continue;
                        }
                        "quote" => {
                            return Ok(expression.as_list().unwrap().get(1).unwrap().clone());
                        }
                        "quasi-quote-expand" => {
                            return Ok(quasi_quote(
                                &expression.as_list().unwrap().get(1).unwrap().clone(),
                            ));
                        }
                        "quasi-quote" => {
                            expression =
                                quasi_quote(&expression.as_list().unwrap().get(1).unwrap().clone());
                            continue;
                        }
                        "macro-expand" => {
                            return macro_expand(
                                expression.as_list().unwrap().get(1).unwrap(),
                                &env,
                            );
                        }
                        "throw" => {
                            let error =
                                eval_ast(expression.as_list().unwrap().get(1).unwrap(), &mut env);
                            if error.is_err() {
                                return error;
                            }
                            return Err(error.unwrap());
                        }
                        "try*" => {
                            let result = eval(
                                &expression.as_list().unwrap().get(1).unwrap().clone(),
                                &mut env,
                            );

                            if result.is_ok() {
                                return Ok(result.unwrap());
                            }

                            let catch_clauses: Vec<LispyType> = expression
                                .as_list()
                                .unwrap()
                                .get(2..expression.as_list().unwrap().len())
                                .unwrap()
                                .iter()
                                .map(|item| item.clone())
                                .collect();

                            for catch_clause in catch_clauses.iter() {
                                let gotten_error = eval_ast(
                                    &catch_clause.as_list().unwrap().get(1).unwrap().clone(),
                                    &mut env,
                                );
                                if gotten_error.is_err() {
                                    return gotten_error;
                                }
                                if gotten_error.unwrap() == result.as_ref().err().unwrap().clone() {
                                    return eval(
                                        &catch_clause.as_list().unwrap().get(2).unwrap().clone(),
                                        &mut env,
                                    );
                                }
                            }
                            return result;
                        }
                        _ => {}
                    }
                }
                let evaluated = eval_ast(&expression, &mut env);
                if evaluated.is_err() {
                    return evaluated;
                }
                let callee = evaluated
                    .as_ref()
                    .unwrap()
                    .as_list()
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .clone();
                let len = evaluated.as_ref().unwrap().as_list().unwrap().len();
                let arguments: Vec<LispyType> = evaluated
                    .as_ref()
                    .unwrap()
                    .as_list()
                    .unwrap()
                    .get(1..len)
                    .unwrap()
                    .into();

                if callee.is_function() && !callee.is_lambda() {
                    return callee.apply_function(arguments);
                }

                let parse = callee.apply_lambda(arguments);
                if parse.is_err() {
                    return Err(parse.err().unwrap());
                }
                let unwrapped = parse.unwrap();
                expression = unwrapped.0;
                env = unwrapped.1;
                continue;
            }
            _ => return eval_ast(&expression, &mut env),
        }
    }
}

impl LispyMachine {
    pub fn new() -> Self {
        let mut this = Self {
            env: LispyEnv::root(),
        };

        this.evaluate_file("lispy_std/core.lispy");

        this
    }

    pub fn execute(&mut self, input_code: &str) {
        let ast = compile_source_code_to_ast(input_code);

        for expression in ast {
            let result = eval(&expression, &mut self.env);

            if result.is_err() {
                panic!(
                    "Error: {:?}",
                    result.err().unwrap().as_error().unwrap().message
                );
            }
        }
    }

    pub fn evaluate_file(&mut self, filepath: &str) {
        let contents =
            fs::read_to_string(filepath).expect(format!("File {} not found", filepath).as_str());
        self.execute(contents.as_str());
    }
}
