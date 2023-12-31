use crate::ast::{Expr, Program, Stmt};
use candle_core::DType;
use chumsky::prelude::*;
use itertools::Itertools;

fn parse(program: &str) -> Result<Program, anyhow::Error> {
    parser()
        .parse(program)
        .map_err(|e| anyhow::anyhow!("failed to parse: {}", e.into_iter().join("\n")))
}

fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    let uint = text::int(10);
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
            start: start.parse().unwrap(),
            stop: stop.parse().unwrap(),
            step: step.map(|s| s.parse().unwrap()),
        });

    let fill_tensor = just('[')
        .padded()
        .ignore_then(uint.clone().separated_by(just('x').padded()))
        .then_ignore(just(';').padded())
        .then(uint.clone())
        .then_ignore(just(']').padded())
        .map(|(shape, fill)| Expr::FillTensor {
            fill: fill.parse().unwrap(),
            shape: shape.into_iter().map(|s| s.parse().unwrap()).collect(),
            // TODO: Support other data types
            data_type: Some(DType::U32),
        });

    let expr = fill_tensor.or(range_tensor);

    let expr_stmt = expr.clone().then_ignore(just(';').padded()).map(Stmt::Expr);

    let assign_stmt = text::ident()
        .then_ignore(just('=').padded())
        .then(expr)
        .then_ignore(just(';').padded())
        .map(|(lhs, rhs)| Stmt::Assign { lhs, rhs });

    let stmt = expr_stmt.or(assign_stmt);

    stmt.repeated()
        .map(|stmts| Program { stmts })
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
