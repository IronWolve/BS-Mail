//! Carries the display name, its slug and the tagline from the one manifest
//! field into the program, so no source file spells any of them and a rename
//! of the project folder changes nothing.

use std::path::PathBuf;

fn main() {
    let manifest = workspace_manifest();
    println!("cargo::rerun-if-changed={}", manifest.display());
    let text = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|error| panic!("the workspace manifest must be readable: {error}"));
    let root: toml::Value = toml::from_str(&text)
        .unwrap_or_else(|error| panic!("the workspace manifest must parse: {error}"));
    let app = root
        .get("workspace")
        .and_then(|workspace| workspace.get("metadata"))
        .and_then(|metadata| metadata.get("app"))
        .expect("the workspace manifest needs a [workspace.metadata.app] table");
    let name = field(app, "display_name");
    println!("cargo::rustc-env=APP_DISPLAY_NAME={name}");
    println!("cargo::rustc-env=APP_TAGLINE={}", field(app, "tagline"));
    println!("cargo::rustc-env=APP_SLUG={}", slug(&name));
}

fn workspace_manifest() -> PathBuf {
    let here = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest directory");
    PathBuf::from(here).join("..").join("..").join("Cargo.toml")
}

fn field(app: &toml::Value, key: &str) -> String {
    app.get(key)
        .and_then(toml::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| panic!("[workspace.metadata.app] needs a non-empty {key}"))
        .to_string()
}

/// The display name as a file name: lowercase, with one hyphen for every run
/// of characters that do not belong in one.
fn slug(name: &str) -> String {
    let mut out = String::new();
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            out.push(character.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}
