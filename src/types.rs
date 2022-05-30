use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Add, Div, Mul, Sub};
use crate::env::LispyEnv;
use crate::machine::eval;

fn integer_decode(val: f64) -> (u64, i16, i8) {
    let bits: u64 = unsafe { mem::transmute(val) };
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };

    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

pub type TypeMeta = HashMap<String, LispyType>;

#[derive(Debug, Clone)]
pub enum LispyType {
    Nil { meta: TypeMeta },

    Bool { value: bool, meta: TypeMeta },
    Number { value: f64, meta: TypeMeta },
    Symbol { value: String, meta: TypeMeta },
    Keyword { value: String, meta: TypeMeta },
    String { value: String, meta: TypeMeta },

    List { collection: Box<Vec<LispyType>>, meta: TypeMeta },
    Hash { collection: Box<HashMap<LispyType, LispyType>>, meta: TypeMeta },

    Error { error_type: String, message: String, meta: TypeMeta },

    Function { arity: Option<i32>, func: fn(args: Vec<LispyType>) -> Result<LispyType, LispyType>, meta: TypeMeta },
    Lambda { bindings: Box<Vec<LispyType>>, to_eval: Box<LispyType>, env: Box<LispyEnv>, meta: TypeMeta },
}

pub struct LispyErrorInternal {
    pub message: String,
    pub error_type: String,
}

// is_? impls
impl LispyType {
    pub fn is_nil(&self) -> bool {
        match self {
            LispyType::Nil { .. } => true,
            _ => false
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            LispyType::Bool { .. } => true,
            _ => false
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            LispyType::Number { .. } => true,
            _ => false
        }
    }

    pub fn is_symbol(&self) -> bool {
        match self {
            LispyType::Symbol { .. } => true,
            _ => false
        }
    }

    pub fn is_keyword(&self) -> bool {
        match self {
            LispyType::Keyword { .. } => true,
            _ => false
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            LispyType::String { .. } => true,
            _ => false
        }
    }

    pub fn is_list(&self) -> bool {
        match self {
            LispyType::List { .. } => true,
            _ => false
        }
    }

    pub fn is_hash(&self) -> bool {
        match self {
            LispyType::Hash { .. } => true,
            _ => false
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            LispyType::Error { .. } => true,
            _ => false
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            LispyType::Function { .. } => true,
            LispyType::Lambda { .. } => true,
            _ => false
        }
    }

    pub fn is_lambda(&self) -> bool {
        match self {
            LispyType::Lambda { .. } => true,
            _ => false
        }
    }
}

// as_? impls
impl LispyType {
    pub fn as_bool(&self) -> Option<&bool> {
        match self {
            LispyType::Bool { value, .. } => Some(value),
            _ => None
        }
    }

    pub fn as_number(&self) -> Option<&f64> {
        match self {
            LispyType::Number { value, .. } => Some(value),
            _ => None
        }
    }

    pub fn as_symbol(&self) -> Option<&String> {
        match self {
            LispyType::Symbol { value, .. } => Some(value),
            _ => None
        }
    }

    pub fn as_keyword(&self) -> Option<&String> {
        match self {
            LispyType::Keyword { value, .. } => Some(value),
            _ => None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            LispyType::String { value, .. } => Some(value),
            _ => None
        }
    }

    pub fn as_list(&self) -> Option<&Box<Vec<LispyType>>> {
        match self {
            LispyType::List { collection, .. } => Some(collection),
            _ => None
        }
    }

    pub fn as_hash(&self) -> Option<&Box<HashMap<LispyType, LispyType>>> {
        match self {
            LispyType::Hash { collection, .. } => Some(collection),
            _ => None
        }
    }

    pub fn as_error(&self) -> Option<LispyErrorInternal> {
        match self {
            LispyType::Error { message, error_type, .. } => Some(LispyErrorInternal { message: message.clone(), error_type: error_type.clone() }),
            _ => None
        }
    }
}

// functions impls
impl LispyType {
    pub fn apply_function(&self, args: Vec<LispyType>) -> Result<LispyType, LispyType> {
        match self {
            LispyType::Function { func, arity, .. } => {
                if arity.is_some() && arity.unwrap() as usize != args.len() {
                    return Err(LispyType::Error {
                        message: format!("Expected arity {}, received {}", arity.unwrap(), args.len()),
                        error_type: "INCORRECT_ARITY".to_string(),
                        meta: HashMap::new(),
                    });
                }
                (func)(args)
            }
            _ => Err(LispyType::Error {
                message: format!("{:?} is not a function", self).to_string(),
                error_type: "NOT_A_FUNCTION".to_string(),
                meta: HashMap::new(),
            })
        }
    }

    pub fn apply_lambda(&self, args: Vec<LispyType>) -> Result<(LispyType, LispyEnv), LispyType> {
        match self {
            LispyType::Lambda { env, to_eval, bindings, .. } => {
                let mut n_env = LispyEnv::child_lambda(env.clone());
                if bindings.len() != args.len() {
                    return Err(LispyType::Error {
                        message: format!("Expected arity {}, received {}", bindings.len(), args.len()),
                        error_type: "INCORRECT_ARITY".to_string(),
                        meta: HashMap::new(),
                    });
                }

                for i in 0..bindings.len() {
                    let key = bindings.get(i).unwrap().clone();
                    let value = args.get(i).unwrap().clone();
                    if !key.is_symbol() {
                        return Err(LispyType::Error {
                            message: format!("Bindings should consist of symbols. Received {}", key),
                            error_type: "INCORRECT_TYPE".to_string(),
                            meta: HashMap::new(),
                        });
                    }
                    n_env.set_item(key.as_symbol().unwrap().clone(), value);
                }

                Ok((*to_eval.clone(), n_env))
            }
            _ => Err(LispyType::Error {
                message: format!("{:?} is not a function", self).to_string(),
                error_type: "NOT_A_FUNCTION".to_string(),
                meta: HashMap::new(),
            })
        }
    }
}


// truthiness
impl LispyType {
    pub fn is_truthy(&self) -> bool {
        match self {
            LispyType::Nil { .. } => false,
            LispyType::Bool { value, .. } => *value,
            LispyType::Number { value, .. } => *value != 0.0,
            _ => true,
        }
    }
}

// Countable
impl LispyType {
    pub fn len(&self) -> LispyType {
        match self {
            LispyType::String { value, .. } => {
                let value = value.len() as f64;
                LispyType::Number { value, meta: HashMap::new() }
            }
            LispyType::List { collection, .. } => {
                let value = collection.len() as f64;
                LispyType::Number { value, meta: HashMap::new() }
            }
            LispyType::Hash { collection, .. } => {
                let value = collection.len() as f64;
                LispyType::Number { value, meta: HashMap::new() }
            }
            _ => LispyType::Error {
                message: format!("{} does not have length", self),
                error_type: "INCORRECT_TYPE".to_string(),
                meta: HashMap::new(),
            }
        }
    }
}

// constructors
impl LispyType {
    pub fn create_function(arity: Option<i32>, func: fn(args: Vec<LispyType>) -> Result<LispyType, LispyType>) -> Self {
        Self::Function {
            arity,
            func,
            meta: HashMap::new(),
        }
    }

    pub fn create_bool(value: bool) -> Self {
        Self::Bool {
            value,
            meta: HashMap::new(),
        }
    }

    pub fn create_number(value: f64) -> Self {
        Self::Number {
            value,
            meta: HashMap::new(),
        }
    }

    pub fn create_nil() -> Self {
        Self::Nil {
            meta: HashMap::new()
        }
    }

    pub fn create_error(message: &str, err_type: &str) -> Self {
        Self::Error {
            message: message.to_string(),
            error_type: err_type.to_string(),
            meta: HashMap::new(),
        }
    }

    pub fn create_string(value: &str) -> Self {
        Self::String {
            value: value.to_string(),
            meta: HashMap::new(),
        }
    }
}

impl Display for LispyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LispyType::Nil { .. } => {
                write!(f, "nil")
            }
            LispyType::Bool { value, .. } => {
                write!(f, "{}", value)
            }
            LispyType::Number { value, .. } => {
                write!(f, "{}", value)
            }
            LispyType::Symbol { value, .. } => {
                write!(f, "Symbol<{}>", value)
            }
            LispyType::Keyword { value, .. } => {
                write!(f, "Keyword<{}>", value)
            }
            LispyType::String { value, .. } => {
                write!(f, "{}", value)
            }
            LispyType::List { collection, .. } => {
                let mut str = "".to_string();
                // collection.iter().for_each(|item| { str += &format!("{}", item).to_string() });
                str += &format!("{}", collection.first().unwrap()).to_string();
                for index in 1..collection.len() {
                    str += &format!(", {}", collection.get(index).unwrap());
                }
                write!(f, "[{}]", str)
            }
            LispyType::Hash { collection, .. } => {
                let mut str = "".to_string();

                collection.iter().for_each(|(key, value)| {
                    str += &format!("{} -> {}, ", key, value);
                });

                str = str.to_string()[0..str.len() - 2].to_string();

                write!(f, "{{{}}}", str)
            }
            LispyType::Error { message, .. } => {
                write!(f, "{}", message)
            }
            LispyType::Function { .. } => {
                write!(f, "#<function>")
            }
            LispyType::Lambda { .. } => {
                write!(f, "#<lambda-function>")
            }
        }
    }
}

impl PartialEq for LispyType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            LispyType::Nil { .. } => other.is_nil(),
            LispyType::Bool { .. } => {
                other.is_bool() && self.as_bool() == other.as_bool()
            }
            LispyType::Number { .. } => {
                other.is_number() && self.as_number() == other.as_number()
            }
            LispyType::Symbol { .. } => {
                other.is_symbol() && self.as_symbol() == other.as_symbol()
            }
            LispyType::Keyword { .. } => {
                other.is_keyword() && self.as_keyword() == other.as_keyword()
            }
            LispyType::String { .. } => {
                other.is_string() && self.as_string() == other.as_string()
            }
            LispyType::List { .. } => {
                if !other.is_list() { return false; }
                if other.as_list().unwrap().len() != self.as_list().unwrap().len() { return false; }

                for index in 0..self.as_list().unwrap().len() - 1 {
                    if self.as_list().unwrap().get(index) != other.as_list().unwrap().get(index) {
                        return false;
                    }
                }

                true
            }
            LispyType::Hash { .. } => {
                if !other.is_hash() { return false; }
                if other.as_hash().unwrap().len() != self.as_hash().unwrap().len() { return false; }

                for key in self.as_hash().as_ref().unwrap().keys() {
                    if self.as_hash().as_ref().unwrap().get(key) != other.as_hash().as_ref().unwrap().get(key) {
                        return false;
                    }
                }

                true
            }
            LispyType::Error { .. } => {
                other.is_error() && self.as_error().unwrap().error_type == other.as_error().unwrap().error_type
            }
            LispyType::Function { .. } => false,
            LispyType::Lambda { .. } => false,
        }
    }
}

impl Eq for LispyType {}

impl Hash for LispyType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            LispyType::Nil { .. } => false.hash(state),
            LispyType::Bool { value, .. } => value.hash(state),
            LispyType::Number { value, .. } => integer_decode(value.clone()).hash(state),
            LispyType::Symbol { value, .. } => value.hash(state),
            LispyType::Keyword { value, .. } => value.hash(state),
            LispyType::String { value, .. } => value.hash(state),
            _ => panic!("Could not use this as a hash key"),
        }
    }
}

impl Add for LispyType {
    type Output = LispyType;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            LispyType::Number { value: a, .. } => {
                if rhs.is_number() {
                    let res = a + rhs.as_number().unwrap();
                    return LispyType::Number { value: res, meta: HashMap::new() };
                }
                LispyType::Error { message: "+ only works with number vars".to_string(), error_type: "INCORRECT_TYPE".to_string(), meta: HashMap::new() }
            }
            _ => LispyType::Error { message: "+ only works with number vars".to_string(), error_type: "INCORRECT_TYPE".to_string(), meta: HashMap::new() }
        }
    }
}

impl Sub for LispyType {
    type Output = LispyType;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            LispyType::Number { value: a, .. } => {
                if rhs.is_number() {
                    let res = a - rhs.as_number().unwrap();
                    return LispyType::Number { value: res, meta: HashMap::new() };
                }
                LispyType::Error { message: "+ only works with number vars".to_string(), error_type: "INCORRECT_TYPE".to_string(), meta: HashMap::new() }
            }
            _ => LispyType::Error { message: "+ only works with number vars".to_string(), error_type: "INCORRECT_TYPE".to_string(), meta: HashMap::new() }
        }
    }
}

impl Mul for LispyType {
    type Output = LispyType;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            LispyType::Number { value: a, .. } => {
                if rhs.is_number() {
                    let res = a * rhs.as_number().unwrap();
                    return LispyType::Number { value: res, meta: HashMap::new() };
                }
                LispyType::Error { message: "+ only works with number vars".to_string(), error_type: "INCORRECT_TYPE".to_string(), meta: HashMap::new() }
            }
            _ => LispyType::Error { message: "+ only works with number vars".to_string(), error_type: "INCORRECT_TYPE".to_string(), meta: HashMap::new() }
        }
    }
}

impl Div for LispyType {
    type Output = LispyType;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            LispyType::Number { value: a, .. } => {
                if rhs.is_number() {
                    let res = a / rhs.as_number().unwrap();
                    return LispyType::Number { value: res, meta: HashMap::new() };
                }
                LispyType::Error { message: "+ only works with number vars".to_string(), error_type: "INCORRECT_TYPE".to_string(), meta: HashMap::new() }
            }
            _ => LispyType::Error { message: "+ only works with number vars".to_string(), error_type: "INCORRECT_TYPE".to_string(), meta: HashMap::new() }
        }
    }
}

impl PartialOrd for LispyType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            LispyType::Number { .. } => {
                self.as_number().unwrap().partial_cmp(other.as_number().unwrap())
            }
            _ => None,
        }
    }
}