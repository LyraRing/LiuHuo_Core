pub use crate::Tags;
use crate::{
    defs::{BeanId, FieldId},
    r#type::TypeKind,
};
pub use serde::{Deserialize, Serialize};

pub struct DefBean {
    pub id: BeanId,

    pub name: String,
    pub module: String,
    pub comment: Option<String>,
    pub alias: Option<String>, // 别名，用于序列化时的字段名
    pub sep: Option<String>,   // 分隔符，用于序列化时的分隔符
    pub is_value_type: bool,   // 是否是值类型（即没有引用类型的字段）

    pub tags: Tags,
    pub groups: Vec<String>,

    pub is_abstract: bool,           // 是否是抽象类
    pub parent: Option<BeanId>,      // 父类 BeanId
    pub root_parent: Option<BeanId>, // 根父类 BeanId
    pub children: Vec<BeanId>,       // 子类 BeanId 列表

    pub hierarchy_not_abstract_children: Vec<BeanId>, // 层级中非抽象类的子类 BeanId 列表

    pub fields: Vec<FieldId>, // 字段列表
}

pub struct DefField {
    pub id: FieldId,

    pub name: String,
    pub alias: Option<String>, // 别名，用于序列化时的字段名
    pub comment: Option<String>,
    pub type_str: String,         // 字段类型（字符串表示）
    pub r#type: Option<TypeKind>, // 字段类型（解析后的 Type 对象）

    pub tags: Tags,
    pub groups: Vec<String>,

    pub is_base_type: bool,  // 是否是基础类型（如 int、float、string 等）
    pub is_collection: bool, // 是否是集合类型（如 array、list、set、map、matrix）
}

impl DefBean {
    /// 获取 Bean 的全名，格式为 "module.name" 或 "name"（如果 module 为空）。
    pub fn full_name(&self) -> String {
        if self.module.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.module, self.name)
        }
    }
}

impl DefField {}
