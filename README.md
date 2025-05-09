# aadl2rust
## aadl_parser

使用pest解析aadl源码

## codegen_test_subpro项目结构

- 数据结构
- Handlebars模板
- 代码生成器

生成的rust代码作为模块，在test_gen中测试运行

## ast

根据aadl标准文件中的BNF，在rust中定义抽象语法

为便于CTRL F，目前未模块化管理代码，整个ast在ast.rs单个文件中