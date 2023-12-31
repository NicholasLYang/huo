mod ast;
mod parser;
mod type_checker;

use ast::{BinaryOp, Expr, Stmt};
use candle_core::DType;
use itertools::Itertools;
use quote::{format_ident, quote};

#[derive(Debug, Default)]
struct Compiler {
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

    fn compile_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.compile_expr(expr);
                self.output.push(quote! { ; });
            }
            Stmt::Assign { lhs, rhs } => {
                let lhs = format_ident!("{}", lhs);
                self.output.push(quote! { let #lhs = });
                self.compile_expr(rhs);
                self.output.push(quote! { ; });
            }
        }
    }

    fn compile_expr(&mut self, expr: Expr) {
        match expr {
            Expr::FillTensor {
                fill,
                shape,
                data_type,
            } => {
                let data_type = Self::compile_data_type(data_type.unwrap_or(DType::F32));
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
            } => self.output.push(quote! {
                Tensor::arange_step(#start, #stop, #step, DType::I64, &Device::Cpu)
            }),
            Expr::RangeTensor {
                start,
                stop,
                step: None,
            } => self.output.push(quote! {
                Tensor::arange(#start, #stop, DType::I64, &Device::Cpu)
            }),
            Expr::Binary { lhs, rhs, op } => {
                self.compile_expr(*lhs);
                match op {
                    BinaryOp::Add => self.output.push(quote! { + }),
                    BinaryOp::Sub => self.output.push(quote! { - }),
                    BinaryOp::Mul => self.output.push(quote! { * }),
                }
                self.compile_expr(*rhs);
            }
        }
    }

    fn print(&self) -> Result<(), anyhow::Error> {
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

fn main() -> Result<(), anyhow::Error> {
    let mut compiler = Compiler::default();
    compiler.compile_stmt(Stmt::Assign {
        lhs: "tensor".to_string(),
        rhs: Expr::Binary {
            rhs: Box::new(Expr::FillTensor {
                fill: 0,
                shape: vec![2, 3],
                data_type: None,
            }),
            lhs: Box::new(Expr::RangeTensor {
                start: 0,
                stop: 10,
                step: None,
            }),
            op: BinaryOp::Mul,
        },
    });
    compiler.print()?;

    Ok(())
}
