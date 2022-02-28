
use serde::{Serialize, Deserialize};
use convert_case::{Case, Casing};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    // #[serde(skip_deserializing)]
    #[serde(skip)]
    pub id: usize,

    #[serde(skip)]
    pub name: String,

    #[serde(default = "Field::default_is_nullable")]
    pub is_nullable: Option<bool>,

    #[serde(rename = "type")]
    pub field_type: tarantool::space::SpaceFieldType,
}
impl Field {
    fn default_is_nullable() -> Option<bool> { None }

    pub fn const_name(&self) -> String {
        format!("FIELD__{}", self.name.clone().to_case(Case::ScreamingSnake))
    }
    pub fn name(&self) -> String {
        // self.name.clone().to_case(Case::Snake)
        self.name.clone()
    }
}