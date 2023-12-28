// Ideas for syntax:
// zeros: [2 x 3; 0]
// ones:  [2 x 3; 1]
// arange: [0..10]
// arange w/step: [0..10; 2]
// range: [0..=10]
//
// a = [2 x 3; 0]
// b = [3 x 4; 1]
// c = a * b

use candle_core::DType;
use quote::quote;

enum Expr {
    Tensor {
        fill: i32,
        shape: Vec<usize>,
        data_type: Option<DType>
    },
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

enum BinaryOp {
    Add,
    Sub,
    Mul,
}

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

    fn compile_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Tensor { fill, shape, data_type } => {
                let data_type = Self::compile_data_type(data_type.unwrap_or(DType::F32));
                self.output.push(match fill {
                    0 => quote! { Tensor::zeros((#(#shape),*), #data_type, &Device::Cpu) },
                    1 => quote! { Tensor::ones((#(#shape),*), #data_type, &Device::Cpu) },
                    _ => quote! { Tensor::full((#(#shape),*), #fill, #data_type, &Device::Cpu) },
                })
            }
            Expr::RangeTensor { .. } => {
                todo!()
            }
            Expr::Binary { .. } => {
                todo!()
            }
        }
    }

    fn print(&self) -> Result<(), anyhow::Error> {
        for stream in &self.output {
            let stream_str = format!("fn main() {{ {} }}", stream.to_string());
            let parsed_file = syn::parse_file(stream_str.as_str())?;
            let pretty_output = prettyplease::unparse(&parsed_file);
            println!("{}", pretty_output);
        }

        Ok(())
    }
}

fn main() -> Result<(), anyhow::Error> {
    let mut compiler = Compiler::default();
    compiler.compile_expr(Expr::Tensor {
        fill: 0,
        shape: vec![2, 3],
        data_type: None,
    });
    compiler.print()?;

    Ok(())
}
