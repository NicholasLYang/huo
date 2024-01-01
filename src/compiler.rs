use crate::ast::{BinaryOp, Expr, Loc, Program, Stmt};
use candle_core::DType;
use itertools::Itertools;
use quote::{format_ident, quote};

#[derive(Debug, Default)]
pub struct Compiler {
    output: Vec<proc_macro2::TokenStream>,
}

impl Compiler {
    fn compile_data_type(data_type: DType) -> proc_macro2::TokenStream {
        match data_type {
            DType::F32 => quote! { DType::F32 },
            DType::F64 => quote! { DType::F64 },
            DType::I64 => quote! { DType::I64 },
            DType::U32 => quote! { DType::U32 },
            DType::U8 => quote! { DType::U8 },
            DType::BF16 => quote! { DType::BF16 },
            DType::F16 => quote! { DType::F16 },
        }
    }

    pub fn compile_program(&mut self, program: Loc<Program>) {
        for stmt in program.0.stmts {
            self.compile_stmt(stmt);
        }
    }

    fn compile_stmt(&mut self, stmt: Loc<Stmt>) {
        match stmt.0 {
            Stmt::Expr(expr) => {
                self.compile_expr(expr);
                self.output.push(quote! { ; });
            }
            Stmt::Assign { lhs, rhs } => {
                let lhs = format_ident!("{}", lhs.0);
                self.output.push(quote! { let #lhs = });
                self.compile_expr(rhs);
                self.output.push(quote! { ; });
            }
        }
    }

    fn compile_expr(&mut self, expr: Loc<Expr>) {
        match expr.0 {
            Expr::FillTensor {
                fill,
                shape,
                data_type,
            } => {
                let data_type =
                    Self::compile_data_type(data_type.map(|dt| dt.0).unwrap_or(DType::F32));
                let shape = shape.iter().map(|dim| dim.0);
                let fill = fill.0;
                self.output.push(match fill {
                    0 => quote! { Tensor::zeros((#(#shape),*), #data_type, &Device::Cpu) },
                    1 => quote! { Tensor::ones((#(#shape),*), #data_type, &Device::Cpu) },
                    _ => quote! { Tensor::full((#(#shape),*), #fill, #data_type, &Device::Cpu) },
                })
            }
            Expr::RangeTensor {
                start,
                stop,
                step: Some(step),
            } => {
                let start = start.0;
                let stop = stop.0;
                let step = step.0;
                self.output.push(quote! {
                    Tensor::arange_step(#start, #stop, #step, DType::I64, &Device::Cpu)
                })
            }
            Expr::RangeTensor {
                start,
                stop,
                step: None,
            } => {
                let start = start.0;
                let stop = stop.0;

                self.output.push(quote! {
                Tensor::arange(#start, #stop, DType::I64, &Device::Cpu)
                })
            }
            Expr::Variable(name) => {
                let name = format_ident!("{}", name);
                self.output.push(quote! { #name });
            }
            Expr::Binary { lhs, rhs, op } => {
                self.compile_expr(*lhs);
                match &*op {
                    BinaryOp::Add => self.output.push(quote! { + }),
                    BinaryOp::Sub => self.output.push(quote! { - }),
                    BinaryOp::Mul => self.output.push(quote! { * }),
                }
                self.compile_expr(*rhs);
            }
        }
    }

    pub fn print(&self) -> Result<(), anyhow::Error> {
        let streams = self
            .output
            .iter()
            .map(|stream| stream.to_string())
            .join(" ");
        let stream_str = format!("fn main() {{ {} }}", streams.to_string());
        let parsed_file = syn::parse_file(stream_str.as_str())?;
        let pretty_output = prettyplease::unparse(&parsed_file);
        println!("{}", pretty_output);

        Ok(())
    }
}
