fn main() {
    println!("cargo:rerun-if-changed=c_src/robot.c");
    println!("cargo:rerun-if-changed=c_include/robot.h");

    cc::Build::new()
        .file("c_src/robot.c")
        .include("c_include")
        .flag_if_supported("-std=c11") // Linux 下使用 -std 语法
        .compile("c_lib");

    bindgen::Builder::default()
        .header("c_include/robot.h")
        .clang_arg("-Ic_include") // 确保能找到头文件
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("生成绑定失败")
        .write_to_file(
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
                .join("aadl_c_bindings.rs")
        )
        .expect("写入绑定文件失败");
}
