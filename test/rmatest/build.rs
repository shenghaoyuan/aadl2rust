fn main() {
    println!("cargo:rerun-if-changed=c_src/hello.c");
    println!("cargo:rerun-if-changed=include/hello.h");

    cc::Build::new()
        .file("c_src/hello.c")
        .include("include")
        .flag_if_supported("-std=c11") // Linux 下使用 -std 语法
        .compile("c_lib");

    bindgen::Builder::default()
        .header("include/hello.h")
        .clang_arg("-Iinclude") // 确保能找到头文件
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("生成绑定失败")
        .write_to_file(
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
                .join("aadl_c_bindings.rs")
        )
        .expect("写入绑定文件失败");
}
