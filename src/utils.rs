use crate::models::*;
use std::collections::HashMap;

pub trait Subst: Clone + Sized {
    fn apply(self, s: &Substitution) -> Self;
}

#[derive(Debug, Clone)]
pub struct Substitution {
    map: HashMap<String, MonoType>,
}

impl Substitution {
    pub fn empty() -> Substitution {
        Self {
            map: HashMap::default(),
        }
    }

    pub fn from_map(map: HashMap<String, MonoType>) -> Substitution {
        Substitution { map }
    }

    pub fn combine(self, mut other: Substitution) -> Substitution {
        let map: Vec<_> = self
            .map
            .into_iter()
            .map(|(k, v)| (k, v.apply(&other)))
            .collect();
        other.map.extend(map.into_iter());

        other
    }
}

impl Subst for MonoType {
    fn apply(self, s: &Substitution) -> Self {
        match self {
            MonoType::TyVar(name) => s.map.get(&name).cloned().unwrap_or(MonoType::TyVar(name)),
            MonoType::TyApp(app) => {
                let monotypes = app.monotypes.into_iter().map(|ty| ty.apply(s)).collect();
                MonoType::TyApp(TypeApp { monotypes, ..app })
            }
        }
    }
}

impl Subst for PolyType {
    fn apply(self, s: &Substitution) -> Self {
        match self {
            PolyType::Mono(mono) => PolyType::Mono(mono.apply(s)),
            PolyType::TyQuantifier(quan) => {
                let ty = Box::new(quan.ty.apply(s));
                PolyType::TyQuantifier(TypeQuantifier { ty, ..quan })
            }
        }
    }
}

impl Subst for Context {
    fn apply(self, s: &Substitution) -> Self {
        let constrains = self
            .constrains
            .into_iter()
            .map(|(k, v)| (k, v.apply(s)))
            .collect();

        Context { constrains }
    }
}

static mut TID: usize = 0;
pub fn new_type_var() -> MonoType {
    unsafe {
        let id = TID;
        TID += 1;

        MonoType::TyVar(format!("t{id}"))
    }
}

pub fn reset_tid() {
    unsafe {
        TID = 0;
    }
}

pub fn unify(ty1: MonoType, ty2: MonoType) -> anyhow::Result<Substitution> {
    match (ty1, ty2) {
        (MonoType::TyVar(var1), MonoType::TyVar(var2)) if var1 == var2 => Ok(Substitution::empty()),
        (MonoType::TyVar(var), ty) => {
            if contains(&ty, &var) {
                anyhow::bail!("Infinite type detected");
            } else {
                let mut map = HashMap::new();
                map.insert(var, ty);
                Ok(Substitution::from_map(map))
            }
        }
        (ty, MonoType::TyVar(var)) => unify(MonoType::TyVar(var), ty),
        (MonoType::TyApp(app1), MonoType::TyApp(app2)) => {
            if app1.func != app2.func {
                anyhow::bail!(
                    "Could not unify types (different type functions): {} and {}",
                    app1.func,
                    app2.func
                );
            }
            if app1.monotypes.len() != app2.monotypes.len() {
                anyhow::bail!(
                    "Could not unify types (different argument lengths): {app1} and {app2}"
                );
            }
            let mut subst = Substitution::empty();
            let mono_tuples = app1.monotypes.into_iter().zip(app2.monotypes.into_iter());
            for (mono1, mono2) in mono_tuples {
                let unit_subst = unify(mono1.apply(&subst), mono2.apply(&subst))?;
                subst = subst.combine(unit_subst);
            }

            Ok(subst)
        }
    }
}

fn contains(ty1: &MonoType, var: &str) -> bool {
    match ty1 {
        MonoType::TyVar(tvar) => tvar == var,
        MonoType::TyApp(app) => app.monotypes.iter().any(|mono| contains(mono, var)),
    }
}
