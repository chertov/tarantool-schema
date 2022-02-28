mod cargo;
mod schema;
mod space;
mod field;
mod index;

pub fn generate(schema_path: &std::path::Path, output_path: &std::path::Path, crate_name: Option<String>) -> Result<(), anyhow::Error> {
    let schema_yaml = {
        let data = std::fs::read(schema_path)?;
        String::from_utf8(data)?
    };
    let schema = schema::Schema::new(schema_yaml)?;

    let mut output_path = output_path.to_path_buf();

    match &crate_name {
        Some(crate_name) => {
            output_path.push(crate_name);
            let _ = std::fs::remove_dir_all(&output_path);
            std::fs::create_dir_all(&output_path)?;
            cargo::generate(&output_path, crate_name, &schema, None, None)?;
            output_path.push("src");
        },
        None => {
            let _ = std::fs::remove_dir_all(&output_path);
        }
    }

    std::fs::create_dir_all(&output_path)?;

    schema.generate(output_path.to_path_buf(), crate_name)?;

    Ok(())
}
