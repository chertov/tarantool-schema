use serde::{Serialize, Deserialize};
use convert_case::{Case, Casing};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPart {
    #[serde(rename = "field")]
    pub field_name: String,

    #[serde(rename = "type")]
    pub index_field_type: tarantool::index::IndexFieldType,
    pub is_nullable: Option<bool>,

    pub path: Option<String>,

    #[serde(skip)]
    pub field: Option<super::field::Field>,
    #[serde(skip)]
    pub part: Option<tarantool::index::IndexPart>,
}
impl IndexPart {
    pub fn const_path(&self) -> String {
        self.field.clone().unwrap().const_name()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    #[serde(skip)]
    pub name: String,

    #[serde(rename = "type")]
    #[serde(default = "Index::default_index_type")]
    pub index_type: tarantool::index::IndexType,

    #[serde(default = "Index::default_unique")]
    pub unique: bool,

    pub parts: Vec<IndexPart>,
}
impl Index {
    fn default_index_type() -> tarantool::index::IndexType { tarantool::index::IndexType::Tree }
    fn default_unique() -> bool { false }

    pub fn const_name(&self) -> String {
        format!("INDEX__{}", self.name.clone().to_case(Case::ScreamingSnake))
    }
    pub fn name(&self) -> String { self.name.clone() }
}

// SpaceFieldType          IndexFieldType
//      Any           -         -
//      -             -      Varbinary
//      Unsigned      -      Unsigned
//      String        -      String
//      Number        -      Number
//      Double        -      Double
//      Integer       -      Integer
//      Boolean       -      Boolean
//      Decimal       -      Decimal
//      Uuid          -      Uuid
//      Array         -      Array
//      Scalar        -      Scalar