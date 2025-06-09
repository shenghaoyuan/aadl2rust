use anyhow::Result;
use quote::{quote, ToTokens};
use syn::{parse_quote, Ident, Item, ItemImpl, ItemStruct, File};
use std::fs;

// ======================
// 1. 定义自定义 AADL AST
// ======================
#[derive(Debug)]
enum PortDirection {
    Input,
    Output,
}

#[derive(Debug)]
struct AadlPort {
    name: String,
    direction: PortDirection,
    data_type: String,
}

#[derive(Debug)]
struct AadlComponent {
    name: String,
    ports: Vec<AadlPort>,
    is_periodic: bool,
    period_ms: Option<u64>,
}

// ======================
// 2. 转换逻辑 (AADL AST → Rust AST)
// ======================
fn convert_to_rust_ast(aadl: &AadlComponent) -> Result<Vec<Item>> {
    let mut rust_items = Vec::new();

    // 生成结构体
    let struct_name = Ident::new(&aadl.name, proc_macro2::Span::call_site());
    let fields = aadl.ports.iter().map(|port| {
        let port_ident = Ident::new(&port.name, proc_macro2::Span::call_site());
        let port_type = match port.direction {
            PortDirection::Input => quote!(tokio::sync::mpsc::Receiver<Data>),
            PortDirection::Output => quote!(tokio::sync::mpsc::Sender<Data>),
        };
        quote! { #port_ident: #port_type }
    });

    let struct_def: ItemStruct = parse_quote! {
        #[derive(Debug)]
        pub struct #struct_name {
            #(#fields),*
        }
    };
    rust_items.push(Item::Struct(struct_def));

    // 生成实现块
    let impl_block: ItemImpl = if aadl.is_periodic {
        let period = aadl.period_ms.unwrap_or(100);
        parse_quote! {
            impl #struct_name {
                pub async fn run(&mut self) {
                    let mut interval = tokio::time::interval(std::time::Duration::from_millis(#period));
                    loop {
                        interval.tick().await;
                        if let Some(data) = self.input_port.recv().await {
                            println!("Processing: {:?}", data);
                        }
                    }
                }
            }
        }
    } else {
        parse_quote! {
            impl #struct_name {
                pub async fn run(&mut self) {
                    while let Some(data) = self.input_port.recv().await {
                        println!("Processing: {:?}", data);
                    }
                }
            }
        }
    };
    rust_items.push(Item::Impl(impl_block));

    Ok(rust_items)
}

// ======================
// 3. 主函数：从 AADL 生成 Rust 代码
// ======================
fn main() -> Result<()> {
    let aadl_component = AadlComponent {
        name: "SensorThread".to_string(),
        ports: vec![AadlPort {
            name: "input_port".to_string(),
            direction: PortDirection::Input,
            data_type: "Data".to_string(),
        }],
        is_periodic: true,
        period_ms: Some(100),
    };

    // 转换为 Rust AST
    let rust_ast = convert_to_rust_ast(&aadl_component)?;

    // 构造 syn::File（Rust 源文件语法树）
    let file: File = parse_quote! {
        #![allow(unused_imports)]
        use tokio::sync::mpsc;
        use std::time::Duration;

        #(#rust_ast)*
    };
    //println!("{:#?}", file);

    // 使用 prettyplease 格式化
    let formatted = prettyplease::unparse(&file);

    // 写入格式化后的 Rust 代码
    fs::write("generated.rs", formatted)?;

    Ok(())
}
