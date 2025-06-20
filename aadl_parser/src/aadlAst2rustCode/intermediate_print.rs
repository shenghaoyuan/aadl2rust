// src/ir/intermediate_print.rs
use super::intermediate_ast::*;
use std::fmt::{self, Write};
use chrono::Local;

/// Rust代码生成器
pub struct RustCodeGenerator {
    buffer: String,
    indent_level: usize,
}

impl RustCodeGenerator {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            indent_level: 0,
        }
    }

    /// 主入口：生成完整模块代码
    pub fn generate_module_code(&mut self, module: &RustModule) -> String {
        self.buffer.clear();
        
        // 文件头
        self.writeln("// 自动生成的 Rust 代码 - 来自 AADL 模型");
        self.writeln(&format!("// 生成时间: {}", Local::now().format("%Y-%m-%d %H:%M:%S")));
        self.writeln("");
        self.writeln("#![allow(unused_imports)]");
        self.writeln("use std::sync::mpsc;");
        self.writeln("use std::thread;");
        self.writeln("use std::time::{Duration, Instant};");
        self.writeln("");

        // 生成模块内容
        self.generate_items(&module.items);

        self.buffer.clone()
    }

    /// 生成多个项
    fn generate_items(&mut self, items: &[Item]) {
        for item in items {
            self.generate_item(item);
        }
    }

    /// 生成单个项
    fn generate_item(&mut self, item: &Item) {
        match item {
            Item::Struct(s) => self.generate_struct(s),
            Item::Enum(e) => self.generate_enum(e),
            Item::Function(f) => self.generate_function(f),
            Item::Impl(i) => self.generate_impl(i),
            Item::Const(c) => self.generate_const(c),
            Item::TypeAlias(t) => self.generate_type_alias(t),
            Item::Use(u) => self.generate_use(u),
            Item::Mod(m) => self.generate_nested_module(m),
        }
    }

    fn generate_nested_module(&mut self, m: &RustModule) {
        self.writeln(&format!("mod {} {{", m.name));
        self.indent();
        
        // 模块级文档和属性
        for doc in &m.docs {
            self.writeln(doc);
        }
        for attr in &m.attrs {
            self.generate_attribute(attr);
        }
        
        // 模块内容
        self.generate_items(&m.items);
        
        self.dedent();
        self.writeln("}");
        self.writeln("");
    }

    fn generate_struct(&mut self, s: &StructDef) {
        // 文档注释
        for doc in &s.docs {
            self.writeln(doc);
        }

        // 派生属性
        if !s.derives.is_empty() {
            self.write("#[derive(");
            for (i, derive) in s.derives.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(derive);
            }
            self.writeln(")]");
        }

        // 结构体定义
        self.write(&format!("{}struct {} ", self.visibility(&s.vis), s.name));
        
        if s.generics.is_empty() {
            self.writeln("{");
        } else {
            self.write("<");
            for (i, generic) in s.generics.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(&generic.name);
                if !generic.bounds.is_empty() {
                    self.write(": ");
                    for (j, bound) in generic.bounds.iter().enumerate() {
                        if j > 0 { self.write(" + "); }
                        self.write(bound);
                    }
                }
            }
            self.writeln("> {");
        }

        self.indent();
    
        // 1. 生成端口字段
        for field in &s.fields {
            self.generate_field(field);
        }

        // 如果是进程结构体则不生成属性字段
        if !s.name.ends_with("Process") && !s.properties.is_empty() {
            self.writeln("\n    // --- AADL属性 ---");
            for prop in &s.properties {
                self.writeln(&format!(
                    "pub {}: {}, {}",
                    prop.name.to_lowercase(),
                    self.type_for_property(&prop.value),
                    prop.docs.join("\n")
                ));
            }
        }
        self.dedent();
        self.writeln("}");
        self.writeln("");

        self.generate_properties_impl(s);
        
    }

    /// 生成属性初始化impl块
    fn generate_properties_impl(&mut self, s: &StructDef) {
        if s.properties.is_empty() {
            return;
        }

        self.writeln(&format!("impl {} {{", s.name));
        self.writeln("    /// 创建组件并初始化AADL属性");
        self.write("    pub fn new(");
        
        // 生成构造函数参数（只包含端口字段）
        // 这样能够在之后构造 SenderThread 时，把一个真实可用的通道发送端 Sender<i32> 传进来，当然也可以通过直接访问pub的p来设置
        let port_args = s.fields.iter()
            .map(|f| format!("{}: {}", f.name, self.type_to_string(&f.ty)))
            .collect::<Vec<_>>()
            .join(", ");
        self.write(&port_args);
        
        self.writeln(") -> Self {");
        self.writeln("        Self {");
        
        // 端口字段初始化
        for field in &s.fields {
            self.writeln(&format!("            {},", field.name));
        }
        
        // 属性字段初始化
        for prop in &s.properties {
            let init_value = match &prop.value {
                StruPropertyValue::Boolean(b) => b.to_string(),
                StruPropertyValue::Integer(i) => i.to_string(),
                StruPropertyValue::Float(f) => f.to_string(),
                StruPropertyValue::String(s) => format!("\"{}\".to_string()", s),
                StruPropertyValue::Duration(val, _) => val.to_string(),
                StruPropertyValue::Range(min, max, _) => format!("({}, {})", min, max),
            };
            self.writeln(&format!(
                "            {}: {}, // {}",
                prop.name.to_lowercase(),
                init_value,
                prop.docs[0].trim_start_matches("/// ")
            ));
        }
        
        self.writeln("        }");
        self.writeln("    }");
        self.writeln("}");
    }

    /// 根据属性值推断Rust类型
    fn type_for_property(&self, value: &StruPropertyValue) -> String {
        match value {
            StruPropertyValue::Boolean(_) => "bool".to_string(),
            StruPropertyValue::Integer(_) => "i64".to_string(),
            StruPropertyValue::Float(_) => "f64".to_string(),
            StruPropertyValue::String(_) => "String".to_string(),
            StruPropertyValue::Duration(_, _) => "u64".to_string(),
            StruPropertyValue::Range(_, _, _) => "(i64, i64)".to_string(),
        }
    }

    fn generate_field(&mut self, field: &Field) {
        for doc in &field.docs {
            self.writeln(doc);
        }
        for attr in &field.attrs {
            self.generate_attribute(attr);
        }
        self.writeln(&format!(
            "pub {}: {},",
            field.name,
            self.type_to_string(&field.ty)
        ));
    }

    fn generate_impl(&mut self, i: &ImplBlock) {
        self.write("impl");
        
        // 泛型参数
        if !i.generics.is_empty() {
            self.write("<");
            for (idx, generic) in i.generics.iter().enumerate() {
                if idx > 0 { self.write(", "); }
                self.write(&generic.name);
                if !generic.bounds.is_empty() {
                    self.write(": ");
                    for (j, bound) in generic.bounds.iter().enumerate() {
                        if j > 0 { self.write(" + "); }
                        self.write(bound);
                    }
                }
            }
            self.write(">");
        }
        
        // 目标类型
        self.write(&format!(" {} ", self.type_to_string(&i.target)));
        
        // trait实现
        if let Some(trait_ty) = &i.trait_impl {
            self.write(&format!("for {} ", self.type_to_string(trait_ty)));
        }
        
        self.writeln("{");
        self.indent();
        
        // 判断是否为进程实现
        let is_process = match &i.target {
            Type::Named(name) => name.ends_with("Process"),
            _ => false
        };
        
        // 特殊处理进程的new方法
        if is_process {
            self.generate_process_impl(i) ;
        }

        for item in &i.items {
            match item {
                ImplItem::Method(m) => self.generate_function(m),
                ImplItem::AssocConst(name, ty, expr) => {
                    self.writeln(&format!("const {}: {} = ", name, self.type_to_string(ty)));
                    self.generate_expr(expr);
                    self.writeln(";");
                }
                ImplItem::AssocType(name, ty) => {
                    self.writeln(&format!("type {} = {};", name, self.type_to_string(ty)));
                }
            }
        }
        
        self.dedent();
        self.writeln("}");
        self.writeln("");
    }

    fn generate_process_impl(&mut self, impl_block: &ImplBlock) {
        self.writeln(&format!("impl {} {{", self.type_to_string(&impl_block.target)));
        self.indent();

        // TODO:动态生成new方法
        if let Some(new_method) = impl_block.items.iter().find(|i| matches!(i, ImplItem::Method(f) if f.name == "new")) {
            if let ImplItem::Method(func) = new_method {
                self.generate_function(func);
            }
        }

        // 动态生成其他方法
        for item in &impl_block.items {
            if let ImplItem::Method(func) = item {
                if func.name != "new" {
                    self.generate_function(func);
                }
            }
        }

        self.dedent();
        self.writeln("}");
    }

    fn generate_function(&mut self, f: &FunctionDef) {
        // 文档注释
        for doc in &f.docs {
            self.writeln(doc);
        }
        
        // 属性
        for attr in &f.attrs {
            self.generate_attribute(attr);
        }
        
        // 函数签名
        self.write(&format!(
            "{} {}fn {}",
            self.visibility(&f.vis),
            if f.asyncness { "async " } else { "" },
            f.name
        ));
        
        // 参数
        self.write("(");
        for (i, param) in f.params.iter().enumerate() {
            if i > 0 { self.write(", "); }
            self.write(&format!("{}: {}", param.name, self.type_to_string(&param.ty)));
        }
        self.write(")");
        
        // 返回类型
        self.write(&format!(" -> {}", self.type_to_string(&f.return_type)));
        
        // 函数体
        self.writeln(" {");
        self.indent();
        self.generate_block(&f.body);
        self.dedent();
        self.writeln("}");
        self.writeln("");
    }

    fn generate_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.generate_statement(stmt);
        }
        
        if let Some(expr) = &block.expr {
            self.generate_expr(expr);
            self.writeln(";");
        }
    }

    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let(ls) => {
                self.write(&format!("let {}", ls.name));
                if let Some(ty) = &ls.ty {
                    self.write(&format!(": {}", self.type_to_string(ty)));
                }
                if let Some(init) = &ls.init {
                    self.write(" = ");
                    self.generate_expr(init);
                }
                self.writeln(";");
            }
            Statement::Expr(expr) => {
                // 处理连接建立的表达式
                if let Expr::MethodCall(receiver, method, args) = expr {
                    if method == "send" || method == "receive" {
                        self.writeln("/// bulid connection: ");
                        self.write("    ");
                        self.generate_expr(receiver);
                        self.write(" = ");

                        for (i, arg) in args.iter().enumerate() {
                            if i > 0 { self.write(", "); }
                            self.generate_expr(arg);
                        }
                        self.writeln(";");

                        return;
                    }
                }
                // 普通表达式处理
                self.generate_expr(expr);
                self.writeln(";");
            }
            Statement::Item(item) => self.generate_item(item),
        }
    }

    fn generate_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(id) => self.write(id),
            Expr::Path(path) => {
                for (i, part) in path.iter().enumerate() {
                    if i > 0 { self.write("::"); }
                    self.write(part);
                }
            }
            Expr::Literal(lit) => self.generate_literal(lit),
            Expr::Call(callee, args) => {
                self.generate_expr(callee);
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.generate_expr(arg);
                }
                self.write(")");
            }
            Expr::MethodCall(receiver, method, args) => {
                self.generate_expr(receiver);
                self.write(&format!(".{}(", method));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.generate_expr(arg);
                }
                self.write(")");
            }
            Expr::Block(block) => {
                self.writeln("{");
                self.indent();
                self.generate_block(block);
                self.dedent();
                self.write("}");
            }
            Expr::Loop(block) => {
                self.write("loop ");
                self.generate_block(block);
            }
            Expr::Await(expr) => {
                self.generate_expr(expr);
                self.write(".await");
            }
            Expr::Closure(params, body) => {
                self.write("|");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.write(param);
                }
                self.write("| ");
                self.generate_expr(body);
            }
        }
    }

    fn generate_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Int(i) => self.write(&i.to_string()),
            Literal::Float(f) => self.write(&f.to_string()),
            Literal::Str(s) => self.write(&format!("\"{}\"", s)),
            Literal::Bool(b) => self.write(&b.to_string()),
            Literal::Char(c) => self.write(&format!("'{}'", c)),
        }
    }

    fn generate_type_alias(&mut self, t: &TypeAlias) {
        for doc in &t.docs {
            self.writeln(doc);
        }
        self.writeln(&format!(
            "{}type {} = {};",
            self.visibility(&t.vis),
            t.name,
            self.type_to_string(&t.target)
        ));
        self.writeln("");
    }

    fn generate_enum(&mut self, e: &EnumDef) {
        for doc in &e.docs {
            self.writeln(doc);
        }
        
        if !e.derives.is_empty() {
            self.write("#[derive(");
            for (i, derive) in e.derives.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(derive);
            }
            self.writeln(")]");
        }
        
        self.write(&format!("{}enum {} ", self.visibility(&e.vis), e.name));
        
        if e.generics.is_empty() {
            self.writeln("{");
        } else {
            self.write("<");
            for (i, generic) in e.generics.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(&generic.name);
                if !generic.bounds.is_empty() {
                    self.write(": ");
                    for (j, bound) in generic.bounds.iter().enumerate() {
                        if j > 0 { self.write(" + "); }
                        self.write(bound);
                    }
                }
            }
            self.writeln("> {");
        }
        
        self.indent();
        for variant in &e.variants {
            for doc in &variant.docs {
                self.writeln(doc);
            }
            self.write(&variant.name);
            if let Some(types) = &variant.data {
                self.write("(");
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.write(&self.type_to_string(ty));
                }
                self.write(")");
            }
            self.writeln(",");
        }
        self.dedent();
        self.writeln("}");
        self.writeln("");
    }

    fn generate_const(&mut self, c: &ConstDef) {
        for doc in &c.docs {
            self.writeln(doc);
        }
        self.write(&format!(
            "{}const {}: {} = ",
            self.visibility(&c.vis),
            c.name,
            self.type_to_string(&c.ty)
        ));
        self.generate_expr(&c.value);
        self.writeln(";");
        self.writeln("");
    }

    fn generate_use(&mut self, u: &UseStatement) {
        self.write("use ");
        for (i, part) in u.path.iter().enumerate() {
            if i > 0 { self.write("::"); }
            self.write(part);
        }
        
        match &u.kind {
            UseKind::Simple => self.writeln(";"),
            UseKind::Glob => self.writeln("::*;"),
            UseKind::Nested => self.writeln("::{*};")
        }
    }

    fn generate_attribute(&mut self, attr: &Attribute) {
        self.write(&format!("#[{}", attr.name));
        if !attr.args.is_empty() {
            self.write("(");
            for (i, arg) in attr.args.iter().enumerate() {
                if i > 0 { self.write(", "); }
                match arg {
                    AttributeArg::Ident(id) => self.write(id),
                    AttributeArg::Literal(lit) => self.generate_literal(lit),
                    AttributeArg::KeyValue(k, v) => {
                        self.write(&format!("{} = ", k));
                        self.generate_literal(v);
                    }
                }
            }
            self.write(")");
        }
        self.writeln("]");
    }

    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::Path(path) => path.join("::"),
            Type::Named(name) => name.clone(),
            Type::Generic(name, params) => {
                let mut s = name.clone();
                s.push('<');
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { s.push_str(", "); }
                    s.push_str(&self.type_to_string(param));
                }
                s.push('>');
                s
            }
            Type::Reference(inner, mutable) => {
                format!("&{} {}", if *mutable { "mut " } else { "" }, self.type_to_string(inner))
            }
            Type::Tuple(types) => {
                let mut s = "(".to_string();
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 { s.push_str(", "); }
                    s.push_str(&self.type_to_string(ty));
                }
                if types.len() == 1 { s.push(','); }
                s.push(')');
                s
            }
            Type::Slice(inner) => format!("[{}]", self.type_to_string(inner)),
            Type::Unit => "()".to_string(),
            Type::Never => "!".to_string(),
        }
    }

    fn visibility(&self, vis: &Visibility) -> String {
        match vis {
            Visibility::Public => "pub ".to_string(),
            Visibility::Private => "".to_string(),
            Visibility::Restricted(path) => format!("pub(in {}) ", path.join("::")),
        }
    }

    // 辅助方法
    fn writeln(&mut self, s: &str) {
        self.write(s);
        self.buffer.push('\n');
    }

    fn write(&mut self, s: &str) {
        if self.buffer.ends_with('\n') || self.buffer.is_empty() {
            self.buffer.push_str(&"    ".repeat(self.indent_level));
        }
        self.buffer.push_str(s);
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}
