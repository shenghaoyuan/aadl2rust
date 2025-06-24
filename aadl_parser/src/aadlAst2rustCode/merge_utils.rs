use super::intermediate_ast::*;

/// 对 RustModule 中的 StructDef 进行合并去重
pub fn merge_item_defs(module: RustModule) -> RustModule {
    let mut items = module.items;
    let mut i = 0;
    
    println!("开始合并重复的结构体定义...");
    
    while i < items.len() {
        let current_name = match &items[i] {
            Item::Struct(s) => Some(s.name.clone()),
            _ => None,
        };

        if let Some(name) = current_name {
            let mut j = i + 1;
            
            while j < items.len() {
                let is_same_struct = match &items[j] {
                    Item::Struct(s) => s.name == name,
                    _ => false,
                };

                if is_same_struct {
                    // 使用split_at_mut安全获取两个可变引用
                    let (left, right) = items.split_at_mut(j);
                    if let Item::Struct(target) = &mut left[i] {
                        if let Item::Struct(source) = &right[0] {
                            // 打印将被合并/删除的结构体信息
                            println!("合并重复结构体: {}", name);
                            println!("将被移除的结构体定义位置: {}", j);
                            println!("源结构体字段数: {}", source.fields.len());
                            println!("源结构体属性数: {}", source.properties.len());
                            
                            // 创建source的完整克隆以避免引用问题
                            let source_clone = source.clone();
                            merge_single_struct(target, &source_clone);
                        }
                    }
                    let removed = items.remove(j);
                    if let Item::Struct(removed_struct) = removed {
                        println!("-----------------------------成功移除重复结构体--------------------------: {}", removed_struct.name);
                        println!("----------------------------------------");
                    }
                    continue;
                }
                j += 1;
            }
        }
        i += 1;
    }

    println!("结构体合并完成，共保留 {} 个项", items.len());
    
    RustModule {
        name: module.name,
        docs: module.docs,
        items,
        attrs: module.attrs,
    }
}

/// 合并两个同名的StructDef
fn merge_single_struct(target: &mut StructDef, source: &StructDef) {
    // 合并fields（按name去重）
    let original_field_count = target.fields.len();
    for src_field in source.fields.iter().cloned() {
        if !target.fields.iter().any(|f| f.name == src_field.name) {
            target.fields.push(src_field);
        }
    }
    println!("合并字段: 原 {} 个 + 新增 {} 个 = 现在 {} 个",
        original_field_count,
        source.fields.len(),
        target.fields.len());

    // 合并properties（按name去重）
    let original_prop_count = target.properties.len();
    for src_prop in source.properties.iter().cloned() {
        if !target.properties.iter().any(|p| p.name == src_prop.name) {
            target.properties.push(src_prop);
        }
    }
    println!("合并属性: 原 {} 个 + 新增 {} 个 = 现在 {} 个",
        original_prop_count,
        source.properties.len(),
        target.properties.len());

    // 合并generics（按name去重）
    for src_generic in source.generics.iter().cloned() {
        if !target.generics.iter().any(|g| g.name == src_generic.name) {
            target.generics.push(src_generic);
        }
    }

    // 合并derives（直接去重）
    for derive in &source.derives {
        if !target.derives.contains(derive) {
            target.derives.push(derive.clone());
        }
    }

    // 合并docs（直接拼接）
    //target.docs.extend(source.docs.clone());

    // 保留更宽松的可见性
    if let Visibility::Public = source.vis {
        target.vis = Visibility::Public;
    }
}