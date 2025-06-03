pub mod aadlight_parser;
mod ast;
pub mod transform;
pub mod printmessage;
pub mod aadl_to_rust;

use aadl_to_rust::*;
use aadlight_parser::AADLParser;
use pest::{Parser};
use std::fs;
use printmessage::*;

fn main() {
    let path = "pingpong.aadl";
    let aadl_input = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("读取文件失败: {}", err);
            return;
        }
    };
    
    match AADLParser::parse(aadlight_parser::Rule::file, &aadl_input) {
        Ok(pairs) => {
            println!("=== 解析成功，共 {} 个pair ===", pairs.clone().count()); // 调试输出

            // 转换到AST
            let ast = transform::AADLTransformer::transform_file(pairs.clone().collect());
            println!("=== 转换得到 {} 个package ===", ast.len()); // 调试输出

            // 打印AST
            println!("\n================================== AST ==================================");
            print_ast(&ast);
            
            //打印原始解析树
            println!("\n================================== Parse Tree ==================================");
            for pair in pairs {
                print_pair(pair, 0);
            }

            
            // 生成Rust代码
            println!("开始生成Rust代码...\n");
            for package in &ast {
                let rust_code = package.to_rust();
                println!("{}", rust_code);
                
                // 可以选择将代码写入文件
                let output_path = format!("{}.rs", package.name.to_string().replace("::", "_"));
                if let Err(e) = fs::write(&output_path, &rust_code) {
                    eprintln!("写入文件 {} 失败: {}", output_path, e);
                } else {
                    println!("已生成: {}", output_path);
                }
            }
        }
        Err(e) => {
            eprintln!("解析失败: {}", e);
        }
    }
}
    