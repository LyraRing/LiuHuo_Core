use crate::Tags;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RawTable {
    pub module: String,
    pub name: String,
    pub index: String,
    pub value_type: String,

    #[serde(default)]
    pub read_schema_from_file: bool,

    // pub mode: TableMode,
    pub comment: Option<String>,

    #[serde(default)]
    pub tags: Tags,

    #[serde(default)]
    pub groups: Vec<String>,

    #[serde(default)]
    pub input_files: Vec<String>,

    pub output_file: Option<String>,
}

impl RawTable {
    pub fn full_name(&self) -> String {
        if self.module.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.module, self.name)
        }
    }
}
