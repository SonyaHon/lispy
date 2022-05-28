use std::collections::HashMap;
use crate::env::LispyEnv;
use crate::types::LispyType;

pub fn apply_core_ns(env: &mut LispyEnv) {
    //#region Math
    env.set("+", LispyType::create_function(
        Some(2),
        |args| {
            return Ok(args[0].clone() + args[1].clone());
        },
    ));

    env.set("-", LispyType::create_function(
        Some(2),
        |args| {
            return Ok(args[0].clone() - args[1].clone());
        },
    ));

    env.set("*", LispyType::create_function(
        Some(2),
        |args| {
            return Ok(args[0].clone() * args[1].clone());
        },
    ));

    env.set("/", LispyType::create_function(
        Some(2),
        |args| {
            return Ok(args[0].clone() / args[1].clone());
        },
    ));

    //#endregion
    //#region Utility
    env.set("println", LispyType::create_function(
        None,
        |args| {
            let mut str = "".to_string();
            args.iter().for_each(|item| { str += &format!("{}", item).to_string() });
            println!("{}", str);
            Ok(LispyType::Nil { meta: HashMap::new() })
        },
    ));

    env.set("print", LispyType::create_function(
        None,
        |args| {
            let mut str = "".to_string();
            args.iter().for_each(|item| { str += &format!("{}", item).to_string() });
            print!("{}", str);
            Ok(LispyType::Nil { meta: HashMap::new() })
        },
    ));

    env.set("list", LispyType::create_function(
        None,
        |args| {
            Ok(LispyType::List { collection: Box::from(args.clone()), meta: HashMap::new() })
        },
    ));
    env.set("count", LispyType::create_function(
        Some(1),
        |args| {
            let res = args[0].len();
            if res.is_error() {
                return Err(res.clone());
            }
            Ok(res.clone())
        },
    ));
    //#endregion
    //#region is_?
    env.set("nil?", LispyType::create_function(
        Some(1),
        |args| {
            Ok(LispyType::create_bool(args[0].is_nil()))
        },
    ));
    env.set("bool?", LispyType::create_function(
        Some(1),
        |args| {
            Ok(LispyType::create_bool(args[0].is_bool()))
        },
    ));
    env.set("symbol?", LispyType::create_function(
        Some(1),
        |args| {
            Ok(LispyType::create_bool(args[0].is_symbol()))
        },
    ));
    env.set("number?", LispyType::create_function(
        Some(1),
        |args| {
            Ok(LispyType::create_bool(args[0].is_number()))
        },
    ));
    env.set("string?", LispyType::create_function(
        Some(1),
        |args| {
            Ok(LispyType::create_bool(args[0].is_string()))
        },
    ));
    env.set("string?", LispyType::create_function(
        Some(1),
        |args| {
            Ok(LispyType::create_bool(args[0].is_string()))
        },
    ));
    env.set("list?", LispyType::create_function(
        Some(1),
        |args| {
            Ok(LispyType::create_bool(args[0].is_bool()))
        },
    ));
    env.set("hash?", LispyType::create_function(
        Some(1),
        |args| {
            Ok(LispyType::create_bool(args[0].is_hash()))
        },
    ));
    env.set("function?", LispyType::create_function(
        Some(1),
        |args| {
            Ok(LispyType::create_bool(args[0].is_function()))
        },
    ));
    //#endregion
    //#region Compare
    env.set("=", LispyType::create_function(
        Some(2),
        |args| {
            Ok(
                LispyType::create_bool(
                    args[0] == args[1]
                )
            )
        },
    ));
    env.set(">", LispyType::create_function(
        Some(2),
        |args| {
            Ok(
                LispyType::create_bool(
                    args[0] > args[1]
                )
            )
        },
    ));
    env.set("<", LispyType::create_function(
        Some(2),
        |args| {
            Ok(
                LispyType::create_bool(
                    args[0] < args[1]
                )
            )
        },
    ));
    env.set(">=", LispyType::create_function(
        Some(2),
        |args| {
            Ok(
                LispyType::create_bool(
                    args[0] >= args[1]
                )
            )
        },
    ));
    env.set("<", LispyType::create_function(
        Some(2),
        |args| {
            Ok(
                LispyType::create_bool(
                    args[0] <= args[1]
                )
            )
        },
    ));
    //#endregion
}