use crate::ast::{BinaryOp, Expr, Loc, Program, Stmt};
use candle_core::DType;
use miette::{Diagnostic, SourceSpan};
use std::collections::HashMap;
use std::ops::Range;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    Tensor { shape: Vec<usize>, dtype: DType },
    Scalar(DType),
}

#[derive(Debug, Error, Diagnostic)]
pub enum TypeError {
    #[error("Data types {lhs} and {rhs} cannot be combined")]
    #[diagnostic(code(type_check::incompatible_data_types))]
    IncompatibleDataTypes {
        lhs: &'static str,
        rhs: &'static str,
        #[label("a value of this data type")]
        lhs_span: SourceSpan,
        #[label("cannot be combined with a value of this data type")]
        rhs_span: SourceSpan,
    },
    #[error("Cannot multiply tensor of shape {lhs_shape:?} with tensor of shape {rhs_shape:?}")]
    #[diagnostic(code(type_check::cannot_multiply))]
    CannotMultiply {
        lhs_shape: Vec<usize>,
        rhs_shape: Vec<usize>,
        #[label("this shape")]
        lhs_span: SourceSpan,
        #[label("cannot be multiplied with this shape")]
        rhs_span: SourceSpan,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SymbolEntry {
    ty: Type,
    span: Range<usize>,
}

#[derive(Default)]
pub struct TypeChecker {
    symbols: HashMap<String, SymbolEntry>,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn into_errors(self) -> Vec<TypeError> {
        self.errors
    }
    pub fn check_program(&mut self, program: &Loc<Program>) {
        for stmt in &program.stmts {
            self.check_stmt(stmt);
        }
    }

    fn get_expr_source_span(&self, expr: &Loc<Expr>) -> Range<usize> {
        if let Expr::Variable(name) = &expr.0 {
            if let Some(SymbolEntry { span, .. }) = self.symbols.get(name) {
                return span.clone();
            }
        }

        expr.1.clone()
    }
    fn check_stmt(&mut self, stmt: &Loc<Stmt>) {
        match &stmt.0 {
            Stmt::Expr(expr) => {
                self.check_expr(expr);
            }
            Stmt::Assign { lhs, rhs } => {
                let rhs_type = self.check_expr(rhs);
                if let Some(rhs_type) = rhs_type {
                    self.symbols.insert(
                        lhs.0.clone(),
                        SymbolEntry {
                            ty: rhs_type,
                            span: rhs.1.clone().into(),
                        },
                    );
                }
            }
        }
    }

    fn check_expr(&mut self, expr: &Loc<Expr>) -> Option<Type> {
        match &expr.0 {
            Expr::FillTensor {
                shape, data_type, ..
            } => {
                let dtype = data_type.as_deref().cloned().unwrap_or(DType::F32);
                Some(Type::Tensor {
                    shape: shape.iter().map(|dim| dim.0).collect(),
                    dtype,
                })
            }
            Expr::RangeTensor { start, stop, step } => {
                let step = step.as_deref().cloned().unwrap_or(1);
                let start = start.0;
                let stop = stop.0;
                Some(Type::Tensor {
                    shape: vec![((stop - start) / step) as usize],
                    dtype: DType::I64,
                })
            }
            Expr::Variable(name) => self.symbols.get(name).map(|entry| entry.ty.clone()),
            Expr::Binary { lhs, rhs, op } => match &op.0 {
                BinaryOp::Mul => {
                    let lhs_ty = self.check_expr(lhs)?;
                    let rhs_ty = self.check_expr(rhs)?;
                    let lhs_span = self.get_expr_source_span(lhs);
                    let rhs_span = self.get_expr_source_span(rhs);

                    match (lhs_ty, rhs_ty) {
                        (
                            Type::Tensor {
                                shape,
                                dtype: tensor_dtype,
                            },
                            Type::Scalar(scalar_dtype),
                        )
                        | (
                            Type::Scalar(scalar_dtype),
                            Type::Tensor {
                                shape,
                                dtype: tensor_dtype,
                            },
                        ) => {
                            if scalar_dtype != tensor_dtype {
                                self.errors.push(TypeError::IncompatibleDataTypes {
                                    lhs: scalar_dtype.as_str(),
                                    rhs: tensor_dtype.as_str(),
                                    lhs_span: lhs_span.into(),
                                    rhs_span: rhs_span.into(),
                                });
                                return None;
                            }
                            Some(Type::Tensor {
                                shape,
                                dtype: tensor_dtype,
                            })
                        }
                        (Type::Scalar(lhs_dtype), Type::Scalar(rhs_dtype)) => {
                            if lhs_dtype != rhs_dtype {
                                self.errors.push(TypeError::IncompatibleDataTypes {
                                    lhs: lhs_dtype.as_str(),
                                    rhs: rhs_dtype.as_str(),
                                    lhs_span: lhs_span.into(),
                                    rhs_span: rhs_span.into(),
                                });
                                None
                            } else {
                                Some(Type::Scalar(lhs_dtype))
                            }
                        }
                        (
                            Type::Tensor {
                                shape: lhs_shape,
                                dtype: lhs_dtype,
                            },
                            Type::Tensor {
                                shape: rhs_shape,
                                dtype: rhs_dtype,
                            },
                        ) => self.check_matrix_multiply(
                            lhs_shape, lhs_dtype, lhs_span, rhs_shape, rhs_dtype, rhs_span,
                        ),
                    }
                }
                _ => todo!(),
            },
        }
    }

    fn check_matrix_multiply(
        &mut self,
        lhs_shape: Vec<usize>,
        lhs_dtype: DType,
        lhs_span: Range<usize>,
        rhs_shape: Vec<usize>,
        rhs_dtype: DType,
        rhs_span: Range<usize>,
    ) -> Option<Type> {
        if lhs_dtype != rhs_dtype {
            self.errors.push(TypeError::IncompatibleDataTypes {
                lhs: lhs_dtype.as_str(),
                rhs: rhs_dtype.as_str(),
                lhs_span: lhs_span.into(),
                rhs_span: rhs_span.into(),
            });

            return None;
        }

        if lhs_shape.len() < 2 {
            self.errors.push(TypeError::CannotMultiply {
                lhs_shape,
                rhs_shape,
                lhs_span: lhs_span.into(),
                rhs_span: rhs_span.into(),
            });

            return None;
        }

        if lhs_shape.len() != rhs_shape.len() {
            self.errors.push(TypeError::CannotMultiply {
                lhs_shape: lhs_shape.clone(),
                rhs_shape: rhs_shape.clone(),
                lhs_span: lhs_span.into(),
                rhs_span: rhs_span.into(),
            });
            return None;
        }

        let dimension = lhs_shape.len();
        let lhs_k = lhs_shape[dimension - 1];
        let rhs_k = rhs_shape[dimension - 2];
        let lhs_batch: usize = lhs_shape[..dimension - 2].iter().product();
        let rhs_batch: usize = rhs_shape[..dimension - 2].iter().product();

        if lhs_k != rhs_k || lhs_batch != rhs_batch {
            self.errors.push(TypeError::CannotMultiply {
                lhs_shape: lhs_shape.clone(),
                rhs_shape: rhs_shape.clone(),
                lhs_span: lhs_span.into(),
                rhs_span: rhs_span.into(),
            });
            return None;
        }

        Some(Type::Tensor {
            shape: vec![
                lhs_batch,
                lhs_shape[dimension - 2],
                rhs_shape[dimension - 1],
            ],
            dtype: lhs_dtype,
        })
    }
}
