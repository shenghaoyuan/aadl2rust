# aadl2rust
## aadl_parser

使用pest解析aadl源码

将解析后的结构转换到自定义的AST中

## codegen_test_subpro项目结构

- 数据结构
- Handlebars模板
- 代码生成器

生成的rust代码作为模块，在test_gen中测试运行

## ast

根据aadl标准文件中的BNF，在rust中定义抽象语法

为便于CTRL F，目前未模块化管理代码，整个ast在ast.rs单个文件中

use ast目前的问题:

1. **属性表达式**：当前的PropertyExpression系统还不完善，特别是对于复杂属性值如范围(0 ms..1 ms)、引用(reference (the_mem))等的表示方式。
2. **单位处理**：虽然定义了SignedInteger和SignedReal，但没有完整的单位系统来处理像"200 KByte"、"2000 Ms"这样的带单位值。
3. **属性应用范围**：原代码中的"applies to"语法在抽象语法中没有明确定义如何表示。
4. **子程序调用序列**：原代码中的"{ c : subprogram sender_spg; }"语法在抽象语法中的表示方式需要更明确的定义。
5. **数据端口和参数的区别**：在抽象语法中，参数和端口都使用了PortSpec来表示，可能需要更明确的区分。

## syn_test

demo，尝试aadl_ast -> rust_ast -> rust code的流程。

使用syn、quote、prettyplease。