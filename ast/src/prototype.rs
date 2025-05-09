use crate::ast::*;
use crate::subcomponent::*;
/* ========== 4.7 Prototype ========== */
/* ========== 基础原型定义 ========== */
#[derive(Debug, Clone)]
pub enum Prototype {
    Component(ComponentPrototype),
    FeatureGroup(FeatureGroupPrototype),
    Feature(FeaturePrototype),
}

#[derive(Debug, Clone)]
pub struct PrototypeDeclaration {
    pub identifier: String,
    pub prototype: Prototype,
    pub properties: Vec<PrototypePropertyAssociation>,
}

/* ========== 组件原型 ========== */
#[derive(Debug, Clone)]
pub struct ComponentPrototype {
    pub category: ComponentCategory,
    pub classifier: Option<UniqueComponentClassifierReference>,
    pub is_array: bool,  // 对应 [ [] ] 语法
}
/* ========== 特性组原型 ========== */
#[derive(Debug, Clone)]
pub struct FeatureGroupPrototype {
    pub classifier: Option<UniqueFeatureGroupTypeReference>,
}
/* ========== 特性原型 ========== */
#[derive(Debug, Clone)]
pub struct FeaturePrototype {
    pub direction: Option<Direction>,  // in/out
    pub classifier: Option<UniqueComponentClassifierReference>,
}