use std::fs;
use std::path::Path;
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub struct AADLModelMetrics {
    pub file_name: String,
    pub file_size: usize,
    pub line_count: usize,
    pub structural_metrics: StructuralMetrics,
    pub behavioral_metrics: BehavioralMetrics,
    // 移除 complexity_score 字段
}

#[derive(Debug, Clone)]
pub struct StructuralMetrics {
    pub data_types: usize,
    pub subprograms: usize,
    pub threads: usize,
    pub processes: usize,
    pub processors: usize,
    pub systems: usize,
    pub connections: usize,
    pub features: usize,
    pub properties: usize,
}

#[derive(Debug, Clone)]
pub struct BehavioralMetrics {
    pub behavior_annexes: usize,
    pub states: usize,
    pub transitions: usize,
    pub variables: usize,
    pub computations: usize,
    pub guards: usize,
    pub periodic_threads: usize,
    pub sporadic_threads: usize,
    pub aperiodic_threads: usize,
}

impl Default for StructuralMetrics {
    fn default() -> Self {
        StructuralMetrics {
            data_types: 0,
            subprograms: 0,
            threads: 0,
            processes: 0,
            processors: 0,
            systems: 0,
            connections: 0,
            features: 0,
            properties: 0,
        }
    }
}

impl Default for BehavioralMetrics {
    fn default() -> Self {
        BehavioralMetrics {
            behavior_annexes: 0,
            states: 0,
            transitions: 0,
            variables: 0,
            computations: 0,
            guards: 0,
            periodic_threads: 0,
            sporadic_threads: 0,
            aperiodic_threads: 0,
        }
    }
}

pub struct AADLModelEvaluator {
    models: Vec<AADLModelMetrics>,
}

impl AADLModelEvaluator {
    pub fn new() -> Self {
        AADLModelEvaluator {
            models: Vec::new(),
        }
    }

    pub fn analyze_file(&mut self, file_path: &str) -> Result<AADLModelMetrics, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let file_name = Path::new(file_path).file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        
        // 删除注释行
        let cleaned_content = self.remove_comments(&content);
        
        let mut metrics = AADLModelMetrics {
            file_name: file_name.clone(),
            file_size: content.len(), // 保持原始文件大小
            line_count: cleaned_content.lines().count(), // 使用清理后的行数
            structural_metrics: StructuralMetrics::default(),
            behavioral_metrics: BehavioralMetrics::default(),
        };

        self.analyze_structural_complexity(&cleaned_content, &mut metrics.structural_metrics);
        self.analyze_behavioral_complexity(&cleaned_content, &mut metrics.behavioral_metrics);
        
        self.models.push(metrics.clone());
        Ok(metrics)
    }

    fn remove_comments(&self, content: &str) -> String {
        content
            .lines()
            .filter(|line| !line.trim().starts_with("--"))
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn analyze_structural_complexity(&self, content: &str, metrics: &mut StructuralMetrics) {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_connections = false;
        let mut in_features = false;
        let mut in_properties = false;
        
        for line in lines.iter() {
            let trimmed = line.trim();
            
            // 统计组件类型
            if trimmed.starts_with("data ") && !trimmed.contains("implementation") {
                metrics.data_types += 1;
            }
            if trimmed.starts_with("subprogram ") && !trimmed.contains("implementation") {
                metrics.subprograms += 1;
            }
            if trimmed.starts_with("thread ") && !trimmed.contains("implementation") {
                metrics.threads += 1;
            }
            if trimmed.starts_with("process ") && !trimmed.contains("implementation") {
                metrics.processes += 1;
            }
            if trimmed.starts_with("processor ") && !trimmed.contains("implementation") {
                metrics.processors += 1;
            }
            if trimmed.starts_with("system ") && !trimmed.contains("implementation") {
                metrics.systems += 1;
            }
            
            // 检测connections块开始
            if trimmed.contains("connections") && !trimmed.contains("end") {
                in_connections = true;
                continue;
            }
            
            // 检测features块开始
            if trimmed.contains("features") && !trimmed.contains("end") {
                in_features = true;
                continue;
            }
            
            // 检测properties块开始
            if trimmed.contains("properties") && !trimmed.contains("end") {
                in_properties = true;
                continue;
            }
            
            // 在connections块内统计连接
            if in_connections {
                if trimmed.contains("end") || trimmed.contains("};") {
                    in_connections = false;
                    continue;
                }
                // 连接通常以 "连接名: 源 -> 目标;" 的形式出现
                if trimmed.contains(":") && trimmed.contains("->") {
                    metrics.connections += 1;
                }
            }
            
            // 在features块内统计特性
            if in_features {
                if trimmed.contains("end") || trimmed.contains("};") {
                    in_features = false;
                    continue;
                }
                // 特性通常以 "特性名: 方向 类型;" 的形式出现
                if trimmed.contains(":") && (trimmed.contains("in") || trimmed.contains("out") || 
                    trimmed.contains("in out") || trimmed.contains("data") || 
                    trimmed.contains("event") || trimmed.contains("event data")) {
                    metrics.features += 1;
                }
            }
            
            // 在properties块内统计属性
            if in_properties {
                if trimmed.contains("end") || trimmed.contains("};") {
                    in_properties = false;
                    continue;
                }
                // 属性通常以 "属性名 => 值;" 的形式出现
                if trimmed.contains("=>") {
                    metrics.properties += 1;
                }
            }
        }
    }

    fn analyze_behavioral_complexity(&self, content: &str, metrics: &mut BehavioralMetrics) {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_behavior_annex = false;
        let mut in_states = false;
        let mut in_transitions = false;
        let mut in_variables = false;
        
        for line in lines.iter() {
            let trimmed = line.trim();
            
            // 检测行为规范开始
            if trimmed.contains("annex Behavior_specification") {
                metrics.behavior_annexes += 1;
                in_behavior_annex = true;
                continue;
            }
            
            if in_behavior_annex {
                if trimmed.contains("**};") {
                    in_behavior_annex = false;
                    continue;
                }
                
                // 检测states块
                if trimmed.contains("states") && !trimmed.contains("end") {
                    in_states = true;
                    continue;
                }
                
                // 检测transitions块
                if trimmed.contains("transitions") && !trimmed.contains("end") {
                    in_transitions = true;
                    continue;
                }
                
                // 检测variables块
                if trimmed.contains("variables") && !trimmed.contains("end") {
                    in_variables = true;
                    continue;
                }
                
                // 在states块内统计状态
                if in_states {
                    if trimmed.contains("end") || trimmed.contains("};") {
                        in_states = false;
                        continue;
                    }
                    // 状态通常以 "状态名: 状态类型;" 的形式出现
                    if trimmed.contains(":") {
                        metrics.states += 1;
                    }
                }
                
                // 在transitions块内统计转换
                if in_transitions {
                    if trimmed.contains("end") || trimmed.contains("};") {
                        in_transitions = false;
                        continue;
                    }
                    // 转换通常以 "源状态 -> 目标状态" 的形式出现
                    if trimmed.contains("->") {
                        metrics.transitions += 1;
                    }
                }
                
                // 在variables块内统计变量
                if in_variables {
                    if trimmed.contains("end") || trimmed.contains("};") {
                        in_variables = false;
                        continue;
                    }
                    // 变量通常以 "变量名: 类型;" 的形式出现
                    if trimmed.contains(":") {
                        metrics.variables += 1;
                    }
                }
                
                // 统计计算操作
                if trimmed.contains("computation") {
                    metrics.computations += 1;
                }
                
                // 统计守卫条件
                if trimmed.contains("-[") && trimmed.contains("]->") {
                    metrics.guards += 1;
                }
            }
            
            // 统计调度协议
            if trimmed.contains("Dispatch_Protocol") {
                if trimmed.contains("Periodic") {
                    metrics.periodic_threads += 1;
                } else if trimmed.contains("Sporadic") {
                    metrics.sporadic_threads += 1;
                } else if trimmed.contains("Aperiodic") {
                    metrics.aperiodic_threads += 1;
                }
            }
        }
    }

    // 移除 calculate_complexity_score, calculate_structural_score, calculate_behavioral_score 方法

    pub fn analyze_directory(&mut self, dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let entries = fs::read_dir(dir_path)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "aadl") {
                let file_path = path.to_string_lossy();
                match self.analyze_file(&file_path) {
                    Ok(_) => println!("成功分析文件: {}", file_path),
                    Err(e) => eprintln!("分析文件失败 {}: {}", file_path, e),
                }
            }
        }
        
        Ok(())
    }

    pub fn generate_report(&self) {
        println!("\n=== AADL模型分析报告 ===");
        println!("共分析了 {} 个AADL模型\n", self.models.len());
        
        for (i, model) in self.models.iter().enumerate() {
            println!("模型 {}: {}", i + 1, model.file_name);
            println!("  文件大小: {} 字节", model.file_size);
            println!("  代码行数: {} 行", model.line_count);
            
            println!("  结构规模指标:");
            println!("    数据类型: {}", model.structural_metrics.data_types);
            println!("    子程序: {}", model.structural_metrics.subprograms);
            println!("    线程: {}", model.structural_metrics.threads);
            println!("    进程: {}", model.structural_metrics.processes);
            println!("    处理器: {}", model.structural_metrics.processors);
            println!("    系统: {}", model.structural_metrics.systems);
            println!("    连接: {}", model.structural_metrics.connections);
            println!("    特性: {}", model.structural_metrics.features);
            
            println!("  行为复杂度指标:");
            println!("    行为规范: {}", model.behavioral_metrics.behavior_annexes);
            println!("    状态数: {}", model.behavioral_metrics.states);
            println!("    转换数: {}", model.behavioral_metrics.transitions);
            println!("    变量数: {}", model.behavioral_metrics.variables);
            println!("    计算操作: {}", model.behavioral_metrics.computations);
            println!("    守卫条件: {}", model.behavioral_metrics.guards);
            println!("    周期性线程: {}", model.behavioral_metrics.periodic_threads);
            println!("    偶发性线程: {}", model.behavioral_metrics.sporadic_threads);
            println!("    非周期性线程: {}", model.behavioral_metrics.aperiodic_threads);
            println!();
        }
        
        self.print_summary();
    }

    fn print_summary(&self) {
        if self.models.is_empty() {
            return;
        }
        
        let total_files = self.models.len();
        let total_size: usize = self.models.iter().map(|m| m.file_size).sum();
        let total_lines: usize = self.models.iter().map(|m| m.line_count).sum();
        
        println!("=== 统计摘要 ===");
        println!("总文件数: {}", total_files);
        println!("总文件大小: {} 字节", total_size);
        println!("总代码行数: {} 行", total_lines);
        println!("平均文件大小: {:.0} 字节", total_size as f64 / total_files as f64);
        println!("平均代码行数: {:.0} 行", total_lines as f64 / total_files as f64);
    }
}

// 定义测试用例结构
pub struct TestCase {
    pub id: u32,
    pub name: String,
    pub path: String,
    pub description: String,
}

impl TestCase {
    pub fn new(id: u32, name: String, path: String, description: String) -> Self {
        TestCase {
            id,
            name,
            path,
            description,
        }
    }
}

fn main() {
    // 定义可用的测试用例
    let test_cases = vec![
        TestCase::new(
            1,
            "PingPong (Ocarina)".to_string(),
            "AADLSource/pingpong_ocarina.aadl".to_string(),
            "Ocarina版本的PingPong示例".to_string(),
        ),
        TestCase::new(
            2,
            "PingPong (Simple)".to_string(),
            "AADLSource/pingpong_simple.aadl".to_string(),
            "简化版本的PingPong示例".to_string(),
        ),
        TestCase::new(
            3,
            "RMA".to_string(),
            "AADLSource/rma.aadl".to_string(),
            "速率单调分析示例".to_string(),
        ),
        TestCase::new(
            4,
            "Toy".to_string(),
            "AADLSource/toy.aadl".to_string(),
            "玩具示例模型".to_string(),
        ),
        TestCase::new(
            5,
            "Robot(v1)".to_string(),
            "AADLSource/robotv1.aadl".to_string(),
            "机器人模型第一版".to_string(),
        ),
        TestCase::new(
            6,
            "Robot(v2)".to_string(),
            "AADLSource/robotv2.aadl".to_string(),
            "机器人模型第二版".to_string(),
        ),
    ];

    println!("=== AADL模型分析工具 ===");
    
    let mut evaluator = AADLModelEvaluator::new();
    
    loop {
        println!("\n请选择操作:");
        println!("1: 分析预定义测试用例");
        println!("2: 分析单个文件");
        println!("3: 分析整个目录");
        println!("4: 查看已分析的文件");
        println!("0: 退出程序");
        print!("请输入选择 (0-4): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("读取输入失败");
        
        let choice = input.trim();
        
        match choice {
            "0" => {
                println!("程序退出。");
                break;
            },
            "1" => {
                process_test_cases(&test_cases, &mut evaluator);
            },
            "2" => {
                process_single_file(&mut evaluator);
            },
            "3" => {
                process_directory(&mut evaluator);
            },
            "4" => {
                if evaluator.models.is_empty() {
                    println!("还没有分析任何文件。");
                } else {
                    evaluator.generate_report();
                }
            },
            _ => {
                println!("无效选择，请输入 0-4 之间的数字。");
            }
        }
    }
}

fn process_test_cases(test_cases: &[TestCase], evaluator: &mut AADLModelEvaluator) {
    // 显示可用的测试用例
    println!("\n=== 预定义测试用例选择 ===");
    println!("请选择要分析的AADL文件:");
    for test_case in test_cases {
        println!("  {}: {} - {}", test_case.id, test_case.name, test_case.description);
    }
    println!("  0: 返回主菜单");
    print!("请输入选择 (0-{}): ", test_cases.len());
    io::stdout().flush().unwrap();

    // 读取用户输入
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("无法读取输入");
    
    let choice: u32 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("无效输入，请输入数字");
            return;
        }
    };

    if choice == 0 {
        return;
    }

    // 查找选择的测试用例
    let selected_test = test_cases.iter().find(|tc| tc.id == choice);
    match selected_test {
        Some(test_case) => {
            println!("\n选择: {}", test_case.name);
            println!("文件路径: {}", test_case.path);
            println!("描述: {}", test_case.description);
            
            println!("\n正在分析文件: {}...", test_case.name);
            match evaluator.analyze_file(&test_case.path) {
                Ok(_) => {
                    println!("分析完成！");
                    evaluator.generate_report();
                },
                Err(e) => eprintln!("分析失败: {}", e),
            }
        }
        None => {
            println!("无效的选择，请输入 0-{} 之间的数字", test_cases.len());
        }
    }
}

fn process_single_file(evaluator: &mut AADLModelEvaluator) {
    print!("\n请输入AADL文件路径: ");
    io::stdout().flush().unwrap();
    
    let mut file_path = String::new();
    io::stdin().read_line(&mut file_path).expect("读取输入失败");
    let file_path = file_path.trim();
    
    if file_path.is_empty() {
        println!("文件路径不能为空！");
        return;
    }
    
    println!("\n正在分析文件: {}...", file_path);
    match evaluator.analyze_file(file_path) {
        Ok(_) => {
            println!("分析完成！");
            evaluator.generate_report();
        },
        Err(e) => eprintln!("分析失败: {}", e),
    }
}

fn process_directory(evaluator: &mut AADLModelEvaluator) {
    print!("\n请输入目录路径: ");
    io::stdout().flush().unwrap();
    
    let mut dir_path = String::new();
    io::stdin().read_line(&mut dir_path).expect("读取输入失败");
    let dir_path = dir_path.trim();
    
    if dir_path.is_empty() {
        println!("目录路径不能为空！");
        return;
    }
    
    println!("\n正在分析目录: {}...", dir_path);
    match evaluator.analyze_directory(dir_path) {
        Ok(_) => {
            println!("目录分析完成！");
            evaluator.generate_report();
        },
        Err(e) => eprintln!("目录分析失败: {}", e),
    }
}