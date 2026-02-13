# aadl2rust

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

## **Module Overview**

**aadl.pest** parses AADL source files (e.g., `/AADLSource/*.aadl` cases).

**transform.rs** converts the parsed `Pairs` structure into the custom AADL AST defined in **ast.rs**.

**converter.rs** supports the transformation from `aadl_ast` to a lightweight `rust_ast` (defined in **intermediate_ast.rs**).

The **`/implementations`** and **`/types`** directories contain **`conv_*.rs`** files, which translate corresponding AADL component categories.

**collector.rs** performs scans on the `aadl_ast` before and after conversion to collect necessary information.

**intermediate_print.rs** prints the generated Rust code (stored in `/generate/`).

**model_statistics.rs** uses the Pest parsing results to count different types of components in the AADL model. It is invoked during each code generation run, and the results are written to the `/generate/statistics/` directory.

