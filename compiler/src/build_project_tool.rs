use regex::Regex;
use std::fs;
use std::path::Path;

// 定义测试用例结构
pub struct TestCase {
    pub id: u32,
    pub name: String,
    pub path: String,
    pub output_name: String,
}

pub fn assemble_rust_project(test_case: &TestCase) {
    // ---------------- 项目根目录 ----------------
    let project_root = format!("generate/project/{}", test_case.output_name);

    // ---------------- Cargo.toml ----------------
    generate_cargo_toml(&project_root, &test_case.output_name);

    // ---------------- C / H 文件拷贝 ----------------
    copy_c_sources(&test_case.path, &project_root);

    // ---------------- 收集 C / H 文件名 ----------------
    let mut c_files = Vec::new();
    let mut h_files = Vec::new();

    let input_dir = Path::new(&test_case.path);

    for entry in fs::read_dir(input_dir)
        .expect("Failed to read input directory")
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "c" {
                c_files.push(path.file_name().unwrap().to_string_lossy().to_string());
            } else if ext == "h" {
                h_files.push(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }
    }

    // ---------------- build.rs ----------------
    if c_files.is_empty() || h_files.is_empty() {
        generate_empty_build_rs(&project_root);
    } else {
        generate_build_rs_from_c_files(&project_root, &c_files, &h_files);
    }

    // ---------------- Rust support files ----------------
    generate_common_traits_rs(&project_root);
    generate_lib_rs(&project_root);

    // ---------------- main.rs ----------------
    let (module_name, system_type) =
        find_system_impl(&project_root).expect("未找到 impl System for XXX 的实现");

    generate_main_rs(
        &project_root,
        &system_type,
        &module_name,
        &test_case.output_name,
    );

    println!("📦 项目生成完成: {}", project_root);
}

/// 生成 Cargo.toml
fn generate_cargo_toml(project_root: &str, project_name: &str) {
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[build-dependencies]
cc = {{ version = "1.0", features = ["parallel"] }}
bindgen = "0.69"

[dependencies]
libc = "0.2"
lazy_static = "1.4"
crossbeam-channel = "0.5"
rand = "0.7"
tokio = {{ version = "1.40", features = ["sync"] }}
"#,
        project_name.replace('-', "_")
    );

    fs::write(format!("{}/Cargo.toml", project_root), cargo_toml)
        .expect("Failed to write Cargo.toml");
}

/// 复制 C 源文件和头文件到项目目录
fn copy_c_sources(input_dir: &str, project_root: &str) {
    let c_src_dir = format!("{}/c_src", project_root);
    let c_include_dir = format!("{}/c_include", project_root);

    fs::create_dir_all(&c_src_dir).unwrap();
    fs::create_dir_all(&c_include_dir).unwrap();

    let entries = fs::read_dir(input_dir).unwrap();

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "c" {
                let dest = format!(
                    "{}/{}",
                    c_src_dir,
                    path.file_name().unwrap().to_string_lossy()
                );
                fs::copy(&path, &dest).unwrap();
            } else if ext == "h" {
                let dest = format!(
                    "{}/{}",
                    c_include_dir,
                    path.file_name().unwrap().to_string_lossy()
                );
                fs::copy(&path, &dest).unwrap();
            }
        }
    }
}

/// 生成 build.rs
fn generate_build_rs_from_c_files(project_root: &str, c_files: &[String], h_files: &[String]) {
    let mut build_rs = String::new();

    build_rs.push_str("fn main() {\n");

    // rerun-if-changed
    for c in c_files {
        build_rs.push_str(&format!(
            "    println!(\"cargo:rerun-if-changed=c_src/{}\");\n",
            c
        ));
    }
    for h in h_files {
        build_rs.push_str(&format!(
            "    println!(\"cargo:rerun-if-changed=c_include/{}\");\n",
            h
        ));
    }

    build_rs.push_str("\n    cc::Build::new()\n");

    for c in c_files {
        build_rs.push_str(&format!("        .file(\"c_src/{}\")\n", c));
    }

    build_rs.push_str(
        r#"
        .include("c_include")
        .flag_if_supported("-std=c11")
        .compile("c_lib");
    "#,
    );

    // bindgen：对每个头文件生成一次绑定
    for h in h_files {
        build_rs.push_str("    bindgen::Builder::default()\n");
        build_rs.push_str(&format!("        .header(\"c_include/{}\")\n", h));
        build_rs.push_str(
            r#"
            .clang_arg("-Ic_include")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("生成绑定失败")
            .write_to_file(
                std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
                    .join("aadl_c_bindings.rs")
            )
            .expect("写入绑定文件失败");
        "#,
        );
    }

    build_rs.push_str("}\n");

    let build_rs_path = format!("{}/build.rs", project_root);
    fs::write(&build_rs_path, build_rs).expect("Failed to write build.rs");

    println!("Build.rs 已生成: {}", build_rs_path);
}

/// 生成空的 build.rs（无 C 源文件时使用，但仍生成空绑定文件）
fn generate_empty_build_rs(project_root: &str) {
    let path = format!("{}/build.rs", project_root);

    let content = r#"
use std::{env, fs, path::PathBuf};

fn main() {
    // Cargo 提供的输出目录
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_path = out_dir.join("aadl_c_bindings.rs");

    // 生成一个空的绑定文件，供 include! 使用
    fs::write(
        &bindings_path,
        "// empty native bindings\n",
    )
    .expect("failed to write empty aadl_c_bindings.rs");

    // build.rs 自身变化时触发重新运行
    println!("cargo:rerun-if-changed=build.rs");
}
"#;

    fs::write(&path, content).expect("Failed to write empty build.rs");

    println!("build.rs(空绑定版本)已生成: {}", path);
}

/// 生成 src/common_traits.rs
fn generate_common_traits_rs(project_root: &str) {
    let src_dir = format!("{}/src", project_root);
    let path = format!("{}/common_traits.rs", src_dir);

    let content = r#"// ---------------- System ----------------
pub trait System {
    fn new() -> Self
        where Self: Sized;
    fn run(self);
}

// ---------------- Process ----------------
pub trait Process {
    fn new(cpu_id: isize) -> Self
        where Self: Sized;
    fn run(self);
}

// ---------------- Thread ----------------
pub trait Thread {
    fn new(cpu_id: isize) -> Self
        where Self: Sized;
    fn run(self);
}

// ---------------- Device ----------------
pub trait Device {
    fn new() -> Self
        where Self: Sized;
    fn run(self);
}
"#;

    fs::write(&path, content).expect("Failed to write common_traits.rs");

    println!("common_traits.rs 已生成: {}", path);
}

/// 生成 src/lib.rs
fn generate_lib_rs(project_root: &str) {
    let src_dir = format!("{}/src", project_root);
    let lib_rs_path = format!("{}/lib.rs", src_dir);

    let mut modules = Vec::new();

    let entries = fs::read_dir(&src_dir).expect("Failed to read src directory");

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let file_name = path.file_name().unwrap().to_string_lossy();

            // 排除 lib.rs 和 main.rs
            if file_name == "lib.rs" || file_name == "main.rs" {
                continue;
            }

            // 去掉 .rs 后缀
            let module_name = file_name.trim_end_matches(".rs").to_string();
            modules.push(module_name);
        }
    }

    modules.sort(); // 保证确定性（很重要，便于 diff / 论文复现）

    let mut content = String::new();

    // crate-level attributes
    content.push_str("#![allow(non_snake_case)]\n");
    content.push_str("#![allow(non_camel_case_types)]\n\n");

    // 固定模块
    // content.push_str("pub mod common_traits;\n\n");

    // 自动生成模块声明
    for m in modules {
        content.push_str(&format!("pub mod {};\n", m));
    }

    fs::write(&lib_rs_path, content).expect("Failed to write lib.rs");

    println!("lib.rs 已生成: {}", lib_rs_path);
}

/// 生成 src/main.rs
fn generate_main_rs(project_root: &str, system_type: &str, module_name: &str, project_name: &str) {
    let main_rs_path = format!("{}/src/main.rs", project_root);

    let content = format!(
        r#"use {project_name}::common_traits::System;
use {project_name}::{module_name}::{system_type};

pub fn boot<S: System>() {{
    let system = S::new();
    system.run();

    // 主线程保持运行，防止退出
    loop {{
        std::thread::sleep(std::time::Duration::from_secs(60));
    }}
}}

fn main() {{
    boot::<{system_type}>();
}}
"#,
        system_type = system_type,
        module_name = module_name,
        project_name = project_name.replace('-', "_"),
    );

    fs::write(&main_rs_path, content).expect("Failed to write main.rs");

    println!("main.rs 已生成: {}", main_rs_path);
}

/// 返回 (module_name, system_type)
fn find_system_impl(project_root: &str) -> Option<(String, String)> {
    let src_dir = Path::new(project_root).join("src");

    let re = Regex::new(r"impl\s+System\s+for\s+([A-Za-z_][A-Za-z0-9_]*)").expect("invalid regex");

    let entries = fs::read_dir(&src_dir).ok()?;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        // 只处理 .rs 文件
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let file_name = path.file_name()?.to_string_lossy();

        // 排除固定文件
        if file_name == "lib.rs" || file_name == "main.rs" || file_name == "common_traits.rs" {
            continue;
        }

        let content = fs::read_to_string(&path).ok()?;

        if let Some(caps) = re.captures(&content) {
            let system_type = caps.get(1)?.as_str().to_string();

            // 模块名 = 文件名去掉 .rs
            let module_name = file_name.trim_end_matches(".rs").to_string();

            return Some((module_name, system_type));
        }
    }

    None
}
