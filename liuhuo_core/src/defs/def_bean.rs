use std::collections::{HashMap, HashSet};

pub use crate::Tags;
use crate::{
    defs::{BeanId, DefAssembly, FieldId},
    raw_defs::{RawBean, RawField},
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
    pub parent_str: Option<String>,  // 父类的全名（module.name）
    pub parent: Option<BeanId>,      // 父类 BeanId
    pub root_parent: Option<BeanId>, // 根父类 BeanId
    pub children: Vec<BeanId>,       // 子类 BeanId 列表

    pub hierarchy_not_abstract_children: Vec<BeanId>, // 层级中非抽象类的子类 BeanId 列表

    pub fields: Vec<FieldId>,           // 字段列表
    pub hierarchy_fields: Vec<FieldId>, // 层级字段列表，包括父类的字段

    pub name_to_field_id: HashMap<String, FieldId>, // 字段名到 FieldId 的映射
    pub alias_to_field_id: HashMap<String, FieldId>, // 字段别名到 FieldId 的映射
}

/// Bean 字段定义。包含字段的基本信息和结构化内容。
pub struct DefField {
    pub id: FieldId,  // 字段的唯一标识符
    pub host: BeanId, // 所属 Bean 的 ID

    pub name: String,             // 字段名
    pub alias: Option<String>,    // 别名
    pub comment: Option<String>,  // 字段注释
    pub type_str: String,         // 字段类型（字符串表示）
    pub r#type: Option<TypeKind>, // 字段类型（解析后的 Type 对象）

    pub tags: Tags,
    pub groups: Vec<String>,

    pub variants: Vec<String>, // 字段的变体列表（用于枚举类型的字段）
}

impl DefBean {
    /// 根据 RawBean 创建 DefBean 实例
    pub fn new(raw: &RawBean, asm: &mut DefAssembly) -> Self {
        // 先创建字段列表，然后将字段插入到 DefAssembly 中，并获取字段的 FieldId。
        let mut fields = Vec::with_capacity(raw.fields.len());
        for raw_field in &raw.fields {
            let field = DefField::new(raw_field);
            fields.push(asm.add_field(field));
        }

        Self {
            id: BeanId::default(),

            name: raw.name.clone(),
            module: raw.module.clone(),
            comment: raw.comment.clone(),
            alias: raw.alias.clone(),
            sep: raw.sep.clone(),
            is_value_type: raw.is_value_type,

            tags: raw.tags.clone(),
            groups: raw.groups.clone(),

            is_abstract: false,
            parent_str: raw.parent_str.clone(),
            parent: None,
            root_parent: None,
            children: Vec::new(),

            hierarchy_not_abstract_children: Vec::new(),

            fields,
            hierarchy_fields: Vec::new(),

            name_to_field_id: HashMap::new(),
            alias_to_field_id: HashMap::new(),
        }
    }

    /// 设置 Bean 的唯一标识符，并更新所属字段的 host。
    pub fn set_id(&mut self, id: BeanId, asm: &mut DefAssembly) {
        self.id = id;

        for field_id in self.fields.iter().copied() {
            if let Some(field) = asm.fields.get_mut(field_id) {
                field.host = id;
            }
        }
    }

    /// 获取 Bean 的全名，格式为 "module.name" 或 "name"（如果 module 为空）。
    pub fn full_name(&self) -> String {
        if self.module.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.module, self.name)
        }
    }

    /// 向 Bean 添加子类的 BeanId。
    pub fn add_child(&mut self, child_id: BeanId) {
        self.children.push(child_id);
    }

    /// 预编译 Bean 。
    /// 1. 设置父类和子类的关系。
    /// 2. 收集层级字段，包括父类的字段。
    pub fn pre_complie(&mut self, asm: &mut DefAssembly) {
        self.set_parent(asm); // 设置父类和子类的关系
        self.hierarchy_fields = self.collect_hierarchy_fields(asm, Vec::new()); // 收集层级字段
    }

    /// 编译 Bean 。
    pub fn complie(&mut self, asm: &mut DefAssembly) -> Result<(), String> {
        // 这里可以添加更多的编译逻辑，例如解析字段类型等。
        self.name_to_field_id.clear();
        self.alias_to_field_id.clear();

        let mut name_or_alias = HashSet::new();
        let bean_full_name = self.full_name();

        for field_id in self.hierarchy_fields.iter().copied() {
            // 1. 获取字段的名称、别名和解析后的类型
            let (field_name, field_alias, parsed_type) = {
                let field = asm
                    .get_field(field_id)
                    .ok_or_else(|| format!("Bean '{}' 引用了不存在的字段。", bean_full_name))?;

                let parsed_type = TypeKind::parse(&field.type_str, asm)?;
                (field.name.clone(), field.alias.clone(), parsed_type)
            };

            // 2. 检查字段名和别名是否唯一，并更新映射
            if !name_or_alias.insert(field_name.clone()) {
                return Err(format!(
                    "Bean '{}' 中字段名 '{}' 重复，请检查字段定义。",
                    bean_full_name, field_name
                ));
            }
            self.name_to_field_id.insert(field_name, field_id);

            if let Some(alias) = field_alias {
                if !name_or_alias.insert(alias.clone()) {
                    return Err(format!(
                        "Bean '{}' 中字段别名 '{}' 重复，请检查字段定义。",
                        bean_full_name, alias
                    ));
                }
                self.alias_to_field_id.insert(alias, field_id);
            }

            // 3. 更新字段的解析类型（Bean 只更新自己的字段类型，不更新父类的字段类型）
            if self.fields.contains(&field_id) {
                if let Some(field) = asm.get_field_mut(field_id) {
                    field.r#type = Some(parsed_type);
                }
            }
        }
        Ok(())
    }

    /// 后编译 Bean 。
    pub fn post_complie(&mut self, _asm: &mut DefAssembly) {
        // 这里可以添加更多的后编译逻辑，例如处理继承关系等。
    }

    /// 设置 Bean 的父类，如果 parent_str 存在且在 DefAssembly 中有对应的 BeanId，则设置 parent。
    fn set_parent(&mut self, asm: &mut DefAssembly) {
        if let Some(parent_str) = &self.parent_str {
            // 查找父类的 BeanId，并设置 parent 字段
            if let Some(parent_id) = asm.find_bean_id(parent_str) {
                self.parent = Some(parent_id);

                if let Some(parent_bean) = asm.get_bean_mut(parent_id) {
                    parent_bean.add_child(self.id);
                }
            }
        }
    }

    /// 递归收集 Bean 的层级字段，包括父类的字段。
    fn collect_hierarchy_fields(
        &self,
        asm: &DefAssembly,
        mut hierarchy_fields: Vec<FieldId>,
    ) -> Vec<FieldId> {
        hierarchy_fields.extend(self.fields.iter().copied());

        // 如果有父类，则递归收集父类的层级字段
        if let Some(parent_id) = self.parent {
            if let Some(parent_bean) = asm.get_bean(parent_id) {
                hierarchy_fields = parent_bean.collect_hierarchy_fields(asm, hierarchy_fields);
            }
        }

        hierarchy_fields
    }
}

impl DefField {
    /// 根据 RawField 创建 DefField 实例
    pub fn new(raw: &RawField) -> Self {
        Self {
            id: FieldId::default(),
            host: BeanId::default(),

            name: raw.name.clone(),
            alias: raw.alias.clone(),
            comment: raw.comment.clone(),
            type_str: raw.type_str.clone(),
            r#type: None,

            tags: raw.tags.clone(),
            groups: raw.groups.clone(),

            variants: raw.variants.clone(),
        }
    }

    /// 设置字段的唯一标识符和所属 Bean 的 ID
    pub fn set_id(mut self, id: FieldId) -> Self {
        self.id = id;
        self
    }

    /// 设置字段所属 Bean 的 ID
    pub fn set_host(mut self, host: BeanId) -> Self {
        self.host = host;
        self
    }
}
