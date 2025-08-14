mod aadlAst2rustCode;
pub mod aadlight_parser;
mod ast;
pub mod printmessage;
pub mod transform;
//mod output_ocarina;

use aadlAst2rustCode::generate_build::*;
use aadlAst2rustCode::intermediate_print::*;
use aadlAst2rustCode::merge_utils::*;
use aadlight_parser::AADLParser;
use core::error;
use pest::Parser;
use printmessage::*;
use std::fs;

use syn::{parse_str, ItemFn};

use crate::{aadlAst2rustCode::converter::AadlConverter, ast::aadl_ast_cj::Package};

fn main() {
    //let path = "AADLSource/pingpong_ocarina.aadl";
    //let path = "pingpong.aadl";
    let path = "AADLSource/rma.aadl";
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
            let ast: Vec<ast::aadl_ast_cj::Package> =
                transform::AADLTransformer::transform_file(pairs.clone().collect());
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
            println!(
                "\n====================================test ==================================="
            );
            for Package in &ast {
                generate_rust_code2(&Package);
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

    // 打印整个 AST 的调试信息
    //println!("{:#?}", ast);
}

pub fn generate_rust_code2(aadl_pkg: &Package) -> () {
    // 第一级转换：语义转换
    let converter = AadlConverter::default();

    let rust_module = converter.convert_package(&aadl_pkg);
    println!(
        "\n==================================== rust_module ==================================="
    );
    //println!("{:#?}",rust_module);
    fs::write("rustast0.txt", format!("{:#?}", rust_module)).unwrap();
    let merge_rust_module = merge_item_defs(rust_module);
    //let merge_rust_module=rust_module.clone();
    fs::write("rustast.txt", format!("{:#?}", merge_rust_module)).unwrap();

    let mut code_generator = RustCodeGenerator::new();
    let rust_code = code_generator.generate_module_code(&merge_rust_module);
    //println!("{}", rust_code);

    // 生成 build.rs
    //let build_rs_content = generate_build_rs(&merge_rust_module);
    //fs::write("build.rs", build_rs_content).expect("Failed to write build.rs");

    // 同时保存主Rust代码
    fs::write("generate_main2.rs", rust_code).expect("Failed to write main.rs");
}
