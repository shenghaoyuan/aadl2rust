/* ========== 4.2 Package ========== */
// 包名（双冒号分隔的标识符序列）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageName(pub Vec<String>);

//双冒号分隔的包名
impl PackageName {
    pub fn to_string(&self) -> String {
        self.0.join("::")
    }
}

// 包可见性声明（with/renames）
#[derive(Debug, Clone)]
pub enum VisibilityDeclaration {
    // with package1, package2, property_set;
    Import {
        packages: Vec<PackageName>,
        property_sets: Vec<String>,
    },
    // renames package::component;
    Alias {
        new_name: String,
        original: QualifiedName,
        is_package: bool, // 区分包别名和组件别名
    },
    // renames package::all;
    ImportAll(PackageName),
}

// 可能带命名空间前缀的引用
#[derive(Debug, Clone)]
pub struct QualifiedName {
    pub package_prefix: Option<PackageName>,
    pub identifier: String,
}

// 包声明部分（公共/私有）
#[derive(Debug, Clone)]
pub struct PackageSection {
    pub is_public: bool,
    pub declarations: Vec<AadlDeclaration>,
}

// 包属性声明
#[derive(Debug, Clone)]
pub enum PropertyClause {
    ExplicitNone,  // none;
    Properties(Vec<Property>),
}

// 完整包定义
#[derive(Debug, Clone)]
pub struct Package {
    pub name: PackageName,
    pub visibility_decls: Vec<VisibilityDeclaration>,
    pub public_section: Option<PackageSection>,
    pub private_section: Option<PackageSection>,
    pub properties: PropertyClause,
}

#[derive(Debug, Clone)]
pub enum AadlDeclaration {
    ComponentType(ComponentType),
    ComponentTypeExtension(ComponentTypeExtension),
    ComponentImplementation(ComponentImplementation),
    ComponentImplementationExtension(ComponentImplementationExtension),
    AnnexLibrary(AnnexLibrary)
}

/* ========== 4.3 Component Types ========== */
// 组件类型定义 
#[derive(Debug, Clone)]
pub struct ComponentType {
    pub category: ComponentCategory,
    pub identifier: String,
    pub prototypes: PrototypeClause,
    pub features: FeatureClause,
    pub flows: FlowClause,
    pub modes: Option<ModesClause>,
    pub properties: PropertyClause,
    pub annexes: Vec<AnnexSubclause>
}

#[derive(Debug, Clone)]
pub struct AnnexLibrary{

}
#[derive(Debug, Clone)]
pub enum  AnnexSubclause{

}



// 组件类型的可选子句（None表示子句不存在，Empty表示显式声明none）
//cj:非“关键字可选，使用Option”
#[derive(Debug, Clone)]
pub enum PrototypeClause {
    None,          // 无prototypes子句
    Empty,         // prototypes none;
    Items(Vec<Prototype>)
}

#[derive(Debug, Clone)]
pub enum FeatureClause {
    None,
    Empty,
    Items(Vec<Feature>)
}

#[derive(Debug, Clone)]
pub enum FlowClause {
    None,
    Empty,
    Items(Vec<FlowSpec>)
}

//组件类型扩展 
#[derive(Debug, Clone)]
pub struct ComponentTypeExtension {
    pub category: ComponentCategory,
    pub identifier: String,
    pub extends: UniqueComponentReference,
    pub prototype_bindings: Option<PrototypeBindings>,
    pub prototypes: PrototypeClause,
    pub features: FeatureClause,
    pub flows: FlowClause,
    pub modes: Option<ModesClause>,
    pub properties: PropertyClause,
    pub annexes: Vec<AnnexSubclause>
}
// 相关子结构
#[derive(Debug, Clone)]
pub struct Prototype {
    pub identifier: String,
    pub classifier: ClassifierReference
}

#[derive(Debug, Clone)]
pub struct Feature {
    pub identifier: String,
    pub category: FeatureCategory,
    pub direction: Option<Direction>,
    pub data_type: Option<DataTypeReference>
}

#[derive(Debug, Clone)]
pub struct FlowSpec {
    pub identifier: String,
    pub source: Option<FlowEndpoint>,
    pub sink: Option<FlowEndpoint>
}

#[derive(Debug, Clone)]
pub enum ModesClause {
    Modes(Vec<Mode>),
    RequiresModes
}

//基础类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentCategory {
    // 抽象组件类别
    Abstract,
    // 软件类别
    Data,
    Subprogram,
    SubprogramGroup,
    Thread,
    ThreadGroup,
    Process,
    // 执行平台类别
    Memory,
    Processor,
    Bus,
    Device,
    VirtualProcessor,
    VirtualBus,
    // 复合类别
    System
}

#[derive(Debug, Clone)]
pub struct UniqueComponentReference {
    pub package_prefix: Option<PackageName>,
    pub identifier: String,
}

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
pub struct SubcomponentRefinement {
    pub original_name: String,
    pub refinement: Subcomponent
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
