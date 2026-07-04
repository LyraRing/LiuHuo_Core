use crate::Tags;

/// 原始 Enum 字段定义。没有任何结构化的内容。用于从配置文件中读取 Enum 字段定义。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RawEnumItem {
    pub name: String,
    pub alias: Option<String>,
    pub value: String,
    pub comment: Option<String>,

    #[serde(default)]
    pub tags: Tags,
}

/// 原始 Enum 定义。只包含最基本的信息，不含任何结构化的内容。用于从配置文件中读取 Enum 定义。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RawEnum {
    pub namespace: String,
    pub name: String,

    #[serde(default)]
    pub is_flags: bool,

    #[serde(default)]
    pub is_unique_item_id: bool,

    pub comment: Option<String>,

    #[serde(default)]
    pub tags: Tags,

    #[serde(default)]
    pub items: Vec<RawEnumItem>,

    #[serde(default)]
    pub groups: Vec<String>,
}

impl RawEnum {
    pub fn full_name(&self) -> String {
        if self.namespace.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.namespace, self.name)
        }
    }
}
