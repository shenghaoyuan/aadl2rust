use super::intermediate_ast::*;

pub fn generate_build_rs(module: &RustModule) -> String {
    let mut build_rs = String::new();
    let mut c_files = std::collections::HashSet::new();

    // 扫描所有模块文档，收集C文件
    for item in &module.items {
        if let Item::Mod(submodule) = item {
            for doc_line in &submodule.docs {
                if let Some(line) = doc_line.strip_prefix("// source_files: ") {
                    for file in line.split(',') {
                        let file = file.trim().trim_matches('"').trim_matches('\'');
                        if !file.is_empty() {
                            c_files.insert(file.to_string());
                        }
                    }
                }
            }
        }
    }

    // 开始生成build.rs内容
    build_rs.push_str("fn main() {\n");

    // 生成文件监视指令
    for file in &c_files {
        build_rs.push_str(&format!(
            "    println!(\"cargo:rerun-if-changed=c_src/{}\");\n", 
            file
        ));
        let header = file.replace(".c", ".h");
        build_rs.push_str(&format!(
            "    println!(\"cargo:rerun-if-changed=include/{}\");\n",
            header
        ));
    }

    // 生成C编译指令
    if !c_files.is_empty() {
        build_rs.push_str(
            r#"
    // 编译C代码
    cc::Build::new()
"#);

        for file in &c_files {
            build_rs.push_str(&format!("        .file(\"c_src/{}\")\n", file));
        }

        build_rs.push_str(
            r#"        .include("include")
        .flag_if_supported("/std:c11")
        .flag_if_supported("/TC")
        .compile("c_lib");

    // 生成Rust绑定
    bindgen::Builder::default()
"#);

        for file in &c_files {
            build_rs.push_str(&format!(
                "        .header(\"include/{}\")\n",
                file.replace(".c", ".h")
            ));
        }

        build_rs.push_str(
            r#"        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("生成绑定失败")
        .write_to_file(
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
                .join("aadl_c_bindings.rs")
        )
        .expect("写入绑定文件失败");
"#);
    } else {
        build_rs.push_str("    // 没有检测到C源文件，跳过C编译\n");
    }

    build_rs.push_str("}\n");
    build_rs
}