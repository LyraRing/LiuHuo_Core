pub mod raw_bean;
pub mod raw_enum;
pub mod raw_table;

pub use raw_bean::*;
pub use raw_enum::*;
pub use raw_table::*;

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RawAssembly {
    #[serde(default)]
    pub beans: Vec<RawBean>,

    #[serde(default)]
    pub enums: Vec<RawEnum>,

    #[serde(default)]
    pub tables: Vec<RawTable>,

    // #[serde(default)]
    // pub groups: Vec<RawGroup>,

    // #[serde(default)]
    // pub targets: Vec<RawTarget>,

    // #[serde(default)]
    // pub ref_groups: Vec<RawRefGroup>,
    #[serde(default)]
    pub const_aliases: BTreeMap<String, String>,
}
