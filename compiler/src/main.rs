mod aadlAst2rustCode;
pub mod aadlight_parser;
mod ast;
pub mod printmessage;
pub mod transform;
pub mod transform_annex;
//mod output_ocarina;

use aadlAst2rustCode::intermediate_print::*;
use aadlAst2rustCode::merge_utils::*;
use aadlight_parser::AADLParser;
use pest::Parser;
use pest::error::ErrorVariant;
use printmessage::*;
use std::fs;
use std::io::{self, Write};


use crate::{aadlAst2rustCode::converter::AadlConverter, ast::aadl_ast_cj::Package};

// 定义测试用例结构
pub struct TestCase {
    pub id: u32,
    pub name: String,
    pub path: String,
    pub output_name: String,
}

fn main() {
    // 定义可用的测试用例
    let test_cases = vec![
        TestCase {
            id: 1,
            name: "PingPong (Ocarina)".to_string(),
            path: "AADLSource/pingpong_ocarina.aadl".to_string(),
            output_name: "pingpong_ocarina".to_string(),
        },
        TestCase {
            id: 2,
            name: "PingPong (Simple)".to_string(),
            path: "pingpong.aadl".to_string(),
            output_name: "pingpong_simple".to_string(),
        },
        TestCase {
            id: 3,
            name: "RMA".to_string(),
            path: "AADLSource/rma.aadl".to_string(),
            output_name: "rma".to_string(),
        },
        TestCase {
            id: 4,
            name: "Toy".to_string(),
            path: "AADLSource/toy.aadl".to_string(),
            output_name: "toy".to_string(),
        },
        TestCase {
            id: 5,
            name: "Robot(v1)".to_string(),
            path: "AADLSource/robotv1.aadl".to_string(),
            output_name: "robotv1".to_string(),
        },
        TestCase {
            id: 6,
            name: "Robot(v2)".to_string(),
            path: "AADLSource/robotv2.aadl".to_string(),
            output_name: "robotv2".to_string(),
        },
        TestCase {
            id: 7,
            name: "RMS".to_string(),
            path: "AADLSource/rms.aadl".to_string(),
            output_name: "rms".to_string(),
        },
        TestCase {
            id: 8,
            name: "PingPong (Timed)".to_string(),
            path: "AADLSource/pingpong_timed_aperiodic.aadl".to_string(),
            output_name: "pingpong_timed_aperiodic".to_string(),
        },
        TestCase {
            id: 9,
            name: "base_types".to_string(),
            path: "AADLSource/base_types.aadl".to_string(),
            output_name: "base_types".to_string(),
        },
        TestCase {
            id: 10,
            name: "composite_types".to_string(),
            path: "AADLSource/composite_types.aadl".to_string(),
            output_name: "composite_types".to_string(),
        },
        TestCase {
            id: 11,
            name: "car".to_string(),
            path: "AADLSource/car.aadl".to_string(),
            output_name: "car".to_string(),
        },
    ];

    // 显示可用的测试用例
    println!("=== AADL2Rust 测试用例选择 ===");
    println!("请选择要测试的AADL文件:");
    for test_case in &test_cases {
        println!("  {}: {}", test_case.id, test_case.name);
    }
    println!("  0: 退出程序");
    print!("请输入选择 (0-{}): ", test_cases.len());
    io::stdout().flush().unwrap();

    // 读取用户输入
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("无法读取输入");
    
    let choice: u32 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("无效输入，请输入数字");
            return;
        }
    };

    if choice == 0 {
        println!("程序退出");
        return;
    }

    // 查找选择的测试用例
    let selected_test = test_cases.iter().find(|tc| tc.id == choice);
    match selected_test {
        Some(test_case) => {
            println!("选择: {}", test_case.name);
            println!("文件路径: {}", test_case.path);
            
            // 确保generate目录存在
            if !fs::metadata("generate").is_ok() {
                fs::create_dir("generate").expect("无法创建generate目录");
            }
            
            // 处理选中的测试用例
            process_test_case(test_case);
        }
        None => {
            println!("无效的选择，请输入 0-{} 之间的数字", test_cases.len());
        }
    }
}

fn process_test_case(test_case: &TestCase) {
    println!("开始处理: {}", test_case.name);
    
    let aadl_input = match fs::read_to_string(&test_case.path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("读取文件失败: {}", err);
            return;
        }
    };
    
    match AADLParser::parse(aadlight_parser::Rule::file, &aadl_input) {
        Ok(pairs) => {
            println!("=== 解析成功，共 {} 个pair ===", pairs.clone().count());
            
            // 将解析结果写入文件
            let pairs_debug_path = format!("generate/temp/{}_pairs_debug.txt", test_case.output_name);
            fs::write(&pairs_debug_path, format!("{:#?}", pairs)).unwrap();
            println!("解析结果已保存到: {}", pairs_debug_path);

            // 转换到AST
            let ast: Vec<ast::aadl_ast_cj::Package> =
                transform::AADLTransformer::transform_file(pairs.clone().collect());
            println!("=== 转换得到 {} 个package ===", ast.len());

            // 打印AST
            println!("\n================================== AST ==================================");
            print_ast(&ast);

            println!("\n==================================== 生成Rust代码 ===================================");
            for (index, package) in ast.iter().enumerate() {
                generate_rust_code_for_test_case(package, test_case,ast.len());
            }
            
            println!("✅ 代码生成完成！输出文件保存在 generate/ 目录下");
        }
        Err(e) => {
            eprintln!("解析失败: {}", e);
            // 打印详细的错误信息
            eprintln!("解析错误: {:?}", e);
            
                        // 显示错误位置和上下文
            eprintln!("错误位置: {:?}", e.location);
            
            
            // 显示期望的规则
            if let ErrorVariant::ParsingError { positives, negatives } = e.variant {
                if !positives.is_empty() {
                    eprintln!("期望匹配的规则: {:?}", positives);
                }
                if !negatives.is_empty() {
                    eprintln!("不应该匹配的规则: {:?}", negatives);
                }
            }
            
            eprintln!("解析失败，无法继续处理");
        }
    }
}

pub fn generate_rust_code_for_test_case(aadl_pkg: &Package, test_case: &TestCase,number_of_packages: usize) -> () {
    // 第一级转换：语义转换
    let mut converter = AadlConverter::default();

    let rust_module = converter.convert_package(&aadl_pkg);
    println!("\n==================================== rust_module ===================================");
    
    // 保存中间AST到文件
    let ast_debug_path = format!("generate/temp/{}_ast_debug.txt", test_case.output_name);
    fs::write(&ast_debug_path, format!("{:#?}", rust_module)).unwrap();
    println!("中间AST已保存到: {}", ast_debug_path);
    
    let merge_rust_module = merge_item_defs(rust_module);
    
    let merged_ast_path = format!("generate/temp/{}_merged_ast.txt", test_case.output_name);
    fs::write(&merged_ast_path, format!("{:#?}", merge_rust_module)).unwrap();
    println!("合并后AST已保存到: {}", merged_ast_path);

    let mut code_generator = RustCodeGenerator::new();
    let rust_code = code_generator.generate_module_code(&merge_rust_module);

    // 根据包数量决定输出路径
    let package_name = aadl_pkg.name.to_string().replace("::", "_");
    let output_path = if number_of_packages == 1 {
        // 如果只有一个包，直接生成文件，文件名是test_case
        format!("generate/code/{}.rs", test_case.output_name)
    } else {
        // 如果有多个包，使用文件夹结构
        let output_dir = format!("generate/code/{}", test_case.output_name);
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");
        format!("{}/{}.rs", output_dir, package_name)
    };
    
    fs::write(&output_path, rust_code).expect("Failed to write Rust code");
    println!("Rust代码已生成 (包: {}): {}", package_name, output_path);

    // 可选：生成build.rs
    // let build_rs_content = generate_build_rs(&merge_rust_module);
    // let build_rs_path = format!("generate/build_{}.rs", test_case.output_name);
    // fs::write(&build_rs_path, build_rs_content).expect("Failed to write build.rs");
    // println!("Build.rs已生成: {}", build_rs_path);
}

// 保留原来的函数作为备用
pub fn generate_rust_code2(aadl_pkg: &Package) -> () {
    // 第一级转换：语义转换
    let mut converter = AadlConverter::default();

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
    fs::write("generate/generate_pingpong.rs", rust_code).expect("Failed to write main.rs");
}
