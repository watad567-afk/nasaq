use nasaq_syntax::Spanned;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named(Spanned<String>),
    Int,
    Float,
    Bool,
    String,
    Char,
    Void,
    Array {
        element: Box<Spanned<Type>>,
    },
    Tuple(Vec<Spanned<Type>>),
    Function {
        params: Vec<Spanned<Type>>,
        return_type: Box<Spanned<Type>>,
    },
    Generic {
        base: Box<Spanned<Type>>,
        args: Vec<Spanned<Type>>,
    },
    Option(Box<Spanned<Type>>),
    Result {
        ok: Box<Spanned<Type>>,
        err: Box<Spanned<Type>>,
    },
}
