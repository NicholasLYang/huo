use candle_core::DType;

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Stmt {
    Expr(Expr),
    Assign { lhs: String, rhs: Expr },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    // zeros: [2 x 3; 0]
    // ones:  [2 x 3; 1]
    FillTensor {
        fill: i32,
        shape: Vec<usize>,
        data_type: Option<DType>,
    },
    // arange: [0..10]
    // arange w/step: [0..10; 2]
    RangeTensor {
        start: i32,
        stop: i32,
        step: Option<i32>,
    },
    Binary {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: BinaryOp,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
}
