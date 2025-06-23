// src/ir/intermediate_ast.rs
use std::collections::HashMap;

/// 轻量级Rust抽象语法树（模块级）
#[derive(Debug, Default)]
pub struct RustModule {
    pub name: String,
    pub docs: Vec<String>,
    pub items: Vec<Item>,
    pub attrs: Vec<Attribute>, // #[attributes]
}

/// 模块项定义
#[derive(Debug)]
pub enum Item {
    Struct(StructDef),
    Enum(EnumDef),
    Function(FunctionDef),
    Impl(ImplBlock),
    Const(ConstDef),
    TypeAlias(TypeAlias),
    Use(UseStatement),
    Mod(Box<RustModule>), // 嵌套模块
}

/// 结构体定义
#[derive(Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>, //(对应aadl端口)
    pub properties: Vec<StruProperty>, //存储属性
    pub generics: Vec<GenericParam>,
    pub derives: Vec<String>, // #[derive(...)]
    pub docs: Vec<String>,
    pub vis: Visibility, //控制结构体的可见性
}

/// 枚举定义
#[derive(Debug)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<Variant>,
    pub generics: Vec<GenericParam>,
    pub derives: Vec<String>,
    pub docs: Vec<String>,
    pub vis: Visibility,
}

/// 函数定义
#[derive(Debug)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Block,
    pub asyncness: bool,
    pub vis: Visibility,
    pub docs: Vec<String>,
    pub attrs: Vec<Attribute>,
}

/// 实现块
#[derive(Debug)]
pub struct ImplBlock {
    pub target: Type,
    pub generics: Vec<GenericParam>,
    pub items: Vec<ImplItem>,
    pub trait_impl: Option<Type>, // 为哪个trait实现
}

/// 常量定义
#[derive(Debug)]
pub struct ConstDef {
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub vis: Visibility,
    pub docs: Vec<String>,
}

/// 类型别名
#[derive(Debug)]
pub struct TypeAlias {
    pub name: String,
    pub target: Type,
    pub vis: Visibility,
    pub docs: Vec<String>,
}

#[derive(Debug)]
pub struct StruProperty {
    pub name: String,
    pub value: StruPropertyValue,
    pub docs: Vec<String>, // 属性文档
}
// ========== 基础类型定义 ========== //

/// 类型表示
#[derive(Debug, Clone)]
pub enum Type {
    Path(Vec<String>),           // std::vec::Vec
    Named(String),               // i32, String
    Generic(String, Vec<Type>),  // HashMap<K, V>
    Reference(Box<Type>, bool),  // &mut T
    Tuple(Vec<Type>),            // (T1, T2)
    Slice(Box<Type>),            // [T]
    Unit,                        // ()
    Never,                       // !
}

#[derive(Debug,Clone)]
pub enum PathType {
    Namespace,  // 用 :: 分隔 (如 std::thread)
    Member,     // 用 . 分隔 (如 self.field)
}

/// 表达式
#[derive(Debug)]
pub enum Expr {
    Ident(String),
    Path(Vec<String>,PathType),
    Literal(Literal),
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    Block(Block),
    Loop(Box<Block>),
    Await(Box<Expr>),
    Closure(Vec<String>, Box<Expr>),
    BuilderChain(Vec<BuilderMethod>), // 新增：表示(进程在创建线程时)构建器链式调用
}

//区分.name()、.stack_size()等不同构建器方法
#[derive(Debug)]
pub enum BuilderMethod {
    Named(String), // 如.name("thread_name")
    StackSize(Box<Expr>), // 如.stack_size(expr)
    Spawn {
        closure: Box<Expr>,
        move_kw: bool, // 将move语义放在BuilderMethod中
    },
}


/// 字面量
#[derive(Debug)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Char(char),
}


#[derive(Debug)]
pub enum StruPropertyValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Duration(u64, String),      // (值, 单位)
    Range(i64, i64, Option<String>), // (最小值, 最大值, 单位)
}

/// 代码块
#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Statement>,
    pub expr: Option<Box<Expr>>,
}

/// 语句
#[derive(Debug)]
pub enum Statement {
    Let(LetStmt),
    Expr(Expr),
    Item(Box<Item>),
}

/// let绑定
#[derive(Debug)]
pub struct LetStmt {
    pub name: String,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
}

// ========== 辅助类型 ========== //

#[derive(Debug)]
pub enum Visibility {
    Public,
    Private,
    Restricted(Vec<String>), // pub(in path)
}

#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<AttributeArg>,
}

#[derive(Debug)]
pub enum AttributeArg {
    Ident(String),
    Literal(Literal),
    KeyValue(String, Literal),
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub docs: Vec<String>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug)]
pub struct Variant {
    pub name: String,
    pub data: Option<Vec<Type>>, // Some for tuple variant
    pub docs: Vec<String>,
}

#[derive(Debug)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<String>, // trait bounds
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub enum ImplItem {
    Method(FunctionDef),
    AssocConst(String, Type, Expr),
    AssocType(String, Type),
}

#[derive(Debug)]
pub struct UseStatement {
    pub path: Vec<String>,
    pub kind: UseKind,
}

#[derive(Debug)]
pub enum UseKind {
    Simple,
    Glob,    // {path}::*
    Nested,  // {path}::{a, b}
}