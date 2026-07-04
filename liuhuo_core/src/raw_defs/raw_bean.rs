use crate::Tags;

/// 原始 Bean 字段定义。没有任何结构化的内容。用于从配置文件中读取 Bean 字段定义。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RawField {
    pub name: String,
    pub alias: Option<String>,
    pub r#type: String,
    pub comment: Option<String>,
    #[serde(default)]
    pub tags: Tags,
    #[serde(default)]
    pub variants: Vec<String>,
    #[serde(default)]
    pub not_name_validation: bool,
    #[serde(default)]
    pub groups: Vec<String>,
}

/// 原始 Bean 定义。只包含最基本的信息，不含任何结构化的内容。用于从配置文件中读取 Bean 定义。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RawBean {
    pub namespace: String,
    pub name: String,
    pub parent: Option<String>,

    #[serde(default)]
    pub is_value_type: bool,

    pub comment: Option<String>,

    #[serde(default)]
    pub tags: Tags,

    pub alias: Option<String>,
    pub sep: Option<String>,

    #[serde(default)]
    pub groups: Vec<String>,

    #[serde(default)]
    pub fields: Vec<RawField>,
    // #[serde(default)]
    // pub type_mappers: Vec<TypeMapper>,
}

impl RawBean {
    pub fn full_name(&self) -> String {
        if self.namespace.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.namespace, self.name)
        }
    }
}
