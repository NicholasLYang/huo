use candle_core::DType;
use std::ops::{Deref, DerefMut, Range};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loc<T>(pub T, pub Range<usize>);

impl<T> Loc<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Loc<U> {
        Loc(f(self.0), self.1)
    }
}

impl<T> Deref for Loc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Loc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Loc<Stmt>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Stmt {
    Expr(Loc<Expr>),
    Assign { lhs: Loc<String>, rhs: Loc<Expr> },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    // zeros: [2 x 3; 0]
    // ones:  [2 x 3; 1]
    FillTensor {
        fill: Loc<i32>,
        shape: Vec<Loc<usize>>,
        data_type: Option<Loc<DType>>,
    },
    // arange: [0..10]
    // arange w/step: [0..10; 2]
    RangeTensor {
        start: Loc<i32>,
        stop: Loc<i32>,
        step: Option<Loc<i32>>,
    },
    Variable(String),
    Binary {
        lhs: Box<Loc<Expr>>,
        rhs: Box<Loc<Expr>>,
        op: Loc<BinaryOp>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
}
