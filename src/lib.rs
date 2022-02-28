use anyhow::anyhow;

#[cfg(feature = "codegen")]
pub mod codegen;

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub is_nullable: Option<bool>,
    pub field_type: tarantool::space::SpaceFieldType,
}
impl Field {
    fn format(&self) -> tarantool::space::SpaceFieldFormat {
        tarantool::space::SpaceFieldFormat { name: self.name.to_string(), field_type : self.field_type.clone(), is_nullable: self.is_nullable }
    }
}
#[derive(Debug, Clone)]
pub struct IndexPart {
    pub path: String,
    pub index_field_type: tarantool::index::IndexFieldType,
    pub is_nullable: Option<bool>,
    pub part: tarantool::index::IndexPart,
}
#[derive(Debug, Clone)]
pub struct Index {
    pub name: String,
    pub index_type: tarantool::index::IndexType,
    pub unique: bool,
    pub parts: Vec<IndexPart>,
}
#[derive(Debug, Clone)]
pub struct Space {
    pub name: String,
    pub engine: tarantool::space::SpaceEngineType,
    pub is_local: bool,
    pub temporary: bool,
    pub format: Vec<Field>,
    pub indexes: Vec<Index>,
}
impl Space {
    pub fn create(&self) -> Result<(), anyhow::Error> {
        let mut opts = tarantool::space::SpaceCreateOptions::default();
        opts.if_not_exists = false;
        let mut format = vec![];
        for field in &self.format {
            format.push(field.format());
        }
        opts.format = Some(format);
        let space = tarantool::space::Space::create(&self.name, &opts)?;

        for index in &self.indexes {
            let mut opts = tarantool::index::IndexOptions::default();
            opts.if_not_exists = Some(false);
            opts.unique = Some(index.unique);
            opts.index_type = Some(index.index_type.clone());
            let mut parts = vec![];
            for part in &index.parts {
                parts.push(part.part.clone());
            }
            opts.parts = Some(parts);
            space.create_index(&index.name, &opts)?;
        }

        Ok(())
    }
    pub fn space(&self) -> Result<tarantool::space::Space, anyhow::Error> {
        tarantool::space::Space::find(&self.name).ok_or(anyhow!("Can't find space '{}'", self.name))
    }
    pub fn drop(&self) -> Result<(), anyhow::Error> { self.space()?.drop()?; Ok(()) }
    pub fn truncate(&self) -> Result<(), anyhow::Error> { self.space()?.truncate()?; Ok(()) }

    pub fn verify(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

