extern crate bindgen;
extern crate cc;

fn main() {
    println!("cargo:rerun-if-changed=c_src/ping.c");
    println!("cargo:rerun-if-changed=include/ping.h");

    // 编译C代码
    let mut build = cc::Build::new();
    build
        .file("c_src/ping.c")
        .include("include")            // 添加 include 路径
        .flag("/std:c11");

    if cfg!(target_os = "windows") {
        build.flag("/TC");
    }

    build.compile("ping");

    // 生成Rust绑定
    let bindings = bindgen::Builder::default()
        .header("include/ping.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg("-Iinclude")  // 加头文件路径
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("c_bindings.rs"))
        .expect("Couldn't write bindings!");
}
