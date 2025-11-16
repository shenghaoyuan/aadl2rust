# aadl2rust
## compiler

**aadl.pest**解析aadl源文件（/AADLSource/*.aadl案例）。

**transform.rs**将解析后的pairs结构 -> 自定义的AST(**ast.rs**)中。

**converter.rs**支持aadl_ast -> 轻量级rust_ast(**intermediate_ast.rs**)。

**intermediate_print.rs**打印rust代码(存储在/generate/)。

## model_eval

简单的AADL模型自动评估工具，统计结构规模和行为复杂度。

## test

各案例生成的代码在Linux下可运行的完整项目。