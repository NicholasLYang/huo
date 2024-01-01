use crate::ast::{BinaryOp, Expr, Loc, Program, Stmt};
use chumsky::prelude::*;
use itertools::Itertools;

pub fn parse(code: &str) -> Result<Loc<Program>, anyhow::Error> {
    parser()
        .parse(code)
        .map_err(|e| anyhow::anyhow!("failed to parse: {}", e.into_iter().join("\n")))
}

fn parser() -> impl Parser<char, Loc<Program>, Error = Simple<char>> {
    let uint = text::int(10).map_with_span(Loc);
    let range_tensor = just('[')
        .padded()
        .ignore_then(uint.clone())
        .then_ignore(just("..").padded())
        .then(uint.clone())
        .then(
            just(';')
                .padded()
                .ignore_then(uint.clone())
                .map(Some)
                .or(empty().to(None)),
        )
        .then_ignore(just(']').padded())
        .map(|((start, stop), step)| Expr::RangeTensor {
            start: start.map(|start| start.parse::<i32>().unwrap()),
            stop: stop.map(|start| start.parse::<i32>().unwrap()),
            step: step.map(|s| s.map(|s| s.parse::<i32>().unwrap())),
        });

    let fill_tensor = just('[')
        .padded()
        .ignore_then(uint.clone().separated_by(just('x').padded()))
        .then_ignore(just(';').padded())
        .then(uint.clone())
        .then_ignore(just(']').padded())
        .map(|(shape, fill)| Expr::FillTensor {
            fill: fill.map(|fill| fill.parse().unwrap()),
            shape: shape
                .into_iter()
                .map(|s| s.map(|s| s.parse::<usize>().unwrap()))
                .collect(),
            // TODO: Support other data types
            data_type: None,
        });

    let var = text::ident().map(Expr::Variable);

    let atom = fill_tensor.or(range_tensor).or(var).map_with_span(Loc);

    let expr = atom
        .clone()
        .then(
            just('*')
                .padded()
                .map_with_span(|_, span| Loc(BinaryOp::Mul, span))
                .then(atom)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| {
            let span = (lhs.1.start())..(rhs.1.end());
            Loc(
                Expr::Binary {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op,
                },
                span,
            )
        });

    let expr_stmt = expr.clone().then_ignore(just(';').padded()).map(Stmt::Expr);

    let assign_stmt = text::ident()
        .map_with_span(Loc)
        .then_ignore(just('=').padded())
        .then(expr)
        .then_ignore(just(';').padded())
        .map(|(lhs, rhs)| Stmt::Assign { lhs, rhs });

    let stmt = expr_stmt.or(assign_stmt).map_with_span(Loc);

    stmt.repeated()
        .map_with_span(|stmts, span| Loc(Program { stmts }, span))
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use crate::ast::{Expr, Stmt};
    use crate::parser::parse;
    use candle_core::DType;

    #[test]
    fn test_parse_program() {
        let program = parse("[2 x 3; 0]; [1 x 5; 1]; [0..10]; a = [2..20 ; 2];").unwrap();
        assert_eq!(program.stmts.len(), 4);
        assert_eq!(
            program.stmts[0],
            Stmt::Expr(Expr::FillTensor {
                fill: 0,
                shape: vec![2, 3],
                data_type: Some(DType::U32),
            })
        );
    }
}
