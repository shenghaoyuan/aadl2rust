fn main() {
    println!("cargo:rerun-if-changed=c_src/hello.c");
    println!("cargo:rerun-if-changed=include/hello.h");

    // 编译C代码
    cc::Build::new()
        .file("c_src/hello.c")
        .include("include")
        .flag_if_supported("/std:c11")
        .flag_if_supported("/TC")
        .compile("c_lib");

    // 生成Rust绑定
    bindgen::Builder::default()
        .header("include/hello.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("生成绑定失败")
        .write_to_file(
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
                .join("aadl_c_bindings.rs")
        )
        .expect("写入绑定文件失败");
}
