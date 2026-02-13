
use std::collections::{HashMap, HashSet};


    pub fn dedup_with_min_two_unique(vec: &mut Vec<(String, String)>) -> Vec<(String, String)> {
        let mut freq: HashMap<(String, String), usize> = HashMap::new();

        for item in vec.iter() {
            *freq.entry(item.clone()).or_default() += 1;
        }

        vec.retain(|item| freq.get(item).copied().unwrap_or(0) >= 2);

        let mut seen = HashSet::new();
        vec.retain(|item| seen.insert(item.clone()));
        vec.clone()
    }
    pub fn dedup_with_min_two_unique_single_string(vec: &mut Vec<String>) -> Vec<String> {
        let mut freq: HashMap<String, usize> = HashMap::new();
    
        for item in vec.iter() {
            *freq.entry(item.clone()).or_default() += 1;
        }
    
        vec.retain(|item| freq.get(item).copied().unwrap_or(0) >= 2);
    
        let mut seen = HashSet::new();
        vec.retain(|item| seen.insert(item.clone()));
    
        vec.clone()
    }

    pub fn to_upper_camel_case(name: &str) -> String {
        name.split('_')
            .filter(|s| !s.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => {
                        first.to_ascii_uppercase().to_string() +
                            &chars.as_str().to_ascii_lowercase()
                    }
                    None => String::new(),
                }
            })
            .collect::<String>()
    }
