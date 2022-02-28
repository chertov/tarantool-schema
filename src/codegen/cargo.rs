pub(crate) fn generate(output_path: &std::path::Path, crate_name: &str, schema: &super::schema::Schema, tarantool: Option<String>, tarantool_schema: Option<String>) -> Result<(), anyhow::Error> {
    let mut cargo_path = output_path.to_path_buf();
    cargo_path.push("Cargo.toml");

    let mut src = format!("");
    src += &format!("[package]\n");
    src += &format!("name = \"{}\"\n", crate_name);
    src += &format!("version = \"0.1.0\"\n");
    src += &format!("authors = [\"Chertov Maxim <chertovmv@gmail.com>\"]\n");
    src += &format!("edition = \"2021\"\n");
    src += &format!("\n");
    src += &format!("[dependencies]\n");
    src += &format!("anyhow = \"1\"\n");
    src += &format!("serde = {{ version = \"1\", features = [\"derive\"] }}\n");
    src += &format!("log = {{ version = \"0.4\" }}\n");
    src += &format!("parking_lot = {{ version = \"0.11\" }}\n");
    src += &format!("once_cell = {{ version = \"1.8\" }}\n");
    src += &format!("\n");

    src += &match tarantool {
        Some(tarantool) =>
            format!("# override default tarantool package with value from tarantool_schema::generate() function\n") +
                &format!("tarantool = {tarantool}\n"),
        None => match schema.tarantool() {
            Some(tarantool) =>
                format!("# override default tarantool package with value from YAML schema description\n") +
                    &format!("tarantool = {tarantool}\n"),
            None => format!("tarantool = {{ version = \"*\" }}\n")
        }
    };
    src += &match tarantool_schema {
        Some(tarantool_schema) =>
            format!("# override default tarantool-schema package with value from tarantool_schema::generate() function\n") +
                &format!("tarantool-schema = {tarantool_schema}\n"),
        None => match schema.tarantool_schema() {
            Some(tarantool_schema) =>
                format!("# override default tarantool-schema package with value from YAML schema description\n") +
                    &format!("tarantool-schema = {tarantool_schema}\n"),
            None => format!("tarantool-schema = {{ version = \"*\" }}\n")
        }
    };
    src += &format!("\n");

    let mut dependencies = schema.dependencies();
    if !dependencies.is_empty() {
        src += &format!("#################################################\n");
        src += &format!("### dependencies from YAML schema description ###\n");
        src += &format!("#################################################\n");
    }
    for (name, value) in dependencies {
        src += &format!("{name} = {value}\n");
    }

    std::fs::write(cargo_path, src)?;
    Ok(())
}
