pub mod def_bean;
pub mod def_enum;

pub use def_bean::*;
pub use def_enum::*;

use slotmap::SlotMap;

use slotmap::new_key_type;

new_key_type! {
    pub struct EnumId;
    pub struct BeanId;
    pub struct FieldId;
    pub struct TableId;
}

/// 定义集合，管理所有已注册的 Enum、Bean、Field 定义。
pub struct DefAssembly {
    enums: SlotMap<EnumId, DefEnum>,
    beans: SlotMap<BeanId, DefBean>,
    fields: SlotMap<FieldId, DefField>,

    pub full_name_to_enum_id: HashMap<String, EnumId>,
    pub full_name_to_bean_id: HashMap<String, BeanId>,
}

impl DefAssembly {
    /// 创建一个空的 DefAssembly
    pub fn new() -> Self {
        Self {
            enums: SlotMap::with_key(),
            beans: SlotMap::with_key(),
            fields: SlotMap::with_key(),
            full_name_to_enum_id: HashMap::new(),
            full_name_to_bean_id: HashMap::new(),
        }
    }

    /// 添加一个 Enum 定义，自动分配 id 并注册 full_name 映射。
    /// 返回分配的 EnumId。
    pub fn add_enum(&mut self, mut def_enum: DefEnum) -> Result<EnumId, String> {
        // 1. 检查 Enum 的 full_name 是否已存在（不允许重复定义）。
        let full_name = def_enum.full_name();
        if self.full_name_to_enum_id.contains_key(&full_name) {
            return Err(format!("枚举 '{}' 已存在，请不要重复定义。", full_name));
        }

        // 2. 向 DefAssembly 中添加 Enum，并分配 EnumId
        let enum_id = self.enums.insert_with_key(|id| {
            def_enum.set_id(id);
            def_enum
        });

        // 3. 注册 full_name 映射
        self.full_name_to_enum_id.insert(full_name, enum_id);

        Ok(enum_id)
    }

    /// 添加一个 Bean 定义，自动分配 id、更新字段 host，并注册 full_name 映射。
    pub fn add_bean(&mut self, mut def_bean: DefBean) -> Result<BeanId, String> {
        // 1. 判断 Bean 的 full_name 是否已存在（不允许重复定义）。
        let full_name = def_bean.full_name();
        if self.full_name_to_bean_id.contains_key(&full_name) {
            return Err(format!("Bean '{}' 已存在，请不要重复定义。", full_name));
        }

        // 2. 向 DefAssembly 中添加 Bean，并分配 BeanId
        let bean_id = self.beans.insert_with_key(|id| {
            def_bean.id = id;
            def_bean
        });

        // 3. 注册 full_name 映射
        self.full_name_to_bean_id.insert(full_name, bean_id);

        // 4. 更新字段的 host 为当前 BeanId
        let field_ids = self.beans[bean_id].fields.clone();
        for field_id in field_ids {
            if let Some(field) = self.fields.get_mut(field_id) {
                field.host = bean_id;
            }
        }

        Ok(bean_id)
    }

    /// 添加一个字段定义，自动分配 id。
    pub fn add_field(&mut self, mut def_field: DefField) -> FieldId {
        // 1. 向 DefAssembly 中添加字段，并分配 FieldId。
        self.fields.insert_with_key(|id| {
            def_field.id = id;
            def_field
        })
    }

    /// 根据 EnumId 获取 Enum 定义
    pub fn get_enum(&self, id: EnumId) -> Option<&DefEnum> {
        self.enums.get(id)
    }

    /// 根据 EnumId 获取可变的 Enum 定义
    pub fn get_enum_mut(&mut self, id: EnumId) -> Option<&mut DefEnum> {
        self.enums.get_mut(id)
    }

    /// 根据 BeanId 获取 Bean 定义
    pub fn get_bean(&self, id: BeanId) -> Option<&DefBean> {
        self.beans.get(id)
    }

    /// 根据 BeanId 获取可变的 Bean 定义
    pub fn get_bean_mut(&mut self, id: BeanId) -> Option<&mut DefBean> {
        self.beans.get_mut(id)
    }

    /// 根据 FieldId 获取字段定义
    pub fn get_field(&self, id: FieldId) -> Option<&DefField> {
        self.fields.get(id)
    }

    /// 根据 FieldId 获取可变的字段定义
    pub fn get_field_mut(&mut self, id: FieldId) -> Option<&mut DefField> {
        self.fields.get_mut(id)
    }

    /// 根据 full_name 查找 EnumId
    pub fn find_enum_id(&self, full_name: &str) -> Option<EnumId> {
        self.full_name_to_enum_id.get(full_name).copied()
    }

    /// 根据 full_name 获取 Enum 定义
    pub fn find_enum(&self, full_name: &str) -> Option<&DefEnum> {
        self.full_name_to_enum_id
            .get(full_name)
            .and_then(|id| self.enums.get(*id))
    }

    /// 根据 full_name 获取可变的 Enum 定义
    pub fn find_enum_mut(&mut self, full_name: &str) -> Option<&mut DefEnum> {
        self.full_name_to_enum_id
            .get(full_name)
            .and_then(|id| self.enums.get_mut(*id))
    }

    /// 根据 full_name 查找 BeanId
    pub fn find_bean_id(&self, full_name: &str) -> Option<BeanId> {
        self.full_name_to_bean_id.get(full_name).copied()
    }

    /// 根据 full_name 获取 Bean 定义
    pub fn find_bean(&self, full_name: &str) -> Option<&DefBean> {
        self.full_name_to_bean_id
            .get(full_name)
            .and_then(|id| self.beans.get(*id))
    }

    /// 根据 full_name 获取可变的 Bean 定义
    pub fn find_bean_mut(&mut self, full_name: &str) -> Option<&mut DefBean> {
        self.full_name_to_bean_id
            .get(full_name)
            .and_then(|id| self.beans.get_mut(*id))
    }

    /// 删除指定 EnumId 的 Enum 定义，同时清理映射
    pub fn remove_enum(&mut self, id: EnumId) -> Option<DefEnum> {
        if let Some(def_enum) = self.enums.remove(id) {
            let full_name = def_enum.full_name();
            self.full_name_to_enum_id.remove(&full_name);
            Some(def_enum)
        } else {
            None
        }
    }

    /// 删除指定 BeanId 的 Bean 定义，同时清理 full_name 映射和所属字段。
    pub fn remove_bean(&mut self, id: BeanId) -> Option<DefBean> {
        let def_bean = self.beans.remove(id)?;
        self.full_name_to_bean_id.remove(&def_bean.full_name());

        for field_id in def_bean.fields.iter().copied() {
            self.fields.remove(field_id);
        }

        Some(def_bean)
    }

    /// 删除指定 FieldId 的字段定义，并从所属 Bean 的字段列表中移除。
    pub fn remove_field(&mut self, id: FieldId) -> Option<DefField> {
        let def_field = self.fields.remove(id)?;

        if let Some(bean) = self.beans.get_mut(def_field.host) {
            bean.fields.retain(|field_id| *field_id != id);
        }

        Some(def_field)
    }
}

impl Default for DefAssembly {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw_defs::{RawBean, RawEnum, RawField};

    #[test]
    fn add_enum_sets_id_and_name_index() {
        let mut asm = DefAssembly::new();
        let raw = RawEnum {
            module: "demo".to_string(),
            name: "State".to_string(),
            ..Default::default()
        };

        let enum_id = asm.add_enum(DefEnum::new(&raw)).unwrap();
        let def_enum = asm.get_enum(enum_id).unwrap();

        assert_eq!(def_enum.id, enum_id);
        assert_eq!(asm.find_enum_id("demo.State"), Some(enum_id));
    }

    #[test]
    fn add_bean_sets_bean_and_field_ids() {
        let mut asm = DefAssembly::new();
        let raw = RawBean {
            module: "demo".to_string(),
            name: "Role".to_string(),
            fields: vec![RawField {
                name: "id".to_string(),
                type_str: "int".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let def_bean = DefBean::new(&raw, &mut asm);
        let bean_id = asm.add_bean(def_bean).unwrap();
        let def_bean = asm.get_bean(bean_id).unwrap();
        let field_id = def_bean.fields[0];
        let def_field = asm.get_field(field_id).unwrap();

        assert_eq!(def_bean.id, bean_id);
        assert_eq!(def_field.id, field_id);
        assert_eq!(def_field.host, bean_id);
        assert_eq!(asm.find_bean_id("demo.Role"), Some(bean_id));
    }
}
