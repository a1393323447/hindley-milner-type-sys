use crate::models::*;
use crate::utils::*;

pub fn alg_w(ctx: Context, expr: Expr) -> anyhow::Result<(Substitution, MonoType)> {
    match expr {
        Expr::Lit(literal) => Ok((Substitution::empty(), literal.ty())),
        Expr::Var(var) => match ctx.constrains.get(&var) {
            Some(poly) => Ok((Substitution::empty(), poly.instantiate())),
            None => anyhow::bail!("Undefiend variable: {var}"),
        },
        Expr::Abs(abs) => {
            let beta = new_type_var();
            let new_ctx = ctx.extend_with(abs.arg, PolyType::Mono(beta.clone()));
            let (s1, t1) = alg_w(new_ctx, abs.body)?;
            let infer_type = MonoType::TyApp(TypeApp {
                func: "->".to_string(),
                monotypes: vec![beta, t1],
            })
            .apply(&s1);

            Ok((s1, infer_type))
        }
        Expr::App(app) => {
            let (s1, t1) = alg_w(ctx.clone(), app.fun)?;
            let (s2, t2) = alg_w(ctx.apply(&s1), app.arg)?;
            let beta = new_type_var();
            let s3 = unify(
                t1.apply(&s2),
                MonoType::TyApp(TypeApp {
                    func: "->".to_string(),
                    monotypes: vec![t2, beta.clone()],
                }),
            )?;
            let infer_type = beta.apply(&s3);

            Ok((s3.combine(s2.combine(s1)), infer_type))
        }
        Expr::Let(let_expr) => {
            let (s1, t1) = alg_w(ctx.clone(), let_expr.vexpr)?;
            let new_ty = t1.generalise(&ctx);
            let new_ctx = ctx.apply(&s1).extend_with(let_expr.var, new_ty);
            let (s2, t2) = alg_w(new_ctx, let_expr.aexpr)?;

            Ok((s2.combine(s1), t2))
        }
    }
}
