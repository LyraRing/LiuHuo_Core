pub mod def_bean;
pub mod def_enum;

pub use def_bean::*;
pub use def_enum::*;

use slotmap::new_key_type;
new_key_type! {
    pub struct EnumId;
    pub struct BeanId;
    pub struct FieldId;
    pub struct TableId;
}

pub struct DefAssembly {
    pub enums: slotmap::SlotMap<EnumId, DefEnum>,
    pub beans: slotmap::SlotMap<BeanId, DefBean>,
    pub fields: slotmap::SlotMap<FieldId, DefField>,
}
