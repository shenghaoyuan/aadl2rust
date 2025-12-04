use std::collections::{HashMap, HashSet};


    pub fn dedup_with_min_two_unique(vec: &mut Vec<(String, String)>) -> Vec<(String, String)> {
        let mut freq: HashMap<(String, String), usize> = HashMap::new();

        // 统计频次
        for item in vec.iter() {
            *freq.entry(item.clone()).or_default() += 1;
        }

        // 保留出现次数 >= 2 的
        vec.retain(|item| freq.get(item).copied().unwrap_or(0) >= 2);

        // 对剩余的每种项只保留一个
        let mut seen = HashSet::new();
        vec.retain(|item| seen.insert(item.clone()));
        return vec.clone();
    }
