use crate::ast::*;

/* ========== 4.4 Component Implementations ========== */
/* ========== 组件实现 ========== */
#[derive(Debug, Clone)]
pub struct ComponentImplementation {
    pub category: ComponentCategory,
    pub name: ImplementationName,
    pub prototype_bindings: Option<PrototypeBindings>,
    pub prototypes: PrototypeClause,
    pub subcomponents: SubcomponentClause,
    pub internal_features: Vec<InternalFeature>,
    pub processor_features: Vec<ProcessorFeature>,
    pub calls: CallSequenceClause,
    pub connections: ConnectionClause,
    pub flows: FlowImplementationClause,
    pub modes: Option<ModesClause>,
    pub properties: PropertyClause,
    pub annexes: Vec<AnnexSubclause>
}

// 组件实现名称（type_id.impl_id）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImplementationName {
    pub type_identifier: String,
    pub implementation_identifier: String
}

impl ImplementationName {
    pub fn to_string(&self) -> String {
        format!("{}.{}", self.type_identifier, self.implementation_identifier)
    }
}

// 组件实现扩展
#[derive(Debug, Clone)]
pub struct ComponentImplementationExtension {
    pub category: ComponentCategory,
    pub name: ImplementationName,
    
    // 被扩展的实现引用
    pub extends: UniqueImplementationReference,
    
    pub prototype_bindings: Option<PrototypeBindings>,
    pub prototypes: PrototypeClause,
    pub subcomponents: SubcomponentClause,
    pub internal_features: Vec<InternalFeature>,
    pub processor_features: Vec<ProcessorFeature>,
    pub calls: CallSequenceClause,
    pub connections: ConnectionClause,
    pub flows: FlowImplementationClause,
    pub modes: Option<ModesClause>,
    pub properties: PropertyClause,
    pub annexes: Vec<AnnexSubclause>
}

// 唯一的组件实现引用（可能带包前缀）
#[derive(Debug, Clone)]
pub struct UniqueImplementationReference {
    pub package_prefix: Option<PackageName>,
    pub implementation_name: ImplementationName
}

// 子句类型定义
#[derive(Debug, Clone)]
pub enum SubcomponentClause {
    None,
    Empty,
    Items(Vec<Subcomponent>),
    Refinements(Vec<SubcomponentRefinement>)
}

#[derive(Debug, Clone)]
pub enum CallSequenceClause {
    None,
    Empty,
    Items(Vec<CallSequence>)
}

#[derive(Debug, Clone)]
pub enum ConnectionClause {
    None,
    Empty,
    Items(Vec<Connection>),
    Refinements(Vec<ConnectionRefinement>)
}

#[derive(Debug, Clone)]
pub enum FlowImplementationClause {
    None,
    Empty,
    Items(Vec<FlowImplementation>),
    EndToEndFlows(Vec<EndToEndFlow>),
    Refinements(Vec<FlowRefinement>)
}

#[derive(Debug, Clone)]
pub struct ConnectionRefinement {
    pub original_name: String,
    pub refinement: Connection
}

#[derive(Debug, Clone)]
pub struct FlowRefinement {
    pub original_name: String,
    pub refinement: FlowImplementation
}
