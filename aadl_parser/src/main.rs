pub mod aadlight_parser;
mod ast;
pub mod transform;
pub mod printmessage;
mod aadlAst2rustCode;

use aadlight_parser::AADLParser;
use pest::{Parser};
use core::error;
use std::fs;
use printmessage::*;
use aadlAst2rustCode::intermediate_print::*;



use syn::{parse_str,ItemFn};

use crate::{ast::aadl_ast_cj::Package, aadlAst2rustCode::converter::AadlConverter};

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
            let ast: Vec<ast::aadl_ast_cj::Package> = transform::AADLTransformer::transform_file(pairs.clone().collect());
            println!("=== 转换得到 {} 个package ===", ast.len()); // 调试输出

            //打印原始解析树
            // println!("\n================================== Parse Tree ==================================");
            // for pair in pairs {
            //    print_pair(pair, 0);
            // }

            // 打印AST
            println!("\n================================== AST ==================================");
            print_ast(&ast);

            //
            println!("\n====================================test ===================================");
            for Package in & ast{
                let rust_code = generate_rust_code2(&Package);
                //println!("{}",rust_code);
            }
            
            // 生成Rust代码
            // println!("开始生成Rust代码...\n");
            // for package in &ast {
            //     let rust_code = package.to_rust();
            //     println!("{}", rust_code);
                
            //     // 可以选择将代码写入文件
            //     let output_path = format!("{}.rs", package.name.to_string().replace("::", "_"));
            //     if let Err(e) = fs::write(&output_path, &rust_code) {
            //         eprintln!("写入文件 {} 失败: {}", output_path, e);
            //     } else {
            //         println!("已生成: {}", output_path);
            //     }
            // }
        }
        Err(e) => {
            eprintln!("解析失败: {}", e);
        }
    }
    let code = "fn hello() { println!(\"Hello, world!\"); }";
    let ast: ItemFn = parse_str(code).unwrap();
    
    // 打印整个 AST 的调试信息
    //println!("{:#?}", ast);
    
}
    
// pub fn generate_rust_code(aadl_pkg: &Package) -> () {
//     // 第一级转换：语义转换
//     let rust_ast = AadlConverter::default().convert_package(aadl_pkg);
//     println!("\n==================================== IntermediateRoot ===================================");
//     println!("{:#?}",rust_ast);
    
//     let mut generator = RustCodeGenerator::new();
//     let mut printer = RustPrinter::new();
//     if let Err(e) = printer.print_root(&rust_ast){
//         eprint!("格式化错误cj:{}",e);
//         return;
//     }

//     match printer.into_code() {
//         Ok(code) => {
//             println!("{}",code);
            
//             // 写入文件
//             if let Err(e) = fs::write("output.rs", code) {
//                 eprintln!("写入文件失败: {}", e);
//             } else {
//                 println!("代码已成功写入 output.rs 文件");
//             }
//         } 
//         Err(errors) => {
//             eprint!("转换错误cj:");
//             for error in errors{
//                 eprint!("- {}",error);
//             }
//         }
//     }
// }

pub fn generate_rust_code2(aadl_pkg: &Package) -> () {
    // 第一级转换：语义转换
    let converter = AadlConverter::default();

    let rust_module = converter.convert_package(&aadl_pkg);
    println!("\n==================================== rust_module ===================================");
    println!("{:#?}",rust_module);
    
    let mut code_generator = RustCodeGenerator::new();
    let rust_code = code_generator.generate_module_code(&rust_module);
    println!("{}", rust_code);

    // 写入文件
            if let Err(e) = fs::write("output.rs", rust_code) {
                eprintln!("写入文件失败: {}", e);
            } else {
                println!("代码已成功写入 output.rs 文件");
            }
    
}