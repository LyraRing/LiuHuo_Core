use crate::raw_defs::RawGroup;

/// 原始的 Group 定义。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LiuHuoConfig {
    pub config_file_url: String,
    pub groups: Vec<RawGroup>,
    pub args: Vec<String>, // 额外的参数
}
