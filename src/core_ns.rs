use crate::compile_source_code_to_ast;
use crate::env::LispyEnv;
use crate::types::LispyType;
use std::collections::HashMap;
use std::fs;

pub fn apply_core_ns(env: &mut LispyEnv) {
    //#region Math
    env.set(
        "+",
        LispyType::create_function(Some(2), |args| {
            return Ok(args[0].clone() + args[1].clone());
        }),
    );

    env.set(
        "-",
        LispyType::create_function(Some(2), |args| {
            return Ok(args[0].clone() - args[1].clone());
        }),
    );

    env.set(
        "*",
        LispyType::create_function(Some(2), |args| {
            return Ok(args[0].clone() * args[1].clone());
        }),
    );

    env.set(
        "/",
        LispyType::create_function(Some(2), |args| {
            return Ok(args[0].clone() / args[1].clone());
        }),
    );

    //#endregion
    //#region Utility
    env.set(
        "println",
        LispyType::create_function(None, |args| {
            let mut str = "".to_string();
            args.iter()
                .for_each(|item| str += &format!("{}", item).to_string());
            println!("{}", str);
            Ok(LispyType::Nil {
                meta: HashMap::new(),
            })
        }),
    );

    env.set(
        "print",
        LispyType::create_function(None, |args| {
            let mut str = "".to_string();
            args.iter()
                .for_each(|item| str += &format!("{}", item).to_string());
            print!("{}", str);
            Ok(LispyType::Nil {
                meta: HashMap::new(),
            })
        }),
    );

    env.set(
        "list",
        LispyType::create_function(None, |args| {
            Ok(LispyType::List {
                collection: Box::from(args.clone()),
                meta: HashMap::new(),
            })
        }),
    );
    env.set(
        "count",
        LispyType::create_function(Some(1), |args| {
            let res = args[0].len();
            if res.is_error() {
                return Err(res.clone());
            }
            Ok(res.clone())
        }),
    );
    env.set(
        "cons",
        LispyType::create_function(None, |args| {
            let mut collection = vec![];

            for i in 0..args.len() - 1 {
                collection.push(args.get(i).unwrap().clone());
            }
            let last = args.get(args.len() - 1).unwrap().as_list().unwrap().clone();

            for item in last.iter() {
                collection.push(item.clone());
            }

            Ok(LispyType::List {
                collection: Box::new(collection),
                meta: HashMap::new(),
            })
        }),
    );
    env.set(
        "concat",
        LispyType::create_function(None, |args| {
            let mut collection = vec![];

            for x in args {
                for item in x.as_list().unwrap().iter() {
                    collection.push(item.clone());
                }
            }

            Ok(LispyType::List {
                collection: Box::new(collection),
                meta: HashMap::new(),
            })
        }),
    );

    env.set(
        "first",
        LispyType::create_function(Some(1), |args| args[0].first()),
    );

    env.set(
        "rest",
        LispyType::create_function(Some(1), |args| args[0].rest()),
    );

    env.set(
        "nth",
        LispyType::create_function(Some(2), |args| {
            args[0].nth(args[1].as_number().unwrap().clone() as usize)
        }),
    );
    //#endregion
    //#region is_?
    env.set(
        "nil?",
        LispyType::create_function(Some(1), |args| Ok(LispyType::create_bool(args[0].is_nil()))),
    );
    env.set(
        "bool?",
        LispyType::create_function(Some(1), |args| {
            Ok(LispyType::create_bool(args[0].is_bool()))
        }),
    );
    env.set(
        "symbol?",
        LispyType::create_function(Some(1), |args| {
            Ok(LispyType::create_bool(args[0].is_symbol()))
        }),
    );
    env.set(
        "number?",
        LispyType::create_function(Some(1), |args| {
            Ok(LispyType::create_bool(args[0].is_number()))
        }),
    );
    env.set(
        "string?",
        LispyType::create_function(Some(1), |args| {
            Ok(LispyType::create_bool(args[0].is_string()))
        }),
    );
    env.set(
        "string?",
        LispyType::create_function(Some(1), |args| {
            Ok(LispyType::create_bool(args[0].is_string()))
        }),
    );
    env.set(
        "list?",
        LispyType::create_function(Some(1), |args| {
            Ok(LispyType::create_bool(args[0].is_bool()))
        }),
    );
    env.set(
        "hash?",
        LispyType::create_function(Some(1), |args| {
            Ok(LispyType::create_bool(args[0].is_hash()))
        }),
    );
    env.set(
        "function?",
        LispyType::create_function(Some(1), |args| {
            Ok(LispyType::create_bool(args[0].is_function()))
        }),
    );
    env.set(
        "macro?",
        LispyType::create_function(Some(1), |args| {
            Ok(LispyType::create_bool(args[0].is_macro()))
        }),
    );
    //#endregion
    //#region Compare
    env.set(
        "=",
        LispyType::create_function(Some(2), |args| {
            Ok(LispyType::create_bool(args[0] == args[1]))
        }),
    );
    env.set(
        ">",
        LispyType::create_function(Some(2), |args| {
            Ok(LispyType::create_bool(args[0] > args[1]))
        }),
    );
    env.set(
        "<",
        LispyType::create_function(Some(2), |args| {
            Ok(LispyType::create_bool(args[0] < args[1]))
        }),
    );
    env.set(
        ">=",
        LispyType::create_function(Some(2), |args| {
            Ok(LispyType::create_bool(args[0] >= args[1]))
        }),
    );
    env.set(
        "<",
        LispyType::create_function(Some(2), |args| {
            Ok(LispyType::create_bool(args[0] <= args[1]))
        }),
    );
    //#endregion
    //#region Eval
    env.set(
        "compile-string",
        LispyType::create_function(Some(1), |args| {
            let ast = compile_source_code_to_ast(args[0].clone().as_string().unwrap().as_str());
            let start = vec![LispyType::Symbol {
                value: "do".to_string(),
                meta: HashMap::new(),
            }];

            let collection = vec![start, ast].concat();
            Ok(LispyType::List {
                collection: Box::from(collection),
                meta: HashMap::new(),
            })
        }),
    );
    //#endregion
    //#region FS
    env.set(
        "slurp",
        LispyType::create_function(Some(1), |args| {
            let path = args[0].as_string().unwrap();
            let contents = fs::read_to_string(path);
            if contents.is_err() {
                return Err(LispyType::create_error(
                    format!("File {} not found", path).as_str(),
                    "SYSTEM_ERROR",
                ));
            }
            Ok(LispyType::create_string(contents.unwrap().as_str()))
        }),
    )
    //#endregion
}
