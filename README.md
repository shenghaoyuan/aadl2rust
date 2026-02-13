# AADL2Rust

## Environment
we have tested on 

```shell
# 
# - rustc 1.89.0-nightly (e703dff8f 2025-06-11)
# - cargo-llvm-cov v0.6.10
# or
# - rustc 1.91.0 (f8297e351 2025-10-28)
# - cargo-llvm-cov v0.6.21

# run `rustup component add llvm-tools-preview --toolchain nightly-x86_64-unknown-linux-gnu` to install the `llvm-tools-preview`
```

Some additional packages are required:
```shell
sudo apt install -y jq
cargo install tokei
```

## Usage

```shell
cd compiler
cargo test #run all test cases.
just cov-html / make cov # generate an HTML coverage report. 
			  #output file: "\compiler\target\llvm-cov\html\index.html"
cargo run -- --input <folder_name>   # run a single case
```

To count effective lines of AADL code (excluding blank lines and comments) for each case under `AADLSource/`:

```shell
chmod +x scripts/aadl_loc_by_folder_csv.sh
./scripts/aadl_loc_by_folder_csv.sh # output file:AADLSource/aadl_code_loc_by_folder.csv
```

To count effective lines of Rust code for each generated project under `generate/project/`:

```shell
chmod +x scripts/rust_loc_by_project_csv.sh
./scripts/rust_loc_by_project_csv.sh 
#output file :generate/project_rust_code_loc_by_folder.csv
```

## compiler

**aadl.pest**解析aadl源文件（/AADLSource/*.aadl案例）。

**transform.rs**将解析后的pairs结构 -> 自定义的AST(**ast.rs**)中。

**converter.rs**支持aadl_ast -> 轻量级rust_ast(**intermediate_ast.rs**)：

**/implementations、/types**文件夹下的**conv_*.rs**文件，按AADL组件的分类，分别对相应的组件进行转换。

**collector.rs**：在转换开始前/结束后对aadl_ast进行一些扫描，获取信息。

**intermediate_print.rs**打印rust代码(存储在/generate/)。

**model_statistics.rs**使用pest解析的结果，统计AADL模型中各类型组件的数量，每次执行代码生成时被调用。结果在\generate\statistics\目录下。



**aadl.pest** parses AADL source files (e.g., `/AADLSource/*.aadl` cases).

**transform.rs** converts the parsed `Pairs` structure into the custom AADL AST defined in **ast.rs**.

**converter.rs** supports the transformation from `aadl_ast` to a lightweight `rust_ast` (defined in **intermediate_ast.rs**).

The **`/implementations`** and **`/types`** directories contain **`conv_*.rs`** files, which translate corresponding AADL component categories.

**collector.rs** performs scans on the `aadl_ast` before and after conversion to collect necessary information.

**intermediate_print.rs** prints the generated Rust code (stored in `/generate/`).

**model_statistics.rs** uses the Pest parsing results to count different types of components in the AADL model. It is invoked during each code generation run, and the results are written to the `/generate/statistics/` directory.

