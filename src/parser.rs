use std::iter::Peekable;

use crate::lex::*;
use crate::models::*;

use anyhow::Context;

macro_rules! next {
    ($l: expr) => {
        match $l.next() {
            Some(t) => t,
            None => anyhow::bail!("Unexpected EOF"),
        }
    };
}

macro_rules! expected_next {
    ($l: expr, $k: ident, $loc: expr) => {
        match $l.next() {
            Some(t) => t,
            None => anyhow::bail!(format!("Expected {:?} near {}", TokenKind::$k, $loc)),
        }
    };
}

macro_rules! expected {
    ($t: ident, $k: ident) => {
        if $t.kind != TokenKind::$k {
            anyhow::bail!(format!(
                "expected {:?} but got {:?} in {}",
                TokenKind::$k,
                $t.kind,
                $t.loc
            ))
        }
    };
}

pub struct Parser<'c> {
    lexer: Peekable<Lexer<'c>>,
}

impl<'c> Parser<'c> {
    pub fn new(lexer: Lexer<'c>) -> Self {
        Self {
            lexer: lexer.peekable(),
        }
    }

    pub fn parse(&mut self) -> anyhow::Result<Expr> {
        let token = next!(self.lexer);
        match token.kind {
            TokenKind::LitBool => Ok(Expr::Lit(Literal::Bool)),
            TokenKind::LitInt => Ok(Expr::Lit(Literal::Int)),
            TokenKind::Var => Ok(Expr::Var(token.value)),
            TokenKind::OpenP => self.parse_rest_expr(),
            _ => {
                anyhow::bail!("Unexpected tokenkind {:?} in {}", token.kind, token.loc)
            }
        }
    }

    fn parse_rest_expr(&mut self) -> anyhow::Result<Expr> {
        let token = match self.lexer.peek() {
            Some(t) => t,
            None => anyhow::bail!("Unexpected EOF"),
        };
        let token_loc = token.loc;

        let res = match token.kind {
            TokenKind::Let => self.parse_let(),
            TokenKind::Var => self.parse_app(),
            TokenKind::BackSlash => self.parse_abs(),
            TokenKind::OpenP => {
                let fun = self.parse().context("expected function expr")?;
                self.parse_app_with_fun(fun)
            }
            _ => anyhow::bail!("Unexpected tokenkind {:?} in {}", token.kind, token.loc),
        };

        let closp = expected_next!(self.lexer, ClosP, token_loc);
        expected!(closp, ClosP);

        res
    }

    fn parse_let(&mut self) -> anyhow::Result<Expr> {
        let token_let = next!(self.lexer);
        let var = expected_next!(self.lexer, Var, token_let.loc);
        expected!(var, Var);

        let token_eq = expected_next!(self.lexer, Eq, var.loc);
        expected!(token_eq, Eq);

        let vexpr = self.parse().context("expected vexpr")?;

        let token_in = expected_next!(self.lexer, In, token_eq.loc);
        expected!(token_in, In);

        let aexpr = self.parse().context("expected aexpr")?;

        Ok(Expr::Let(Box::new(LetExpr {
            var: var.value,
            vexpr,
            aexpr,
        })))
    }

    fn parse_abs(&mut self) -> anyhow::Result<Expr> {
        let token_backslash = next!(self.lexer);
        let arg = expected_next!(self.lexer, Var, token_backslash.loc);
        expected!(arg, Var);

        let arrow = expected_next!(self.lexer, Arrow, arg.loc);
        expected!(arrow, Arrow);

        let body = self.parse().context("expected function body")?;

        Ok(Expr::Abs(Box::new(AbsExpr {
            arg: arg.value,
            body,
        })))
    }

    fn parse_app_with_fun(&mut self, fun: Expr) -> anyhow::Result<Expr> {
        let arg = self.parse().context("expected function arg")?;

        Ok(Expr::App(Box::new(AppExpr { fun, arg })))
    }

    fn parse_app(&mut self) -> anyhow::Result<Expr> {
        let fun = self.parse().context("expected function expr")?;
        self.parse_app_with_fun(fun)
    }
}
