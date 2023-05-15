pub mod alg_w;
pub mod lex;
pub mod models;
pub mod parser;
pub mod utils;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::alg_w::alg_w;
    use crate::lex::*;
    use crate::models::*;
    use crate::parser::*;

    macro_rules! infer {
        ($source: expr) => {
            let expr = Parser::new(Lexer::new($source)).parse().unwrap();
            let ctx = default_ctx();
            let (_, ty) = alg_w(ctx, expr)?;
            println!("`{}` infer as `{}`", $source, ty);
        };
    }

    fn default_ctx() -> Context {
        let constrains = HashMap::new();
        Context { constrains }
    }

    #[test]
    fn test_alg_w() -> anyhow::Result<()> {
        infer!(r"(\x -> x)");
        infer!(r"(\x -> 10)");
        infer!(r"((\x -> x) 10)");
        infer!(r"(let x = 10 in x)");
        infer!(r"(let const = (\y -> true) in const)");
        infer!(r"(let const = (\x -> true) in (\y -> (\z -> (const 20))))");

        Ok(())
    }
}
