// build.rs
extern crate bindgen;
extern crate cc;

fn main() {
    println!("cargo:rerun-if-changed=c_src/ping.c");
    println!("cargo:rerun-if-changed=include/ping.h");

    // 编译C代码 - 添加Windows特定配置
    let mut build = cc::Build::new();
    build.file("c_src/ping.c")
         .flag("/std:c11");  // MSVC的C11标准标志

    // Windows下需要特别处理
    if cfg!(target_os = "windows") {
        build.flag("/TC");  // 强制作为C代码编译
    }

    build.compile("ping");

    // 生成绑定
    let bindings = bindgen::Builder::default()
        .header("include/ping.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg("-Iinclude")  // 添加头文件搜索路径
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("c_bindings.rs"))
        .expect("Couldn't write bindings!");
}