use crate::ast::*;
use crate::compimplement::*;
use crate::prototype::*;
/* ========== 4.5 subComponent ========== */
#[derive(Debug, Clone)]
pub struct Subcomponent {
    pub identifier: String,
    pub category: ComponentCategory,
    pub classifier: SubcomponentClassifier,
    pub array_spec: Option<ArraySpec>,
    pub properties: Vec<SubcomponentProperty>,
    pub modes: Option<ComponentInModes>
}
#[derive(Debug, Clone)]
pub enum SubcomponentClassifier {
    /// 组件分类器引用
    ClassifierReference(UniqueComponentClassifierReference),
    /// 原型引用
    Prototype(String)
}
/// 唯一的组件分类器引用
#[derive(Debug, Clone)]
pub enum UniqueComponentClassifierReference {
    Type(UniqueImplementationReference),
    Implementation(UniqueImplementationReference)
}
/* ========== 子组件精化 ========== */
#[derive(Debug, Clone)]
pub struct SubcomponentRefinement {
    pub identifier: String,
    pub category: ComponentCategory,
    pub classifier: Option<SubcomponentClassifier>, // refined to可能省略引用
    pub array_spec: Option<ArraySpec>,
    pub properties: Vec<SubcomponentProperty>,
    pub modes: Option<ComponentInModes>
}
/* ========== 数组维度定义 ========== */
#[derive(Debug, Clone)]
pub struct ArraySpec {
    pub dimensions: Vec<ArrayDimension>,
    pub element_implementations: Option<Vec<ArrayElementImplementation>>
}
#[derive(Debug, Clone)]
pub struct ArrayDimension {
    pub size: Option<ArrayDimensionSize> // 可选尺寸表示 [ ]
}

#[derive(Debug, Clone)]
pub enum ArrayDimensionSize {
    Fixed(u32),
    PropertyReference(String) // 属性常量标识符
}

#[derive(Debug, Clone)]
pub struct ArrayElementImplementation {
    pub implementation: UniqueImplementationReference,
    pub prototype_bindings: Option<PrototypeBindings>
}
/* ========== TODO 属性关联 ========== */
#[derive(Debug, Clone)]
pub enum SubcomponentProperty {
    Direct(PropertyAssociation),
    Contained(ContainedPropertyAssociation)
}