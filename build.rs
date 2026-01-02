fn main() {
    println!("cargo::rerun-if-changed=schema.graphql");

    cynic_codegen::register_schema("linear")
        .from_sdl_file("schema.graphql")
        .expect("Failed to register schema")
        .as_default()
        .unwrap();
}
