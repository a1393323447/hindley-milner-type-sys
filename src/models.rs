use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::utils::new_type_var;

pub trait GetFreeVars {
    fn free_vars(&self) -> HashSet<&str>;
}

#[derive(Debug, Clone, Copy)]
pub enum Literal {
    Int,
    Bool,
}

impl Literal {
    pub fn ty(&self) -> MonoType {
        let type_name = match self {
            Literal::Int => "Int",
            Literal::Bool => "Bool",
        }
        .to_string();

        MonoType::TyApp(TypeApp {
            func: type_name,
            monotypes: vec![],
        })
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int => write!(f, "int"),
            Literal::Bool => write!(f, "bool"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppExpr {
    pub fun: Expr,
    pub arg: Expr,
}

impl Display for AppExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.fun, self.arg)
    }
}

#[derive(Debug, Clone)]
pub struct AbsExpr {
    pub arg: String,
    pub body: Expr,
}

impl Display for AbsExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(\\{} -> {})", self.arg, self.body)
    }
}

#[derive(Debug, Clone)]
pub struct LetExpr {
    pub var: String,
    pub vexpr: Expr,
    pub aexpr: Expr,
}

impl Display for LetExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(let {} = {} in {})", self.var, self.vexpr, self.aexpr)
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Literal),
    Var(String),
    App(Box<AppExpr>),
    Abs(Box<AbsExpr>),
    Let(Box<LetExpr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lit(lit) => write!(f, "{lit}"),
            Expr::Var(var) => write!(f, "{var}"),
            Expr::App(app) => write!(f, "{app}"),
            Expr::Abs(abs) => write!(f, "{abs}"),
            Expr::Let(elet) => write!(f, "{elet}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeApp {
    pub func: String,
    pub monotypes: Vec<MonoType>,
}

impl Display for TypeApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.func.as_str() {
            "->" => {
                let len = self.monotypes.len();
                assert!(len == 2, "expected 2 args for `->` but got {len}");
                write!(f, "{} -> {}", self.monotypes[0], self.monotypes[1])
            }
            _ => {
                write!(f, "{}", self.func)?;
                for mono in self.monotypes.iter() {
                    write!(f, " {}", mono)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum MonoType {
    TyVar(String),
    TyApp(TypeApp),
}

impl Display for MonoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonoType::TyVar(var) => write!(f, "{var}"),
            MonoType::TyApp(app) => write!(f, "{app}"),
        }
    }
}

impl MonoType {
    pub fn generalise(&self, ctx: &Context) -> PolyType {
        let ty_free_vars = self.free_vars();
        let ctx_free_vars = ctx.free_vars();
        let quantifiers = ty_free_vars.difference(&ctx_free_vars);

        quantifiers.fold(PolyType::Mono(self.clone()), |acc, q| {
            PolyType::TyQuantifier(TypeQuantifier {
                bounded: q.to_string(),
                ty: Box::new(acc),
            })
        })
    }

    fn instantiate_impl(&self, mapping: &mut HashMap<&str, MonoType>) -> MonoType {
        match self {
            MonoType::TyVar(name) => mapping
                .get(name.as_str())
                .cloned()
                .unwrap_or(MonoType::TyVar(name.clone())),
            MonoType::TyApp(app) => {
                let monotypes: Vec<_> = app
                    .monotypes
                    .iter()
                    .map(|m| m.instantiate_impl(mapping))
                    .collect();

                MonoType::TyApp(TypeApp {
                    func: app.func.clone(),
                    monotypes,
                })
            }
        }
    }
}

impl GetFreeVars for MonoType {
    fn free_vars(&self) -> HashSet<&str> {
        match self {
            MonoType::TyVar(name) => {
                let mut set = HashSet::new();
                set.insert(name.as_str());
                set
            }
            MonoType::TyApp(app) => app.monotypes.iter().flat_map(|m| m.free_vars()).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeQuantifier {
    pub bounded: String,
    pub ty: Box<PolyType>,
}

impl Display for TypeQuantifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "∀{}.{} -> {}", self.bounded, self.bounded, self.ty)
    }
}

#[derive(Debug, Clone)]
pub enum PolyType {
    Mono(MonoType),
    TyQuantifier(TypeQuantifier),
}

impl Display for PolyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolyType::Mono(mono) => write!(f, "{mono}"),
            PolyType::TyQuantifier(quan) => write!(f, "{quan}"),
        }
    }
}

impl PolyType {
    pub fn instantiate(&self) -> MonoType {
        self.instantiate_impl(&mut HashMap::new())
    }

    fn instantiate_impl<'a>(&'a self, mapping: &mut HashMap<&'a str, MonoType>) -> MonoType {
        match self {
            PolyType::Mono(mono) => mono.instantiate_impl(mapping),
            PolyType::TyQuantifier(quan) => {
                mapping.insert(&quan.bounded, new_type_var());
                quan.ty.instantiate_impl(mapping)
            }
        }
    }
}

impl GetFreeVars for PolyType {
    fn free_vars(&self) -> HashSet<&str> {
        match self {
            PolyType::Mono(mono) => mono.free_vars(),
            PolyType::TyQuantifier(quan) => quan
                .ty
                .free_vars()
                .into_iter()
                .filter(|&v| v != quan.bounded)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub constrains: HashMap<String, PolyType>,
}

impl Context {
    pub fn extend_with(&self, k: String, v: PolyType) -> Context {
        let mut new_ctx = self.clone();
        new_ctx.constrains.insert(k, v);

        new_ctx
    }
}

impl GetFreeVars for Context {
    fn free_vars(&self) -> HashSet<&str> {
        self.constrains
            .values()
            .flat_map(|p| p.free_vars())
            .collect()
    }
}
