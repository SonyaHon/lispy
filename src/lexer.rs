use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum LexerToken {
    #[token("(")]
    ListStart,

    #[token(")")]
    ListEnd,

    #[token("{")]
    HashStart,

    #[token("}")]
    HashEnd,

    #[regex("true|false", | lex | lex.slice().parse())]
    Boolean(bool),

    #[token("nil")]
    Nil,

    #[token("'")]
    Quote,

    #[token("`")]
    QuasiQuote,

    #[token("~")]
    Unquote,

    #[token("~@")]
    SpliceUnquote,

    #[token("&")]
    ArgsSpread,
    
    #[regex(r#""(\\"|[^"])*""#, | lex | lex.slice().parse())]
    String(String),

    #[regex(r"[-]?((\d+(\.\d*)?)|(\.\d+))", | lex | lex.slice().parse(), priority = 2)]
    Number(f64),

    #[regex(r":(:|\w)[\w\-!@#$+?~]*", | lex | lex.slice().parse())]
    Keyword(String),

    #[regex(r"[\w+\-*/$&#=][\w\-!@#$+?~*=]*", | lex | lex.slice().parse())]
    Symbol(String),

    #[error]
    #[regex(r"[\s,]", logos::skip)]
    #[regex(r";.*\n", logos::skip)]
    Error,
}
