fn main() {
    println!("cargo:rerun-if-changed=c_src/ping.c");
    println!("cargo:rerun-if-changed=include/ping.h");

    // 1. 编译C代码（跨平台优化）
    cc::Build::new()
        .file("c_src/ping.c")
        .include("include")
        .flag_if_supported("/std:c11")  // 自动跳过不支持此flag的平台
        .flag_if_supported("/TC")       // 同上，替代cfg判断
        .compile("ping");

    // 2. 生成Rust绑定（简化配置）
    bindgen::Builder::default()
        .header("include/ping.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("生成绑定失败")
        .write_to_file(
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("c_bindings.rs")
        )
        .expect("写入绑定文件失败");
}
