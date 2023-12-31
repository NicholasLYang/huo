use crate::ast::{BinaryOp, Expr};
use candle_core::DType;
use std::collections::HashMap;

enum Type {
    Tensor { shape: Vec<usize>, dtype: DType },
    Scalar(DType),
}

enum TypeError {
    IncompatibleDataTypes {
        lhs: DType,
        rhs: DType,
    },
    CannotMultiplyVector {
        lhs_shape: Vec<usize>,
        rhs_shape: Vec<usize>,
    },
    IncompatibleShapes {
        lhs_shape: Vec<usize>,
        rhs_shape: Vec<usize>,
    },
}

struct TypeChecker {
    symbols: HashMap<String, Type>,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    fn check_expr(&mut self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::FillTensor {
                shape, data_type, ..
            } => {
                let dtype = data_type.unwrap_or(DType::F32);
                Some(Type::Tensor {
                    shape: shape.clone(),
                    dtype,
                })
            }
            Expr::RangeTensor { start, stop, step } => {
                let step = step.unwrap_or(1);
                Some(Type::Tensor {
                    shape: vec![((stop - start) / step) as usize],
                    dtype: DType::I64,
                })
            }
            Expr::Binary { lhs, rhs, op } => match op {
                BinaryOp::Mul => {
                    let lhs = self.check_expr(lhs)?;
                    let rhs = self.check_expr(rhs)?;
                    match (lhs, rhs) {
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
                                    lhs: scalar_dtype,
                                    rhs: tensor_dtype,
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
                                    lhs: lhs_dtype,
                                    rhs: rhs_dtype,
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
                        ) => self.check_matrix_multiply(lhs_shape, lhs_dtype, rhs_shape, rhs_dtype),
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
        rhs_shape: Vec<usize>,
        rhs_dtype: DType,
    ) -> Option<Type> {
        if lhs_dtype != rhs_dtype {
            self.errors.push(TypeError::IncompatibleDataTypes {
                lhs: lhs_dtype,
                rhs: rhs_dtype,
            });

            return None;
        }

        if lhs_shape.len() < 2 {
            self.errors.push(TypeError::CannotMultiplyVector {
                lhs_shape,
                rhs_shape,
            });

            return None;
        }

        if lhs_shape.len() != rhs_shape.len() {
            self.errors.push(TypeError::IncompatibleShapes {
                lhs_shape: lhs_shape.clone(),
                rhs_shape: rhs_shape.clone(),
            });
            return None;
        }

        todo!();
    }
}
