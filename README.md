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

## compiler

**aadl.pest**解析aadl源文件（/AADLSource/*.aadl案例）。

**transform.rs**将解析后的pairs结构 -> 自定义的AST(**ast.rs**)中。

**converter.rs**支持aadl_ast -> 轻量级rust_ast(**intermediate_ast.rs**)：

​	/implementations、/types文件夹下的conv_*.rs文件，按AADL组件的分类，分别对相应的组件进行转换。

​	collector.rs：在转换开始前/结束后对aadl_ast进行一些扫描，获取信息。

**intermediate_print.rs**打印rust代码(存储在/generate/)。

## model_eval

简单的AADL模型自动评估工具，统计结构规模和行为复杂度。

## test

各案例生成的代码在Linux下可运行的完整项目。

### test coverage
```shell
cd compiler
make cov

# see readme from compiler/generate_test 
# check `/aadl2rust/compiler/target/llvm-cov/html/index.html` for the whole coverage report
```