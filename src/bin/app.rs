use std::collections::HashMap;
use std::io::Write;

use hm_type_sys::alg_w::alg_w;
use hm_type_sys::lex::*;
use hm_type_sys::models::*;
use hm_type_sys::parser::*;
use hm_type_sys::utils::reset_tid;

macro_rules! s {
    ($($t: tt)+) => {
        stringify!($($t)+).to_string()
    };
}

macro_rules! tyvar {
    ($n: ident) => {
        MonoType::TyVar(s!($n))
    };
}

macro_rules! tyapp {
    ($func: expr, $monos: expr) => {
        MonoType::TyApp({
            TypeApp {
                func: $func,
                monotypes: $monos,
            }
        })
    };
}

macro_rules! tyname {
    ($name: ident) => {
        tyapp!(s!($name), vec![])
    };
}

macro_rules! to_poly {
    ($mono: expr) => {
        PolyType::Mono($mono)
    };
}

macro_rules! quan {
    ($bound: expr, $ty: expr) => {
        PolyType::TyQuantifier(TypeQuantifier {
            bounded: $bound,
            ty: Box::new($ty),
        })
    };
}

fn default_ctx() -> Context {
    let mut constrains = HashMap::new();

    // list: ∀ a. a -> List a
    constrains.insert(
        s!(list),
        quan!(
            s!(a),
            to_poly!(tyapp!(
                s!(->),
                vec![tyvar!(a), tyapp!(s!(List), vec![tyvar!(a)])]
            ))
        ),
    );
    // inc: Int -> Int
    constrains.insert(
        s!(inc),
        to_poly!(tyapp!(s!(->), vec![tyname!(Int), tyname!(Int)])),
    );
    // dec: Int -> Int
    constrains.insert(
        s!(dec),
        to_poly!(tyapp!(s!(->), vec![tyname!(Int), tyname!(Int)])),
    );
    // is_null: ∀ a. a -> Bool
    constrains.insert(
        s!(isNull),
        quan!(
            s!(a),
            to_poly!(tyapp!(s!(->), vec![tyvar!(a), tyname!(Bool)]))
        ),
    );
    // add: Int -> Int -> Int
    constrains.insert(
        s!(add),
        to_poly!(tyapp!(
            s!(->),
            vec![
                tyname!(Int),
                tyapp!(s!(->), vec![tyname!(Int), tyname!(Int)])
            ]
        )),
    );

    Context { constrains }
}

fn print_syntax() {
    println!("syntax: e ::= x | (e1 e2) | (\\x -> e) | (let e1 in e2)\n");
}

fn print_default_ctx() {
    println!("Run in Context: ");
    let ctx = default_ctx();
    for (n, c) in ctx.constrains {
        println!("{n}: {c}");
    }
    println!();
}

fn main() -> anyhow::Result<()> {
    print_syntax();
    print_default_ctx();

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    stdout.write_all(b"> ")?;
    stdout.flush()?;

    for line in stdin.lines() {
        let source = line?;

        let expr = match Parser::new(Lexer::new(source.as_str())).parse() {
            Ok(e) => e,
            Err(err) => {
                stdout.write_fmt(format_args!("Syntax Error: {err}\n"))?;
                stdout.write_all(b"> ")?;
                stdout.flush()?;
                continue;
            }
        };

        let ctx = default_ctx();
        let ty = match alg_w(ctx, expr) {
            Ok((_, ty)) => ty,
            Err(err) => {
                stdout.write_fmt(format_args!("Type Error: {err}\n"))?;
                stdout.write_all(b"> ")?;
                stdout.flush()?;
                continue;
            }
        };
        reset_tid();

        stdout.write_fmt(format_args!("`{}` infer as `{}`\n", source, ty))?;

        stdout.write_all(b"> ")?;
        stdout.flush()?;
    }

    Ok(())
}
