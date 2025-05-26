#[allow(dead_code)]
pub mod aadl_ast_cj{

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
    // with package1, package2, property_set;对应 `with` 语法
    Import {
        packages: Vec<PackageName>,
        property_sets: Vec<String>,
    },
    // renames package::component;对应 `renames` 语法
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
    pub visibility_decls: Vec<VisibilityDeclaration>, //声明当前包与其他包或属性集之间的依赖关系
    pub public_section: Option<PackageSection>,
    pub private_section: Option<PackageSection>,
    pub properties: PropertyClause, //暂时例子中，为空
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
    pub prototypes: PrototypeClause,  //“原型”，暂没见过
    pub features: FeatureClause,
    //pub flows: FlowClause,
    //pub modes: Option<ModesClause>,
    pub properties: PropertyClause,
    pub annexes: Vec<AnnexSubclause>
}

//sTODO 4.8 Annex Subclauses and Annex Libraries
#[derive(Debug, Clone)]
pub struct AnnexLibrary{

}
#[derive(Debug, Clone)]
pub enum  AnnexSubclause{

}
//eTODO


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

// #[derive(Debug, Clone)]
// pub enum FlowClause {
//     None,
//     Empty,
//     Items(Vec<FlowSpec>)
// }

//组件类型扩展 
#[derive(Debug, Clone)]
pub struct ComponentTypeExtension {
    pub category: ComponentCategory,
    pub identifier: String,
    pub extends: UniqueComponentReference,
    pub prototype_bindings: Option<PrototypeBindings>,
    pub prototypes: PrototypeClause,
    pub features: FeatureClause,
    //pub flows: FlowClause,
    //pub modes: Option<ModesClause>,
    pub properties: PropertyClause,
    pub annexes: Vec<AnnexSubclause>
}


// #[derive(Debug, Clone)]
// pub struct FlowSpec {
//     pub identifier: String,
//     pub source: Option<FlowEndpoint>,
//     pub sink: Option<FlowEndpoint>
// }

// #[derive(Debug, Clone)]
// pub enum ModesClause {
//     Modes(Vec<Mode>),
//     RequiresModes
// }

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
    //pub internal_features: Vec<InternalFeature>,
    //pub processor_features: Vec<ProcessorFeature>,
    pub calls: CallSequenceClause,
    pub connections: ConnectionClause,
    //pub flows: FlowImplementationClause,
    //pub modes: Option<ModesClause>,
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
    //pub internal_features: Vec<InternalFeature>,
    //pub processor_features: Vec<ProcessorFeature>,
    pub calls: CallSequenceClause,
    pub connections: ConnectionClause,
    //pub flows: FlowImplementationClause,
    //pub modes: Option<ModesClause>,
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

// #[derive(Debug, Clone)]
// pub enum FlowImplementationClause {
//     None,
//     Empty,
//     Items(Vec<FlowImplementation>),
//     EndToEndFlows(Vec<EndToEndFlow>),
//     Refinements(Vec<FlowRefinement>)
// }

#[derive(Debug, Clone)]
pub struct ConnectionRefinement {
    pub original_name: String,
    pub refinement: Connection
}

// #[derive(Debug, Clone)]
// pub struct FlowRefinement {
//     pub original_name: String,
//     pub refinement: FlowImplementation
// }


/* ========== 4.5 subComponent ========== */
#[derive(Debug, Clone)]
pub struct Subcomponent {
    pub identifier: String,
    pub category: ComponentCategory,
    pub classifier: SubcomponentClassifier,
    pub array_spec: Option<ArraySpec>,
    pub properties: Vec<Property>,
    //pub modes: Option<ComponentInModes>
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
    pub properties: Vec<Property>,
    //pub modes: Option<ComponentInModes>
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
// 对应标准中的 `unique_feature_group_type_reference`
#[derive(Debug, Clone)]
pub struct UniqueFeatureGroupTypeReference {
    /// 可选的包名前缀 `[ package_name :: ]`
    pub package_prefix: Option<PackageName>,
    
    /// 特性组类型标识符 `feature_group_type_identifier`
    pub identifier: String,
}
/* ========== 特性原型 ========== */
#[derive(Debug, Clone)]
pub struct FeaturePrototype {
    pub direction: Option<PortDirection>,  // in/out
    pub classifier: Option<UniqueComponentClassifierReference>,
}

/* ========== 原型精化 ========== */
#[derive(Debug, Clone)]
pub struct PrototypeRefinement {
    pub identifier: String,
    pub prototype: Prototype,  // 精化后的目标原型
    pub properties: Vec<PrototypePropertyAssociation>,
}

/* ========== 原型绑定 ========== */
#[derive(Debug, Clone)]
pub struct PrototypeBindings {
    pub bindings: Vec<PrototypeBinding>,
}

#[derive(Debug, Clone)]
pub struct PrototypeBinding {
    pub identifier: String,
    pub actual: PrototypeActual,
}

#[derive(Debug, Clone)]
pub enum PrototypeActual {
    Component(ComponentPrototypeActual),
    ComponentList(Vec<ComponentPrototypeActual>),
    FeatureGroup(FeatureGroupPrototypeActual),
    Feature(FeaturePrototypeActual),
}

/* ========== 组件原型实际值 ========== */
#[derive(Debug, Clone)]
pub struct ComponentPrototypeActual {
    pub category: ComponentCategory,
    pub reference: Option<ComponentPrototypeReference>,
    pub bindings: Option<PrototypeBindings>,
}

#[derive(Debug, Clone)]
pub enum ComponentPrototypeReference {
    Classifier(UniqueComponentClassifierReference),
    Prototype(String),  // 引用其他原型
}

/* ========== 特性组原型实际值 ========== */
#[derive(Debug, Clone)]
pub enum FeatureGroupPrototypeActual {
    Classifier {
        reference: UniqueFeatureGroupTypeReference,
        bindings: Option<PrototypeBindings>,
    },
    Prototype(String),  // 引用其他特性组原型
}

/* ========== 特性原型实际值 ========== */
#[derive(Debug, Clone)]
pub enum FeaturePrototypeActual {
    Port {
        direction: PortDirection,
        port_type: PortType,
        classifier: Option<UniqueComponentClassifierReference>,
    },
    Access {
        access_type: AccessDirection,
        connection_type: AccessConnectionType,
        classifier: Option<UniqueComponentClassifierReference>,
    },
    Prototype(String),  // 引用其他特性原型
}

/* ========== 相关枚举类型 ========== */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortDirection {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessDirection {
    Requires,
    Provides,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessConnectionType {
    Bus,
    VirtualBus,
    Data,
    SubprogramGroup,
    Subprogram,
}

/* ========== TODO:属性关联 ========== */
#[derive(Debug, Clone)]
pub struct PrototypePropertyAssociation {
    pub name: String,
    pub value: PropertyValue,
    pub applies_to: Option<Vec<String>>,
}


/*================software component====================*/

/*==========5.1 Data 没有syntax========= */

/*==========5.2 Subprograms and Subprogram Calls============= */
#[derive(Debug, Clone)]
pub struct CallSequence {
    /// 调用序列标识符 (defining_call_sequence_identifier)
    pub identifier: String,
    
    /// 子程序调用列表 (subprogram_call+)
    pub calls: Vec<SubprogramCall>,
    
    /// 调用序列属性 (call_sequence_property_association*)
    pub properties: Vec<Property>,
    
    /// 模式约束 (in_modes)
    pub in_modes: Option<Vec<String>>,
}

/* ========== 子程序调用 ========== */
#[derive(Debug, Clone)]
pub struct SubprogramCall {
    /// 调用标识符 (defining_call_identifier)
    pub identifier: String,
    
    /// 被调用的子程序 (called_subprogram)
    pub called: CalledSubprogram,
    
    /// 调用属性 (subcomponent_call_property_association*)
    pub properties: Vec<Property>,
}

/* ========== 被调用的子程序 ========== */
//TODO:目前只定义了一种引用方式，其它的引用方式未见过案例
#[derive(Debug, Clone)]
pub enum CalledSubprogram {
    /// 通过分类器引用 (subprogram_unique_component_classifier_reference)
    Classifier(UniqueComponentClassifierReference),
}


/*8 features and shared access */
//功能是组件类型定义的一部分，指定接口
//TODO:目前只考虑port,例子在Notion中有图片
#[derive(Debug, Clone)]
pub enum Feature {
    // 抽象特征 (abstract_feature_spec)
    //Abstract(AbstractFeature),
    
    // 端口 (port_spec)
    Port(PortSpec),
    
    // 特征组 (feature_group_spec)
    //FeatureGroup(FeatureGroupSpec),
    
    // 子组件访问 (subcomponent_access_spec)
    //SubcomponentAccess(SubcomponentAccess),
    
    // 参数 (parameter_spec)
    //Parameter(ParameterSpec),
    
    // 精化特征 (feature_refinement)
    //Refinement(FeatureRefinement)
}
/* ========== 端口类型 ========== */
/// 对应标准中的 `port_type`
#[derive(Debug, Clone)]
pub enum PortType {
    /// `data port [reference]`
    Data {
        classifier: Option<PortDataTypeReference>,
    },
    /// `event data port [reference]`
    EventData {
        classifier: Option<PortDataTypeReference>,
    },
    /// `event port`
    Event,
}

/// 端口数据类型引用（对应标准中的两种引用方式）
#[derive(Debug, Clone)]
pub enum PortDataTypeReference {
    /// `data_unique_component_classifier_reference`
    Classifier(UniqueComponentClassifierReference),
    /// `data_component_prototype_identifier`
    Prototype(String),
}
#[derive(Debug, Clone)]
pub struct PortSpec {
    /// `defining_port_identifier`
    pub identifier: String,
    pub direction: PortDirection,
    pub port_type: PortType,
}

/*=================9 connection ============ */
/* ========== 连接类型 ========== */
#[derive(Debug, Clone)]
pub enum Connection {
    // 端口连接 (port_connection)
    Port(PortConnection),
    
    // 参数连接 (parameter_connection)
    Parameter(ParameterConnection),
    
    // 以下为其他连接类型（暂不实现）
    // Feature(FeatureConnection),      // feature_connection
    // Access(AccessConnection),       // access_connection
    // FeatureGroup(FeatureGroupConnection), // feature_group_connection
}

/* ========== 端口连接符号 ========== */
/// 对应标准中的 `connection_symbol`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionSymbol {
    ///  ->
    Direct,
    ///  <-> 
    Didirect,
}
/* ========== port 口连接定义 ========== */
/// 对应标准中的 `port_connection`
#[derive(Debug, Clone)]
pub struct PortConnection {
    pub source: PortEndpoint,
    pub destination: PortEndpoint,
    pub connection_direction: ConnectionSymbol,
}
// 对应标准中的 `port_connection_reference`
#[derive(Debug, Clone)]
pub enum PortEndpoint {
    /// 组件类型端口 (component_type_port_identifier)
    ComponentPort(String),
    
    /// 子组件端口 (subcomponent_identifier.port_identifier)
    SubcomponentPort {
        subcomponent: String,
        port: String,
    },
    
    /// 特征组元素端口 (component_type_feature_group_identifier.element_port_identifier)
    FeatureGroupPort {
        feature_group: String,
        element: String,
    },
    
    /// 聚合数据端口元素 (component_type_port_identifier.data_subcomponent_identifier)
    AggregateDataElement {
        port: String,
        data_element: String,
    },
    
    /// 数据访问要求 (component_type_requires_data_access_identifier)
    RequiresDataAccess(String),
    
    /// 数据子组件 (data_subcomponent_identifier)
    DataSubcomponent(String),
    
    /// 子组件提供的数据访问 (subcomponent_identifier.provides_data_access_identifier)
    SubcomponentDataAccess {
        subcomponent: String,
        access: String,
    },
    
    /// 特征组数据访问元素 (component_type_feature_group_identifier.element_data_access_identifier)
    FeatureGroupDataAccess {
        feature_group: String,
        element: String,
    },
    
    /// 数据子组件嵌套访问 (data_subcomponent_identifier.data_subcomponent_identifier)
    NestedDataAccess {
        container: String,
        element: String,
    },
    
    /// 处理器端口 ([processor.]processor_port_identifier)
    ProcessorPort {
        processor: Option<String>, // None表示隐式当前处理器
        port: String,
    },
    
    /// 组件内部事件源 ([self.]internal_event_or_event_data_identifier)
    InternalEvent {
        self_ref: bool, // 是否显式使用"self."
        identifier: String,
    },
}
/* ========== 参数连接定义 ========== */
/// 对应标准中的 `parameter_connection`
#[derive(Debug, Clone)]
pub struct ParameterConnection {
    pub source: ParameterEndpoint,
    pub destination: ParameterEndpoint,
    pub connection_direction: ConnectionSymbol,
}
/* ========== 参数端点定义 ========== */
/// 对应标准中的 `parameter_reference`
#[derive(Debug, Clone)]
pub enum ParameterEndpoint {
    /// 线程/子程序类型参数 (component_type_parameter_identifier[.data_subcomponent_identifier])
    ComponentParameter {
        parameter: String,
        data_subcomponent: Option<String>, // 可选数据子组件
    },
    
    /// 子程序调用参数 (subprogram_call_identifier.parameter_identifier)
    SubprogramCallParameter {
        call_identifier: String,
        parameter: String,
    },
    
    /// 线程类型的数据/事件数据端口 (component_type_port_identifier[.data_subcomponent_identifier])
    ThreadPort {
        port: String,
        data_subcomponent: Option<String>, // 可选数据元素
    },
    
    /// 数据子组件 (data_subcomponent_identifier)
    DataSubcomponent(String),
    
    /// 要求的数据访问 (requires_data_access_identifier)
    RequiresDataAccess(String),
    
    /// 特征组的数据访问元素 (component_type_feature_group_identifier.element_data_access_identifier)
    FeatureGroupDataAccess {
        feature_group: String,
        element: String,
    },
    
    /// 特征组的端口/参数元素 (component_type_feature_group_identifier.element_port_or_parameter_identifier)
    FeatureGroupElement {
        feature_group: String,
        element: String,
    },
}

/*==============11 属性=============== */
#[derive(Debug, Clone)]
pub enum Property {
    /// 基础属性关联 (basic_property_association)
    BasicProperty(BasicPropertyAssociation),
    SubcomponentProperty(BasicPropertyAssociation),//TODO:暂时使用basic代替
    CallSequenceProperty(BasicPropertyAssociation),
    // 未来可扩展其他属性类型：
}


/* ========== 基础属性关联 ========== */
#[derive(Debug, Clone)]
pub struct BasicPropertyAssociation {
    /// 属性标识符 (unique_property_identifier)
    pub identifier: PropertyIdentifier,
    
    /// 赋值操作符 => 或 +=>
    pub operator: PropertyOperator,
    
    /// 是否为常量 [constant]
    pub is_constant: bool,
    
    /// 属性值 (property_value)
    pub value: PropertyValue,
}
#[derive(Debug, Clone)]
pub struct PropertyIdentifier {
    /// 可选的属性集前缀 [property_set_identifier::]
    pub property_set: Option<String>,
    pub name: String,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyOperator {
    Assign,    // =>
    Append,    // +=>
}
/* ========== 属性值系统 ========== */
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Single(PropertyExpression),      // single_property_value
    List(Vec<PropertyListElement>),  // property_list_value
}

#[derive(Debug, Clone)]
pub enum PropertyListElement {
    Value(PropertyExpression),
    NestedList(Vec<PropertyListElement>), // 支持嵌套列表
}

#[derive(Debug, Clone)]
pub enum PropertyExpression {
    // 基础类型
    Boolean(BooleanTerm),
    Real(SignedRealOrConstant),
    Integer(SignedIntergerOrConstant),
    String(StringTerm),
    //Enumeration(EnumerationTerm),
    //Unit(UnitTerm),
    
    // 范围类型
    //IntegerRange(IntegerRangeTerm),
    //RealRange(RealRangeTerm),
    
    // 复杂类型
    //PropertyReference(PropertyTerm),
    //ComponentClassifier(ComponentClassifierTerm),
    //Reference(ReferenceTerm),
    //Record(RecordTerm),
    //Computed(ComputedTerm),
}
/* ========== 属性常量项 ========== */
#[derive(Debug, Clone)]
pub struct PropertyConstantTerm {
    /// 可选的属性集前缀 ([property_set_identifier::])
    pub property_set: Option<String>,
    
    /// 常量标识符 (real_property_constant_term)
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum BooleanTerm {
    Literal(bool),          // boolean_value
    Constant(PropertyConstantTerm), // boolean_property_constant_term
}
/* ========== 符号定义 ========== */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sign {
    Plus,    // +
    Minus,   // -
}
/* ========== 带符号实数或常量 ========== */
/// 对应标准中的 `signed_aadlreal_or_constant`
#[derive(Debug, Clone)]
pub enum SignedRealOrConstant {
    /// 带符号实数 (signed_aadlreal)
    Real(SignedReal),
    
    /// 实数属性常量 ([sign] real_property_constant_term)
    Constant {
        sign: Option<Sign>,
        constant: PropertyConstantTerm,
    },
}

/* ========== 带符号实数 ========== */
/// 对应标准中的 `signed_aadlreal`
#[derive(Debug, Clone)]
pub struct SignedReal {
    /// 可选符号 ([sign])
    pub sign: Option<Sign>,
    
    /// 实数字面量 (real_literal)
    pub value: f64,
    
    /// 可选单位标识符 ([unit_identifier])
    pub unit: Option<String>,
}


/* ========== 带符号整数或常量 ========== */
/// 对应标准中的 `signed_aadlreal_or_constant`
#[derive(Debug, Clone)]
pub enum SignedIntergerOrConstant {
    /// 带符号实数 (signed_aadlreal)
    Real(SignedInteger),
    
    /// 实数属性常量 ([sign] real_property_constant_term)
    Constant {
        sign: Option<Sign>,
        constant: PropertyConstantTerm,
    },
}

/* ========== 带符号实数 ========== */
/// 对应标准中的 `signed_aadlreal`
#[derive(Debug, Clone)]
pub struct SignedInteger {
    /// 可选符号 ([sign])
    pub sign: Option<Sign>,
    
    /// 实数字面量 (real_literal)
    pub value: i64,
    
    /// 可选单位标识符 ([unit_identifier])
    pub unit: Option<String>,
}
/// 字符串项 (string_term)
#[derive(Debug, Clone)]
pub enum StringTerm {
    /// 字面量 (string_literal)
    Literal(String),
    
    /// 字符串常量 (string_property_constant_term)
    Constant(PropertyConstantTerm),
}







}//end mod aadl_ast_cj

