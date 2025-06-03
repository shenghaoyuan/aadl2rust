// aadl_to_rust.rs

use super::ast::aadl_ast_cj::*;
use std::fmt;

/// 代码生成器trait，为所有需要生成Rust代码的AST节点实现
pub trait RustCodeGenerator {
    /// 生成Rust代码的字符串表示
    fn to_rust(&self) -> String;
}

/// 为Package实现代码生成
impl RustCodeGenerator for Package {
    fn to_rust(&self) -> String {
        let mut code = String::new();
        
        // 生成包注释和模块声明
        code.push_str(&format!(
            "//! 自动生成的Rust代码 - 来自AADL包: {}\n\n",
            self.name.to_string()
        ));
        
        // 生成with子句对应的use语句
        for vis_decl in &self.visibility_decls {
            if let VisibilityDeclaration::Import { packages, property_sets } = vis_decl {
                for pkg in packages {
                    code.push_str(&format!("use {}::*;\n", pkg.to_string().replace("::", "_")));
                }
                for ps in property_sets {
                    code.push_str(&format!("use {}_properties::*;\n", ps));
                }
            }
        }
        code.push_str("\n");
        
        // 生成公共部分的声明
        if let Some(public_section) = &self.public_section {
            for decl in &public_section.declarations {
                code.push_str(&decl.to_rust());
                code.push_str("\n");
            }
        }
        
        code
    }
}

/// 为AadlDeclaration实现代码生成
impl RustCodeGenerator for AadlDeclaration {
    fn to_rust(&self) -> String {
        match self {
            AadlDeclaration::ComponentType(ct) => ct.to_rust(),
            AadlDeclaration::ComponentImplementation(ci) => ci.to_rust(),
            //AadlDeclaration::ComponentTypeExtension(cte) => cte.to_rust(),
            //AadlDeclaration::ComponentImplementationExtension(cie) => cie.to_rust(),
            // 其他声明类型暂不处理
            _ => String::new(),
        }
    }
}

/// 为ComponentType实现代码生成
impl RustCodeGenerator for ComponentType {
    fn to_rust(&self) -> String {
        let mut code = String::new();
        
        // 生成组件类型对应的Rust结构体
        code.push_str(&format!(
            "/// {} 组件类型 - 自动生成自AADL {}\n",
            self.identifier, self.category.to_string()
        ));
        
        match self.category {
            ComponentCategory::Thread => generate_thread_struct(&mut code, self),
            ComponentCategory::Process => generate_process_struct(&mut code, self),
            //ComponentCategory::Processor => generate_processor_struct(&mut code, self),
            //ComponentCategory::Memory => generate_memory_struct(&mut code, self),
            //ComponentCategory::Data => generate_data_struct(&mut code, self),
            //ComponentCategory::Subprogram => generate_subprogram_struct(&mut code, self),
            // 其他组件类型
            _ => generate_default_struct(&mut code, self),
        }
        
        code
    }
}

/// 生成线程类型的Rust结构体
fn generate_thread_struct(code: &mut String, ct: &ComponentType) {
    code.push_str(&format!("pub struct {} {{\n", ct.identifier));
    
    // 生成端口字段
    if let FeatureClause::Items(features) = &ct.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                if let PortType::EventData { classifier:Some(classifier_vaule) } 
                        = &port.port_type {
                            if let PortDataTypeReference::Classifier(UniqueComponentClassifierReference::Type(temp_unique)) 
                                = &classifier_vaule {
                                    //TODO：这里还缺少temp_unique.package_prefix
                                    let ImplementationName{type_identifier,implementation_identifier} = &temp_unique.implementation_name ;
                                        code.push_str(&format!(
                                            "    /// {}端口: {:?} {:?}\n",
                                            port.direction.to_string(),
                                            implementation_identifier,
                                            port.identifier
                                        ));
                                        code.push_str(&format!(
                                            "    pub {}: Port<{}>,\n",
                                            port.identifier,
                                            implementation_identifier
                                        ));
                                    
                                    
                                }
                        }

                
            }
        }
    }
    
    // 生成线程特定字段
    code.push_str("    pub is_running: bool,\n");
    code.push_str("}\n\n");
    
    // 生成实现块
    code.push_str(&format!("impl {} {{\n", ct.identifier));
    
    // new() 方法
    code.push_str("    /// 创建新线程实例\n");
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {\n");
    
    // 初始化端口
    if let FeatureClause::Items(features) = &ct.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                code.push_str(&format!(
                    "            {}: Port::new(PortDirection::{:?}),\n",
                    port.identifier,
                    port.direction
                ));
            }
        }
    }
    
    code.push_str("            is_running: false,\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // 根据调度属性生成运行方法
    if let PropertyClause::Properties(props) = &ct.properties {
        if let Some(dispatch_prop) = props.iter().find(|p| match p {
            Property::BasicProperty(bp) => bp.identifier.name == "Dispatch_Protocol",
            _ => false
        }) {
            if let Property::BasicProperty(bp_dispatch) = dispatch_prop {
                if let PropertyValue::Single(PropertyExpression::String(StringTerm::Literal(dispacth_temp))) 
                        = &bp_dispatch.value{
                    match dispacth_temp.to_string().as_str() {
                        "Periodic" => {
                            if let Some(period_prop) = props.iter().find(|p| match p {
                                Property::BasicProperty(bp) => bp.identifier.name == "Period",
                                _ => false
                            }) {
                                if let Property::BasicProperty(bp_period) = period_prop {
                                    if let PropertyValue::Single(PropertyExpression::Integer(SignedIntergerOrConstant::Real(real_temp))) 
                                            = &bp_period.value{
                                        
                                        let unit = match &real_temp.unit {
                                            Some(unit_temp) => unit_temp,
                                            _ => &"None".to_string()
                                        };
                                        code.push_str(&format!(
                                            "    /// 周期性线程运行方法，周期: {}{}\n",
                                            real_temp.value,unit
                                        ));
                                        code.push_str("    pub fn run(&mut self) {\n");
                                        code.push_str("        self.is_running = true;\n");
                                        code.push_str("        // 周期性执行逻辑\n");
                                        code.push_str("    }\n");
                                    }
                                }

                                
                            }
                        }
                        "Sporadic" => {
                            code.push_str("    /// 偶发线程运行方法\n");
                            code.push_str("    pub fn run_on_event(&mut self) {\n");
                            code.push_str("        self.is_running = true;\n");
                            code.push_str("        // 事件触发执行逻辑\n");
                            code.push_str("    }\n");
                        }
                        _ => {}
                    }    
                }
                
            }

            
        }
    }
    
    code.push_str("}\n");
}

/// 生成默认结构体（用于未特别处理的组件类型）
fn generate_default_struct(code: &mut String, ct: &ComponentType) {
    code.push_str(&format!("pub struct {} {{\n", ct.identifier));
    code.push_str("    // 基础组件字段\n");
    code.push_str("}\n\n");
    
    code.push_str(&format!("impl {} {{\n", ct.identifier));
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {}\n");
    code.push_str("    }\n");
    code.push_str("}\n");
}

/// 生成进程(Process)结构体
fn generate_process_struct(code: &mut String, ct: &ComponentType) {
    code.push_str(&format!("pub struct {} {{\n", ct.identifier));
    
    // 进程特有的字段
    code.push_str("    pub threads: Vec<Thread>,\n");
    code.push_str("    pub address_space: MemoryRegion,\n");
    
    // 生成端口字段
    if let FeatureClause::Items(features) = &ct.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                if let PortType::EventData { classifier:Some(classifier_vaule) } 
                        = &port.port_type {
                            if let PortDataTypeReference::Classifier(UniqueComponentClassifierReference::Type(temp_unique)) 
                                = &classifier_vaule {
                                    let ImplementationName{type_identifier,implementation_identifier} = &temp_unique.implementation_name ;
                                    code.push_str(&format!(
                                        "    pub {}: Port<{}>,\n",
                                        port.identifier,
                                        implementation_identifier
                                    ));
                                }
                            }
                            

                
            }
        }
    }
    
    code.push_str("}\n\n");
    
    // 生成实现块
    code.push_str(&format!("impl {} {{\n", ct.identifier));
    
    // new() 方法
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {\n");
    code.push_str("            threads: Vec::new(),\n");
    code.push_str("            address_space: MemoryRegion::new(),\n");
    
    // 初始化端口
    if let FeatureClause::Items(features) = &ct.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                code.push_str(&format!(
                    "            {}: Port::new(PortDirection::{:?}),\n",
                    port.identifier,
                    port.direction
                ));
            }
        }
    }
    
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // 添加进程管理方法
    code.push_str("    /// 启动所有线程\n");
    code.push_str("    pub fn start_all_threads(&mut self) {\n");
    code.push_str("        for thread in &mut self.threads {\n");
    code.push_str("            thread.run();\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    
    code.push_str("}\n");
}

/* 
/// 生成处理器(Processor)结构体
fn generate_processor_struct(code: &mut String, ct: &ComponentType) {
    code.push_str(&format!("pub struct {} {{\n", ct.identifier));
    
    // 处理器特有字段
    code.push_str("    pub clock_speed: u32,  // MHz\n");
    code.push_str("    pub cores: Vec<ProcessorCore>,\n");
    code.push_str("    pub scheduling_policy: SchedulingPolicy,\n");
    
    // 生成端口字段
    if let FeatureClause::Items(features) = &ct.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                code.push_str(&format!(
                    "    pub {}: Port<{}>,\n",
                    port.identifier,
                    port.port_type.to_rust_type()
                ));
            }
        }
    }
    
    code.push_str("}\n\n");
    
    // 生成实现块
    code.push_str(&format!("impl {} {{\n", ct.identifier));
    
    // new() 方法
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {\n");
    
    // 初始化处理器属性
    if let PropertyClause::Properties(props) = &ct.properties {
        if let Some(clock) = props.iter().find(|p| match p {
            Property::BasicProperty(bp) => bp.identifier.name == "Clock_Speed",
            _ => false
        }) {
            code.push_str(&format!(
                "            clock_speed: {},  // 从AADL属性初始化\n",
                clock.value.to_string()
            ));
        } else {
            code.push_str("            clock_speed: 1000,  // 默认1GHz\n");
        }
    } else {
        code.push_str("            clock_speed: 1000,  // 默认1GHz\n");
    }
    
    code.push_str("            cores: Vec::new(),\n");
    code.push_str("            scheduling_policy: SchedulingPolicy::RoundRobin,\n");
    
    // 初始化端口
    if let FeatureClause::Items(features) = &ct.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                code.push_str(&format!(
                    "            {}: Port::new(PortDirection::{:?}),\n",
                    port.identifier,
                    port.direction
                ));
            }
        }
    }
    
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // 添加处理器方法
    code.push_str("    /// 添加处理核心\n");
    code.push_str("    pub fn add_core(&mut self, core: ProcessorCore) {\n");
    code.push_str("        self.cores.push(core);\n");
    code.push_str("    }\n\n");
    
    code.push_str("    /// 设置调度策略\n");
    code.push_str("    pub fn set_scheduling_policy(&mut self, policy: SchedulingPolicy) {\n");
    code.push_str("        self.scheduling_policy = policy;\n");
    code.push_str("    }\n");
    
    code.push_str("}\n");
}

/// 生成内存(Memory)结构体
fn generate_memory_struct(code: &mut String, ct: &ComponentType) {
    code.push_str(&format!("pub struct {} {{\n", ct.identifier));
    
    // 内存特有字段
    code.push_str("    pub size: usize,\n");
    code.push_str("    pub memory_type: MemoryType,\n");
    code.push_str("    pub regions: Vec<MemoryRegion>,\n");
    
    code.push_str("}\n\n");
    
    // 生成实现块
    code.push_str(&format!("impl {} {{\n", ct.identifier));
    
    // new() 方法
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {\n");
    
    // 初始化内存大小
    if let PropertyClause::Properties(props) = &ct.properties {
        if let Some(size) = props.iter().find(|p| match p {
            Property::BasicProperty(bp) => bp.identifier.name == "Memory_Size",
            _ => false
        }) {
            code.push_str(&format!(
                "            size: {},  // 从AADL属性初始化\n",
                size.value.to_string()
            ));
        } else {
            code.push_str("            size: 1024,  // 默认1KB\n");
        }
    } else {
        code.push_str("            size: 1024,  // 默认1KB\n");
    }
    
    code.push_str("            memory_type: MemoryType::RAM,\n");
    code.push_str("            regions: Vec::new(),\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // 添加内存方法
    code.push_str("    /// 添加内存区域\n");
    code.push_str("    pub fn add_region(&mut self, region: MemoryRegion) {\n");
    code.push_str("        self.regions.push(region);\n");
    code.push_str("    }\n\n");
    
    code.push_str("    /// 获取总内存大小\n");
    code.push_str("    pub fn total_size(&self) -> usize {\n");
    code.push_str("        self.size\n");
    code.push_str("    }\n");
    
    code.push_str("}\n");
}

/// 生成数据(Data)结构体
fn generate_data_struct(code: &mut String, ct: &ComponentType) {
    code.push_str(&format!("pub struct {} {{\n", ct.identifier));
    
    // 数据特有字段
    code.push_str("    pub data_type: DataType,\n");
    code.push_str("    pub initial_value: Option<String>,\n");
    code.push_str("    pub size: usize,\n");
    
    code.push_str("}\n\n");
    
    // 生成实现块
    code.push_str(&format!("impl {} {{\n", ct.identifier));
    
    // new() 方法
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {\n");
    
    // 初始化数据属性
    code.push_str("            data_type: DataType::from_str(\"");
    if let PropertyClause::Properties(props) = &ct.properties {
        if let Some(source_name) = props.iter().find(|p| match p {
            Property::BasicProperty(bp) => bp.identifier.name == "Source_Name",
            _ => false
        }) {
            code.push_str(&format!("{}\"),\n", source_name.value.to_string()));
        } else {
            code.push_str("unknown\"),\n");
        }
        
        if let Some(init_value) = props.iter().find(|p| match p {
            Property::BasicProperty(bp) => bp.identifier.name.contains("Initial_Value"),
            _ => false
        }) {
            code.push_str(&format!(
                "            initial_value: Some({}.to_string()),\n",
                init_value.value.to_string()
            ));
        } else {
            code.push_str("            initial_value: None,\n");
        }
    } else {
        code.push_str("unknown\"),\n");
        code.push_str("            initial_value: None,\n");
    }
    
    code.push_str("            size: 0,\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // 添加数据方法
    code.push_str("    /// 设置初始值\n");
    code.push_str("    pub fn set_initial_value(&mut self, value: String) {\n");
    code.push_str("        self.initial_value = Some(value);\n");
    code.push_str("    }\n\n");
    
    code.push_str("    /// 获取数据类型\n");
    code.push_str("    pub fn get_type(&self) -> &DataType {\n");
    code.push_str("        &self.data_type\n");
    code.push_str("    }\n");
    
    code.push_str("}\n");
}
*/


/*// 生成子程序(Subprogram)结构体
fn generate_subprogram_struct(code: &mut String, ct: &ComponentType) {
    code.push_str(&format!("pub struct {} {{\n", ct.identifier));
    
    // 子程序特有字段
    code.push_str("    pub parameters: Vec<Parameter>,\n");
    code.push_str("    pub return_type: Option<DataType>,\n");
    code.push_str("    pub source_language: String,\n");
    code.push_str("    pub source_name: String,\n");
    
    // 生成端口字段（子程序可能有参数端口）
    if let FeatureClause::Items(features) = &ct.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                code.push_str(&format!(
                    "    pub {}: Port<{}>,\n",
                    port.identifier,
                    port.port_type.to_rust_type()
                ));
            }
        }
    }
    
    code.push_str("}\n\n");
    
    // 生成实现块
    code.push_str(&format!("impl {} {{\n", ct.identifier));
    
    // new() 方法
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {\n");
    code.push_str("            parameters: Vec::new(),\n");
    code.push_str("            return_type: None,\n");
    
    // 初始化源代码信息
    if let PropertyClause::Properties(props) = &ct.properties {
        if let Some(lang) = props.iter().find(|p| match p {
            Property::BasicProperty(bp) => bp.identifier.name == "Source_Language",
            _ => false
        }) {
            code.push_str(&format!(
                "            source_language: {}.to_string(),\n",
                lang.value.to_string()
            ));
        } else {
            code.push_str("            source_language: \"unknown\".to_string(),\n");
        }
        
        if let Some(name) = props.iter().find(|p| match p {
            Property::BasicProperty(bp) => bp.identifier.name == "Source_Name",
            _ => false
        }) {
            code.push_str(&format!(
                "            source_name: {}.to_string(),\n",
                name.value.to_string()
            ));
        } else {
            code.push_str("            source_name: \"unknown\".to_string(),\n");
        }
    } else {
        code.push_str("            source_language: \"unknown\".to_string(),\n");
        code.push_str("            source_name: \"unknown\".to_string(),\n");
    }
    
    // 初始化端口
    if let FeatureClause::Items(features) = &ct.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                code.push_str(&format!(
                    "            {}: Port::new(PortDirection::{:?}),\n",
                    port.identifier,
                    port.direction
                ));
            }
        }
    }
    
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // 添加子程序方法
    code.push_str("    /// 调用子程序\n");
    code.push_str("    pub fn call(&self, args: &[ParameterValue]) -> Option<ParameterValue> {\n");
    code.push_str("        // 子程序调用逻辑\n");
    code.push_str("        None\n");
    code.push_str("    }\n\n");
    
    code.push_str("    /// 添加参数\n");
    code.push_str("    pub fn add_parameter(&mut self, param: Parameter) {\n");
    code.push_str("        self.parameters.push(param);\n");
    code.push_str("    }\n");
    
    code.push_str("}\n");
}
*/


//*********************************************************************************************** */
/// 为ComponentImplementation实现代码生成

impl SubcomponentClassifier {
    /// 将AADL组件分类器转换为Rust类型名称
    pub fn to_rust_type(&self) -> String {
        match self {
            // 处理分类器引用的情况
            SubcomponentClassifier::ClassifierReference(classifier_ref) => {
                match classifier_ref {
                    // 类型引用（引用组件类型）
                    UniqueComponentClassifierReference::Type(type_ref) => {
                        let ImplementationName{type_identifier,implementation_identifier} = 
                                &type_ref.implementation_name ;
                        // 如果有包前缀，添加前缀
                        if let Some(pkg) = &type_ref.package_prefix {
                            format!("{}::{}", pkg.to_string().replace("::", "_"), implementation_identifier)
                        } else {
                            implementation_identifier.clone()
                        }
                    }
                    // 实现引用（引用组件实现）
                    UniqueComponentClassifierReference::Implementation(impl_ref) => {
                        // 实现名称格式为"type_name.impl_name"
                        // 我们通常只需要类型部分作为Rust类型
                        impl_ref.implementation_name.type_identifier.clone()
                    }
                }
            }
            // 处理原型引用的情况
            SubcomponentClassifier::Prototype(proto_name) => {
                // 原型名称直接作为类型名
                proto_name.clone()
            }
        }
    }
}

impl RustCodeGenerator for ComponentImplementation {
    fn to_rust(&self) -> String {
        let mut code = String::new();
        
        // 生成实现注释
        code.push_str(&format!(
            "/// {} 实现 - 自动生成自AADL {}.{}\n",
            self.name.to_string(),
            self.name.type_identifier,
            self.name.implementation_identifier
        ));
        
        // 生成结构体
        code.push_str(&format!("pub struct {} {{\n", self.name.to_string()));
        
        // 生成子组件字段
        if let SubcomponentClause::Items(subcomponents) = &self.subcomponents {
            for sub in subcomponents {
                code.push_str(&format!(
                    "    /// {}子组件: {}\n",
                    sub.category.to_string(),
                    sub.identifier
                ));
                code.push_str(&format!(
                    "    pub {}: {},\n",
                    sub.identifier.to_lowercase(),
                    sub.classifier.to_rust_type()
                ));
            }
        }
        
        code.push_str("}\n\n");
        
        // 生成实现块
        code.push_str(&format!("impl {} {{\n", self.name.to_string()));
        
        // new() 方法
        code.push_str("    /// 创建新实例\n");
        code.push_str("    pub fn new() -> Self {\n");
        code.push_str("        Self {\n");
        
        // 初始化子组件
        if let SubcomponentClause::Items(subcomponents) = &self.subcomponents {
            for sub in subcomponents {
                code.push_str(&format!(
                    "            {}: {}::new(),\n",
                    sub.identifier.to_lowercase(),
                    sub.classifier.to_rust_type()
                ));
            }
        }
        
        code.push_str("        }\n");
        code.push_str("    }\n\n");
        
        // 生成连接设置方法
        if let ConnectionClause::Items(connections) = &self.connections {
            code.push_str("    /// 设置组件间连接\n");
            code.push_str("    pub fn setup_connections(&mut self) {\n");
            for conn in connections {
                code.push_str(&format!("        {}\n", conn.to_rust_connection()));
            }
            code.push_str("    }\n");
        }
        
        // 生成调用序列方法
        if let CallSequenceClause::Items(calls) = &self.calls {
            code.push_str("\n    /// 执行调用序列\n");
            code.push_str("    pub fn execute_call_sequence(&self) {\n");
            for call in calls {
                //code.push_str(&format!("        {}\n", call.to_rust_call()));
            }
            code.push_str("    }\n");
        }
        
        code.push_str("}\n");
        
        code
    }
}

/// 为PortType实现辅助方法
// impl PortType {
//     fn to_string(&self) -> String {
//         match self {
//             PortType::Data { .. } => "Data Port".to_string(),
//             PortType::EventData { .. } => "Event Data Port".to_string(),
//             PortType::Event => "Event Port".to_string(),
//         }
//     }
    
//     fn to_rust_type(&self) -> String {
//         match self {
//             PortType::Data { classifier } => {
//                 if let Some(classifier) = classifier {
//                     match classifier {
//                         PortDataTypeReference::Classifier(ref_) => ref_.to_rust_type(),
//                         PortDataTypeReference::Prototype(name) => name.clone(),
//                     }
//                 } else {
//                     "Data".to_string()
//                 }
//             }
//             PortType::EventData { .. } => "EventData".to_string(),
//             PortType::Event => "Event".to_string(),
//         }
//     }
// }

/// 为UniqueComponentClassifierReference实现辅助方法
// impl UniqueComponentClassifierReference {
//     fn to_rust_type(&self) -> String {
//         match self {
//             UniqueComponentClassifierReference::Type(type_ref) => type_ref.identifier.clone(),
//             UniqueComponentClassifierReference::Implementation(impl_ref) => 
//                 impl_ref.implementation_name.type_identifier.clone(),
//         }
//     }
// }

/// 为Connection实现代码生成辅助方法
impl Connection {
    fn to_rust_connection(&self) -> String {
        match self {
            Connection::Port(pc) => {
                format!(
                    "self.{}.connect(&mut self.{});  // {}连接",
                    pc.source.get_component_name(),
                    pc.destination.get_component_name(),
                    pc.connection_direction.to_string()
                )
            }
            Connection::Parameter(pc) => {
                format!(
                    "self.{}.bind(&mut self.{});  // 参数绑定",
                    pc.source.get_component_name(),
                    pc.destination.get_component_name()
                )
            }
            _ => "// 未实现的连接类型".to_string(),
        }
    }
}

/// 为PortEndpoint实现辅助方法
impl PortEndpoint {
    fn get_component_name(&self) -> String {
        match self {
            PortEndpoint::SubcomponentPort { subcomponent, .. } => subcomponent.clone(),
            PortEndpoint::ComponentPort(name) => name.clone(),
            _ => "unknown".to_string(),
        }
    }
}

impl ParameterEndpoint {
    fn get_component_name(&self) -> String {
        match self {
            ParameterEndpoint::ComponentParameter { parameter, .. } => parameter.clone(),
            ParameterEndpoint::SubprogramCallParameter { call_identifier, ..} => call_identifier.clone(),
            _ => "unknown".to_string(),
        }
    }
}

// /// 为CallSequence实现辅助方法
// impl CallSequence {
//     fn to_rust_call(&self) -> String {
//         if let Some(call) = self.calls.first() {
//             format!(
//                 "self.{}.call();  // 调用子程序{}",
//                 call.identifier,
//                 match &call.called {
//                     CalledSubprogram::Classifier(ref_) => ref_.to_rust_type(),
//                 }
//             )
//         } else {
//             "// 空调用序列".to_string()
//         }
//     }
// }

/// 为ComponentCategory实现Display
impl fmt::Display for ComponentCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ComponentCategory::Thread => write!(f, "Thread"),
            ComponentCategory::Process => write!(f, "Process"),
            ComponentCategory::Processor => write!(f, "Processor"),
            ComponentCategory::Memory => write!(f, "Memory"),
            ComponentCategory::Data => write!(f, "Data"),
            ComponentCategory::Subprogram => write!(f, "Subprogram"),
            _ => write!(f, "Unknown"),
        }
    }
}

/// 为PortDirection实现Display
impl fmt::Display for PortDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PortDirection::In => write!(f, "In"),
            PortDirection::Out => write!(f, "Out"),
            PortDirection::InOut => write!(f, "InOut"),
        }
    }
}

/// 为ConnectionSymbol实现Display
impl fmt::Display for ConnectionSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionSymbol::Direct => write!(f, "直接"),
            ConnectionSymbol::Didirect => write!(f, "双向"),
        }
    }
}