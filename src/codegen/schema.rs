use serde::{ Serialize, Deserialize};

use super::space::Space;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Schema {
    tarantool: Option<String>,
    tarantool_schema: Option<String>,

    spaces: linked_hash_map::LinkedHashMap<String, Space>,
    dependencies: linked_hash_map::LinkedHashMap<String, String>,
}

impl Schema {
    pub fn new(schema_yaml: String) -> Result<Self, anyhow::Error> {
        let schema: Schema = serde_yaml::from_str(&schema_yaml)?;
        schema.validate()
    }
    fn validate(mut self) -> Result<Self, anyhow::Error> {
        let mut spaces =  linked_hash_map::LinkedHashMap::new();
        for (name, mut space) in self.spaces.clone() {
            spaces.insert(name.clone(), space.validate(name)?);
        }
        self.spaces = spaces;

        Ok(self)
    }

    pub(crate) fn dependencies(&self) -> linked_hash_map::LinkedHashMap<String, String> {
        self.dependencies.clone()
    }
    pub(crate) fn tarantool(&self) -> Option<String> {
        self.tarantool.clone()
    }
    pub(crate) fn tarantool_schema(&self) -> Option<String> {
        self.tarantool_schema.clone()
    }

    fn generate_mod(&self) -> Result<String, anyhow::Error> {
        let mut str = format!("");
        Ok(str)
    }

    fn generate_spaces(&self) -> Result<Vec<(String, String)>, anyhow::Error> {
        let mut files = vec![];
        for (name, space) in &self.spaces {
            files.push((name.clone(), space.codegen()?))
        }
        Ok(files)
    }
}

impl Schema {
    pub(crate) fn generate(&self, output_path: std::path::PathBuf, crate_name: Option<String>) -> Result<(), anyhow::Error> {
        let mut mod_rs = format!("");

        let mod_rs_path = {
            let mut path = output_path.clone();
            match crate_name {
                Some(crate_name) => {
                    path.push("lib.rs");
                    mod_rs += &format!("\n");
                },
                None => {
                    path.push("mod.rs");
                }
            }
            path
        };

        mod_rs += &format!("pub mod spaces;\n");

        let mut spaces_mod_rs = format!("");
        let spaces_mod_rs_path = {
            let mut path = output_path.clone();
            path.push("spaces");
            std::fs::create_dir_all(&path)?;
            path.push("mod.rs");
            path
        };

        let spaces = self.generate_spaces()?;

        for (space_name, code) in &spaces {
            spaces_mod_rs += &format!("pub mod {};\n", space_name);

            let space_path = {
                let mut path = output_path.clone();
                path.push("spaces");
                std::fs::create_dir_all(&path)?;
                path.push(format!("{}.rs", space_name));
                path
            };
            std::fs::write(space_path, code)?;
        }

//     spaces_mod_rs += r#"
// struct Field {
//     id: u32,
//     name: &'static str,
//     type_: tarantool::space::SpaceFieldType,
//     index_name: Option<&'static  str>
// }
//
// impl Field {
//     fn id(&self) -> u32 { self.id }
//     fn name(&self) -> String { self.name.to_string() }
//     fn format(&self) -> tarantool::space::SpaceFieldFormat {
//         tarantool::space::SpaceFieldFormat{ name: self.name.to_string(), field_type : self.type_.clone(), is_nullable: None }
//     }
// }
// "#;

        spaces_mod_rs += &format!("\n");
        spaces_mod_rs += &format!("pub fn create() -> Result<(), anyhow::Error> {{\n");
        for (space_name, code) in &spaces {
            spaces_mod_rs += &format!("    {}::create()?;\n", space_name);
            // spaces_mod_rs += &format!("    if let Err(err) = {}::init() {{ log::error!(\"{}::init {{:?}}\", err); }}\n", space_name, space_name);
            // spaces_mod_rs += &format!("    if let Err(err) = {}::init_data() {{ log::error!(\"{}::init_data {{:?}}\", err); }}\n", space_name, space_name);
        }
        spaces_mod_rs += &format!("\n");
        spaces_mod_rs += &format!("    init_data()?;\n");
        spaces_mod_rs += &format!("    verify()?;\n");
        spaces_mod_rs += &format!("    Ok(())\n");
        spaces_mod_rs += &format!("}}\n");

        spaces_mod_rs += &format!("\n");
        spaces_mod_rs += &format!("pub fn init_data() -> Result<(), anyhow::Error> {{\n");
        for (space_name, code) in &spaces {
            spaces_mod_rs += &format!("    {}::init_data()?;\n", space_name);
        }
        spaces_mod_rs += &format!("\n");
        spaces_mod_rs += &format!("    Ok(())\n");
        spaces_mod_rs += &format!("}}\n");

        spaces_mod_rs += &format!("\n");
        spaces_mod_rs += &format!("pub fn verify() -> Result<(), anyhow::Error> {{\n");
        for (space_name, code) in &spaces {
            spaces_mod_rs += &format!("    {}::verify()?;\n", space_name);
        }
        spaces_mod_rs += &format!("\n");
        spaces_mod_rs += &format!("    Ok(())\n");
        spaces_mod_rs += &format!("}}\n");

        spaces_mod_rs += &format!("\n");
        spaces_mod_rs += &format!("pub fn drop() -> Result<(), anyhow::Error> {{\n");
        for (space_name, code) in &spaces {
            spaces_mod_rs += &format!("    {}::drop()?;\n", space_name);
        }
        spaces_mod_rs += &format!("\n");
        spaces_mod_rs += &format!("    Ok(())\n");
        spaces_mod_rs += &format!("}}\n");

        std::fs::write(mod_rs_path, mod_rs)?;
        std::fs::write(spaces_mod_rs_path, spaces_mod_rs)?;

        Ok(())
    }
}