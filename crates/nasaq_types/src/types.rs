#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Int,
    Float,
    Bool,
    String,
    Char,
    Void,
    Unknown,
    Named(String),
    Option(Box<Ty>),
    Result {
        ok: Box<Ty>,
        err: Box<Ty>,
    },
    Function {
        params: Vec<Ty>,
        ret: Box<Ty>,
    },
    Struct {
        name: String,
        fields: Vec<(String, Ty)>,
    },
}

impl Ty {
    pub fn from_ast(ty: &nasaq_ast::Type) -> Self {
        match ty {
            nasaq_ast::Type::Int => Ty::Int,
            nasaq_ast::Type::Float => Ty::Float,
            nasaq_ast::Type::Bool => Ty::Bool,
            nasaq_ast::Type::String => Ty::String,
            nasaq_ast::Type::Char => Ty::Char,
            nasaq_ast::Type::Void => Ty::Void,
            nasaq_ast::Type::Named(name) => Ty::Named(name.node.clone()),
            nasaq_ast::Type::Function { params, return_type } => Ty::Function {
                params: params.iter().map(|p| Ty::from_ast(&p.node)).collect(),
                ret: Box::new(Ty::from_ast(&return_type.node)),
            },
            nasaq_ast::Type::Tuple(_) | nasaq_ast::Type::Array { .. } => Ty::Unknown,
            nasaq_ast::Type::Generic { base, args } => {
                if let nasaq_ast::Type::Named(name) = &base.node {
                    match name.node.as_str() {
                        "Option" if args.len() == 1 => {
                            return Ty::Option(Box::new(Ty::from_ast(&args[0].node)));
                        }
                        "Result" if args.len() == 2 => {
                            return Ty::Result {
                                ok: Box::new(Ty::from_ast(&args[0].node)),
                                err: Box::new(Ty::from_ast(&args[1].node)),
                            };
                        }
                        _ => {}
                    }
                }
                Ty::from_ast(&base.node)
            }
            nasaq_ast::Type::Option(inner) => Ty::Option(Box::new(Ty::from_ast(&inner.node))),
            nasaq_ast::Type::Result { ok, err } => Ty::Result {
                ok: Box::new(Ty::from_ast(&ok.node)),
                err: Box::new(Ty::from_ast(&err.node)),
            },
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::Int | Ty::Float)
    }
}
