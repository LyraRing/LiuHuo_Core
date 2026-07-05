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
    pub enums: SlotMap<EnumId, DefEnum>,
    pub beans: SlotMap<BeanId, DefBean>,
    pub fields: SlotMap<FieldId, DefField>,

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
    pub fn add_enum(&mut self, def_enum: DefEnum) -> Result<EnumId, String> {
        let enum_id = self.enums.insert(def_enum);
        self.enums[enum_id].id = enum_id;
        let full_name = self.enums[enum_id].full_name();
        self.full_name_to_enum_id.insert(full_name, enum_id);
        Ok(enum_id)
    }

    /// 根据 EnumId 获取 Enum 定义
    pub fn get_enum(&self, id: EnumId) -> Option<&DefEnum> {
        self.enums.get(id)
    }

    /// 根据 EnumId 获取可变的 Enum 定义
    pub fn get_enum_mut(&mut self, id: EnumId) -> Option<&mut DefEnum> {
        self.enums.get_mut(id)
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
}

impl Default for DefAssembly {
    fn default() -> Self {
        Self::new()
    }
}
