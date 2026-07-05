pub use crate::Tags;
use crate::{
    defs::EnumId,
    raw_defs::{RawEnum, RawEnumItem},
    utility::parse_string2int,
};

pub use serde::{Deserialize, Serialize};
pub use std::collections::HashMap;
use std::collections::HashSet;

/// 定义的 Enum 信息。包含 name、alias、value、comment 等信息。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefEnum {
    #[serde(skip)]
    pub id: EnumId,

    pub name: String,
    pub module: String,
    pub comment: Option<String>,
    #[serde(default)]
    pub tags: Tags,
    #[serde(default)]
    pub groups: Vec<String>,

    #[serde(default)]
    pub is_flags: bool,
    #[serde(default)]
    pub is_unique_item_id: bool,
    #[serde(default)]
    pub items: Vec<DefEnumItem>,

    #[serde(default)]
    name_to_value: HashMap<String, i64>, // name 映射到 value
    #[serde(default)]
    value_to_name: HashMap<i64, String>, // value 映射到 name
    #[serde(default)]
    value_to_alias: HashMap<i64, String>, // value 映射到 alias
}

/// 定义的 Enum 的字段信息。包含 name、alias、value、comment 等信息。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefEnumItem {
    pub auto_index: usize, // 自动索引，用于标识枚举项在定义中的顺序

    pub name: String,
    pub alias: Option<String>,
    pub comment: Option<String>,

    pub value: String,
    pub value_int: i64,

    #[serde(default)]
    pub tags: Tags,
}

impl DefEnum {
    /// 根据 RawEnum 创建 DefEnum 实例
    pub fn new(raw: &RawEnum) -> Self {
        let items = raw
            .items
            .iter()
            .map(|item| DefEnumItem::new(item))
            .collect();

        Self {
            id: Default::default(),

            name: raw.name.clone(),
            module: raw.module.clone(),
            comment: raw.comment.clone(),
            tags: raw.tags.clone(),
            groups: raw.groups.clone(),

            is_flags: raw.is_flags,
            is_unique_item_id: raw.is_unique_item_id,
            items,

            name_to_value: HashMap::new(),
            value_to_name: HashMap::new(),
            value_to_alias: HashMap::new(),
        }
    }

    /// 设置 Enum 的唯一标识符
    pub fn set_id(&mut self, id: EnumId) {
        self.id = id;
    }

    /// 获取 Enum 的全名，格式为 "module.name"。如果 module 为空，则返回 name。
    pub fn full_name(&self) -> String {
        if self.module.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.module, self.name)
        }
    }

    pub fn pre_complie(&mut self) {}

    /// **[核心方法]**：解析枚举项的值，并将其映射到 name、value 和 alias 的 HashMap 中。
    pub fn complie(&mut self) -> Result<(), String> {
        let full_name = self.full_name();
        let mut last_value = -1;
        let mut names_or_aliases: HashSet<String> = HashSet::new();

        // 解构 self 以分别借用各个字段，避免与 iter_mut() 的借用冲突
        let Self {
            ref mut name_to_value,
            ref mut value_to_name,
            ref mut value_to_alias,
            ref mut items,
            ..
        } = *self;

        name_to_value.clear();
        value_to_name.clear();
        value_to_alias.clear();

        let mut flag_item_index: Vec<usize> = vec![];
        for item in items.iter_mut() {
            // 验证 name 和 alias 是否唯一
            if names_or_aliases.contains(&item.name) {
                return Err(format!(
                    "枚举 {} 中项名称 {} 可能与其他项名称或别名冲突",
                    full_name, item.name
                ));
            } else {
                names_or_aliases.insert(item.name.clone());
            }
            if let Some(alias) = &item.alias {
                if names_or_aliases.contains(alias) {
                    return Err(format!(
                        "枚举 {} 中项别名 {} 可能与其他项名称或别名冲突",
                        full_name, alias
                    ));
                } else {
                    names_or_aliases.insert(alias.clone());
                }
            }

            let value = item.value.as_str();
            if value.is_empty() {
                last_value += 1;
                item.value_int = last_value;
                item.value = last_value.to_string();
            } else if let Ok(v) = parse_string2int::<i64>(value) {
                item.value_int = v;
                last_value = v;
            } else if self.is_flags {
                flag_item_index.push(item.auto_index);
                continue;
            } else {
                return Err(format!(
                    "枚举 {} 的项 {} 的值无法解析为整数: {}",
                    full_name, item.name, value
                ));
            }

            // 注册到映射表
            name_to_value.insert(item.name.clone(), item.value_int);
            if let Some(alias) = &item.alias {
                value_to_alias
                    .entry(item.value_int)
                    .or_insert_with(|| alias.clone());
            }
            value_to_name
                .entry(item.value_int)
                .or_insert_with(|| item.name.clone());
        }

        while !flag_item_index.is_empty() {
            let mut unresolved_items: Vec<usize> = Vec::new();
            let mut resolved_any = false;

            for item_index in flag_item_index {
                let item = &mut items[item_index];
                match Self::parse_flag_value(item.value.as_str(), name_to_value) {
                    Ok(value_int) => {
                        item.value_int = value_int;

                        // 注册到映射表
                        name_to_value.insert(item.name.clone(), item.value_int);
                        if let Some(alias) = &item.alias {
                            value_to_alias
                                .entry(item.value_int)
                                .or_insert_with(|| alias.clone());
                        }
                        value_to_name
                            .entry(item.value_int)
                            .or_insert_with(|| item.name.clone());

                        resolved_any = true;
                    }
                    Err(_) => unresolved_items.push(item_index),
                }
            }

            if unresolved_items.is_empty() {
                break;
            }

            if !resolved_any {
                let unresolved_names = unresolved_items
                    .iter()
                    .map(|index| items[*index].name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!(
                    "枚举 {} 的 flags 项无法解析: {}",
                    full_name, unresolved_names
                ));
            }

            flag_item_index = unresolved_items;
        }

        Ok(())
    }

    /// 解析 flags 枚举的值表达式，支持整数、已知名称或别名，以及按位或操作符 '|'。
    fn parse_flag_value(value: &str, known_values: &HashMap<String, i64>) -> Result<i64, String> {
        value
            .split('|')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .try_fold(0_i64, |acc, part| {
                let part_value = if let Ok(v) = parse_string2int::<i64>(part) {
                    v
                } else if let Some(v) = known_values.get(part) {
                    *v
                } else {
                    return Err(format!("无法解析 flags 片段: {}", part));
                };

                Ok(acc | part_value)
            })
    }
}

impl DefEnumItem {
    /// 根据 RawEnumItem 创建 DefEnumItem 实例
    pub fn new(raw: &RawEnumItem) -> Self {
        Self {
            auto_index: 0,
            name: raw.name.clone(),
            alias: raw.alias.clone(),
            comment: raw.comment.clone(),
            value: raw.value.clone(),
            value_int: i64::MIN,
            tags: raw.tags.clone(),
        }
    }

    /// 获取 alias 或 name，如果 alias 存在则返回 alias，否则返回 name
    pub fn alias_or_name(&self) -> &str {
        if let Some(alias) = &self.alias {
            alias
        } else {
            &self.name
        }
    }

    /// 检查是否包含指定的 tag
    pub fn has_tag(&self, tag_key: &str) -> bool {
        self.tags.contains_key(tag_key)
    }

    /// 获取指定 tag 的值
    pub fn get_tag(&self, tag_key: &str) -> Option<&String> {
        self.tags.get(tag_key)
    }
}

#[cfg(test)]
#[path = "def_enum_test.rs"]
mod tests;
