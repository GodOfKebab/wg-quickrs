// build.rs: create macros to embed the version info and build time into the executable
use chrono::SecondsFormat;
use serde_json::Value as JsonValue;
use std::fs;
use std::path::Path;
use toml::Value as TomlValue;

fn main() {
    let frontend_version = fs::read_to_string("web/package.json")
        .ok()
        .and_then(|content| {
            serde_json::from_str::<JsonValue>(&content)
                .ok()
                .and_then(|json| json.get("version")?.as_str().map(String::from))
        })
        .unwrap_or_else(|| "unknown".to_string());

    let backend_version = fs::read_to_string("Cargo.toml")
        .ok()
        .and_then(|content| {
            toml::from_str::<TomlValue>(&content)
                .ok()
                .and_then(|toml| toml.get("package")?.get("version")?.as_str().map(String::from))
        })
        .unwrap_or_else(|| "unknown".to_string());

    let timestamp = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let content = format!(
        r#"
#[macro_export]
macro_rules! backend_version {{
    () => {{
        "v{backend}"
    }};
}}

#[macro_export]
macro_rules! frontend_version {{
    () => {{
        "v{frontend}"
    }};
}}

#[macro_export]
macro_rules! build_timestamp {{
    () => {{
        "{timestamp}"
    }};
}}

#[macro_export]
macro_rules! full_version {{
    () => {{
        concat!(
            "backend: ", backend_version!(), ", ",
            "frontend: ", frontend_version!(), ", ",
            "built: ", build_timestamp!()
        )
    }};
}}
"#,
        backend = backend_version,
        frontend = frontend_version,
        timestamp = timestamp
    );

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("version_macro.rs");
    fs::write(dest_path, content).expect("Could not write version macro");
}
