use anyhow::anyhow;
use serde::{Serialize, Deserialize};
use tarantool::index::{IndexFieldType, IndexType};
use tarantool::space::SpaceFieldType;

use super::field::Field;
use super::index::Index;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    #[serde(skip)]
    pub name: String,

    #[serde(default = "Space::default_engine")]
    engine: tarantool::space::SpaceEngineType,

    #[serde(default = "Space::default_is_local")]
    is_local: bool,

    #[serde(default = "Space::default_temporary")]
    temporary: bool,

    #[serde(default = "Space::default_format")]
    // format: Vec<Field>,
    format: linked_hash_map::LinkedHashMap<String, Field>,

    #[serde(default = "Space::default_indexes")]
    indexes: linked_hash_map::LinkedHashMap<String, Index>,

    #[serde(default = "Space::default_init_data")]
    init_data: Vec<linked_hash_map::LinkedHashMap<String, String>>,

    row_type: Option<String>,
}
impl Space {
    fn default_engine() -> tarantool::space::SpaceEngineType { tarantool::space::SpaceEngineType::Memtx }
    fn default_is_local() -> bool { false }
    fn default_temporary() -> bool { false }
    fn default_format() -> linked_hash_map::LinkedHashMap<String, Field> { linked_hash_map::LinkedHashMap::new() }
    fn default_indexes() -> linked_hash_map::LinkedHashMap<String, Index> { linked_hash_map::LinkedHashMap::new() }
    fn default_init_data() -> Vec<linked_hash_map::LinkedHashMap<String, String>> { vec![] }
}

impl Space {
    pub fn validate(mut self, name: String) -> Result<Self, anyhow::Error> {
        self.name = name;
        for (index, (name, field)) in self.format.iter_mut().enumerate() {
            field.name = name.clone();
            field.id = index + 1;
        }

        for (index_name, index) in &self.indexes {
            for part in &index.parts {
                if let None = self.filed_by_name(&part.field_name) {
                    return Err(anyhow!("Can't find field by name '{}' in index '{}' from space '{}'", part.field_name, index_name, self.name))
                }
            }
        }

        Ok(self.validate_indexes()?)
    }

    fn filed_by_name(&self, name: &str) -> Option<Field> {
        self.format.get(name).map(|field| field.clone())
//        Err(anyhow!("Can't find filed by path '{}' in space '{}'", path, self.name))
    }

    fn validate_indexes(mut self) -> Result<Self, anyhow::Error> {
        for (index_name, index) in &mut self.indexes {
            index.name = index_name.clone();
        }
        let this = self.clone();
        for (index_name, index) in &mut self.indexes {
            index.name = index_name.clone();
            for (part_index, part) in index.parts.iter_mut().enumerate() {
                let field = match this.filed_by_name(&part.field_name) {
                    Some(field) => field,
                    None => return Err(anyhow!("Can't find field by name '{}' in index '{}' from space '{}'", part.field_name, index_name, self.name))
                };
                let type_incorrect = match part.index_field_type {
                    IndexFieldType::Unsigned    => { field.field_type != SpaceFieldType::Unsigned }
                    IndexFieldType::String      => { field.field_type != SpaceFieldType::String }
                    IndexFieldType::Integer     => { field.field_type != SpaceFieldType::Integer }
                    IndexFieldType::Number      => { field.field_type != SpaceFieldType::Number }
                    IndexFieldType::Double      => { field.field_type != SpaceFieldType::Double }
                    IndexFieldType::Decimal     => { field.field_type != SpaceFieldType::Decimal }
                    IndexFieldType::Boolean     => { field.field_type != SpaceFieldType::Boolean }
                    IndexFieldType::Varbinary   => { field.field_type != SpaceFieldType::Integer }
                    IndexFieldType::Uuid        => { field.field_type != SpaceFieldType::Uuid }
                    IndexFieldType::Array       => { field.field_type != SpaceFieldType::Array }
                    IndexFieldType::Scalar      => { field.field_type != SpaceFieldType::Scalar }
                };
                if type_incorrect {
                    return Err(anyhow!("Index part '{}' of index '{}' from space '{}' has incorrect type. Index part type is '{:?}' but field type is '{:?}'", part.field_name, index_name, self.name, part.index_field_type, field.field_type))
                }
                part.part = Some(tarantool::index::IndexPart {
                    field_index: field.id as u32,
                    field_type: part.index_field_type.clone(),
                    collation: None,
                    is_nullable: part.is_nullable,
                    path: part.path.clone(),
                });
                part.field = Some(field);
            }
        }
        Ok(self)
    }

    fn data(&self) -> Result<String, anyhow::Error> {
        let mut str = format!("");
        str += &format!("pub fn init_data() -> Result<(), anyhow::Error> {{\n");

        for (index, row) in self.init_data.iter().enumerate() {
            str += &format!("    space()?.insert(&Row{{");
            for (_, field) in &self.format {
                let val = match row.get(&field.name) {
                    Some(val) => {
                        Some(match field.field_type {
                            SpaceFieldType::String => { format!("\"{}\".to_string()", val) }
                            SpaceFieldType::Uuid => { format!("\"{}\".to_string()", val) }

                            SpaceFieldType::Unsigned => { format!("{}", val) }
                            SpaceFieldType::Number => { format!("{}", val) }
                            SpaceFieldType::Double => { format!("{}", val) }
                            SpaceFieldType::Integer => { format!("{}", val) }
                            SpaceFieldType::Boolean => { format!("{}", val) }
                            SpaceFieldType::Decimal => { format!("{}", val) }
                            space_field_type => {
                                return Err(anyhow!("Incorrect data value in row #{} of space '{}'. Filed '{}' type '{}' doesn't support", index, self.name, field.name, space_field_type));
                            }
                            // SpaceFieldType::Any => {}
                            // SpaceFieldType::Array => {}
                            // SpaceFieldType::Scalar => {}
                        })
                    },
                    None => {
                        match field.is_nullable {
                            Some(false) => { return Err(anyhow!("Incorrect data value in row #{} of space '{}'. Filed '{}' doesn't exist", index, self.name, field.name)); },
                            _ => {},
                        };
                        None
                    }
                };
                let val = match field.is_nullable {
                    Some(true) => {
                        match val {
                            Some(val) => format!("Some({})", val),
                            None => format!("None")
                        }
                    },
                    _ => {
                        match val {
                            Some(val) => format!("{}", val),
                            None => return Err(anyhow!("Incorrect data value in row #{} of space '{}'. Filed '{}' doesn't exist", index, self.name, field.name))
                        }
                    },
                };
                str += &format!(" {}: {},", field.name(), val);
            }

            str += &format!(" }})?;\n");
            // for field in &self.format {
            //     match row.get(&field.name) {
            //         Some(val) => {
            //             args.push(format!("\"{}\".to_string()", val));
            //         }
            //         None => {
            //             if let Some(true) = field.is_nullable {
            //                 args.push(format!("()"));
            //             } else {
            //                 return Err(anyhow!("Can't find '{}' in space '{}' data row '{}'", field.name, self.name, index));
            //             }
            //         }
            //     }
            // }
            // let args = args.join(", ");
            // str += &format!("    space()?.insert(&({},))?;\n", args);
        }
        str += &format!("    Ok(())\n");
        str += &format!("}}\n");
        Ok(str)
    }
}


impl Space {
    pub fn codegen(&self) -> Result<String, anyhow::Error> {
        let mut str = format!("");

        str += r#"
use tarantool::index::{ IndexType, IndexOptions, IndexPart, IndexFieldType, IteratorType };
use tarantool::space::{ Space, SpaceCreateOptions, SpaceFieldFormat, SpaceFieldType };
use tarantool::tuple::{ AsTuple, Tuple };

"#;
        str += &format!("pub const SPACE_NAME: &str = \"{}\";\n", self.name);

        str += &format!("\n");
        for (_, index) in &self.indexes {
            str += &format!("const {}: &str = \"{}\";\n", index.const_name(), index.name());
            // const FIELD_USER_ID: Field = Field { id: 1, name: "user_id", type_: SpaceFieldType::String, index_name: Some(FIELD_USER_ID__INDEX) };
        }

        str += &format!("\n");
        for (_, field) in &self.format {
            str += &format!("const {}: &str = \"{}\";\n", field.const_name(), field.name());
            // const FIELD_USER_ID: Field = Field { id: 1, name: "user_id", type_: SpaceFieldType::String, index_name: Some(FIELD_USER_ID__INDEX) };
        }

        str += &format!("\n");
        for (index, (_, field)) in self.format.iter().enumerate() {
            str += &format!("pub const {}__ID: u32 = {};\n", field.const_name(), index);
        }

        // for field in &self.format {
        //     str += &format!("const FIELD_{}__INDEX: &str = \"{}\";\n", field.field_const_name(), field.name);
        // }

        // str += &format!("\n");
        // str += &format!("\n");
        // for field in &self.format {
        //     str += &format!("const {}__: Field = Field {{ id: {}, name: \"{}\", type_: SpaceFieldType::{:?}, index_name: None }};\n",
        //                     field.const_name(), field.id, field.name, field.field_type );
        //     // const FIELD_USER_ID: Field = Field { id: 1, name: "user_id", type_: SpaceFieldType::String, index_name: Some(FIELD_USER_ID__INDEX) };
        // }
        // str += &format!("\n");
        // str += &format!("\n");
        // str += &format!("pub fn create() -> Result<(), anyhow::Error> {{\n");
        // str += &format!("    let mut opts = SpaceCreateOptions::default();\n");
        // str += &format!("    opts.format = Some(vec![\n");
        //
        // for field in &self.format {
        //     str += &format!("        {}__.format(),\n", field.const_name());
        // }
        // str += &format!("    ]);\n");
        // str += &format!("    let space = Space::create(SPACE_NAME, &opts)?;\n");
        //
        // str += &format!("    Ok(())\n");
        // str += &format!("}}\n");
        // str += &format!("\n");

        str += &format!("\n");
        str += &format!("pub fn space() -> Result<tarantool::space::Space, anyhow::Error> {{ SPACE.read().space() }}\n");

        str += &format!("\n");
        for (_, index) in &self.indexes {
            str += &format!("pub fn {}_index() -> Result<tarantool::index::Index, anyhow::Error> {{ space()?.index({}).ok_or(anyhow::anyhow!(\"Can't find space '{{}}' index '{{}}'\", SPACE_NAME, {})) }}\n", index.name(), index.const_name(), index.const_name());
        }

        str += &format!("\n");
        str += &format!("pub fn create() -> Result<(), anyhow::Error> {{ SPACE.read().create() }}\n");
        str += &format!("pub fn verify() -> Result<(), anyhow::Error> {{ SPACE.read().verify() }}\n");
        str += &format!("pub fn drop() -> Result<(), anyhow::Error> {{ tarantool_schema::Space::drop(&mut SPACE.read()) }}\n");
        str += &format!("pub fn truncate() -> Result<(), anyhow::Error> {{ SPACE.read().truncate() }}\n");
        str += &format!("\n");
        str += &format!("static SPACE: once_cell::sync::Lazy<parking_lot::RwLock<tarantool_schema::Space>> = once_cell::sync::Lazy::new(|| {{\n");

        str += &format!("\n");
        str += &format!("    let mut format = vec![];\n");
        for (_, field) in &self.format {
            str += &format!("    format.push(tarantool_schema::Field {{\n");
            str += &format!("        name: {}.to_string(),\n", field.const_name());
            str += &format!("        field_type: SpaceFieldType::{},\n", field.field_type);
            str += &format!("        is_nullable: {:?},\n", field.is_nullable);
            str += &format!("    }});\n");
        }

        str += &format!("\n");
        str += &format!("    let mut indexes = vec![];\n");
        for (index_name, index) in &self.indexes {
            str += &format!("    indexes.push(tarantool_schema::Index {{\n");
            str += &format!("        name: {}.to_string(),\n", index.const_name());
            str += &format!("        unique: {},\n", index.unique);
            str += &format!("        index_type: IndexType::{:?},\n", index.index_type);
            str += &format!("        parts: {{\n");
            str += &format!("            let mut parts = vec![];\n");
            for part in &index.parts {
                str += &format!("            parts.push(tarantool_schema::IndexPart{{\n");
                str += &format!("                path: {}.to_string(),\n", part.const_path());
                str += &format!("                is_nullable: {:?},\n", part.is_nullable);
                str += &format!("                index_field_type: IndexFieldType::{:?},\n", part.index_field_type);
                str += &format!("                part: IndexPart{{\n");
                let part = part.part.clone().unwrap();
                str += &format!("                    field_index: {},\n", part.field_index);
                str += &format!("                    field_type: IndexFieldType::{:?},\n", part.field_type);
                str += &format!("                    collation: {:?},\n", part.collation);
                str += &format!("                    is_nullable: {:?},\n", part.is_nullable);
                str += &format!("                    path: {},\n", part.path.map(|path| format!("Some(\"{}\".to_string())", path)).unwrap_or("None".to_string()));
                str += &format!("                }},\n");
                str += &format!("            }});\n");
                // str += &format!("            parts.push(tarantool_schema::IndexPart{{path: {}.to_string(), is_nullable: {:?}, index_field_type: IndexFieldType::{:?}}});\n", part.const_path(), part.is_nullable, part.index_field_type);

            }
            str += &format!("            parts\n");
            str += &format!("        }},\n");
            // str += &format!("        field_type: tarantool::space::SpaceFieldType::{},\n", field.field_type);
            // str += &format!("        is_nullable: {:?},\n", field.is_nullable);
            str += &format!("    }});\n");
        }

        str += &format!("\n");
        str += &format!("    let space = tarantool_schema::Space {{\n");
        str += &format!("        name: SPACE_NAME.to_string(),\n");
        str += &format!("        engine: tarantool::space::SpaceEngineType::Memtx,\n");
        str += &format!("        is_local: false,\n");
        str += &format!("        temporary: false,\n");
        str += &format!("        format,\n");
        str += &format!("        indexes,\n");
        str += &format!("    }};\n");
        // str += &format!("    space.create().unwrap();\n");
        str += &format!("    parking_lot::RwLock::new(space)\n");
        str += &format!("}});\n");

        // str += &format!("\n");
        // str += &format!("pub fn drop() -> Result<(), anyhow::Error> {{ space()?.drop()?; Ok(()) }}\n");

        str += &format!("\n");
        str += &format!("#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\n");
        str += &format!("pub struct Row {{\n");
        for (_, field) in &self.format {
            let field_type = match field.field_type {
                SpaceFieldType::Unsigned => { "u64" }
                SpaceFieldType::String => { "String" }
                SpaceFieldType::Number => { "u64" }
                SpaceFieldType::Double => { "f64" }
                SpaceFieldType::Integer => { "i64" }
                SpaceFieldType::Boolean => { "bool" }
                SpaceFieldType::Decimal => { "f64" }
                SpaceFieldType::Uuid => { "String" }
                space_field_type => { return Err(anyhow!("Type '{}' is not supported", space_field_type)); }
                // SpaceFieldType::Scalar => {}
                // SpaceFieldType::Array => {  }
                // SpaceFieldType::Any => {}
            };
            let field_type = match field.is_nullable {
                Some(true) => format!("Option<{}>", field_type),
                Some(false) => format!("{}", field_type),
                None => format!("{}", field_type),
            };
            str += &format!("    pub {}: {},\n", field.name(), field_type);
        }
        str += &format!("}}\n");
        str += &format!("impl tarantool::tuple::AsTuple for Row {{}}\n");
        if let Some(row_type) = &self.row_type {
            str += &format!("impl Row {{\n");
            str += &format!("    pub fn __check() -> Row {{\n");
            str += &format!("        let row = {}::default();\n", row_type);
            str += &format!("        tarantool::tuple::AsTuple::serialize_as_tuple(&row);\n");
            str += &format!("        {}::__info__fields_count__{}();\n", row_type, self.format.len());
            str += &format!("        Row {{\n");
            for (index, (_, field)) in self.format.iter().enumerate() {
                str += &format!("            {}: {}::__info__field_{}__{}(),\n", field.name, row_type, index, field.name);
            }
            str += &format!("        }}\n");
            str += &format!("    }}\n");
            str += &format!("}}\n");
        }

        str += &format!("\n");
        str += &self.data()?;
        //
        // static SPACE: once_cell::sync::Lazy<parking_lot::RwLock<tarantool_schema::Space>> = once_cell::sync::Lazy::new(|| {
        //     let mut space = { todo!() }; // tarantool_schema::Space::new();
        //     // m.insert(13, "Spica".to_string());
        //     // m.insert(74, "Hoyten".to_string());
        //     parking_lot::RwLock::new(space)
        // });
        Ok(str)
    }
}