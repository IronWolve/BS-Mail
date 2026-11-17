//! Carries the display name, its slug, the tagline and the home page from the
//! manifest's one table into the program, so no source file spells any of
//! them and a rename of the project folder changes nothing. It also finds the
//! font folder, the one place that is named by a build setting rather than by
//! the code.

use std::path::{Path, PathBuf};

/// The faces the program draws with, by what each one is for.
const FACES: [&str; 5] = [
    "text-regular.ttf",
    "text-bold.ttf",
    "text-italic.ttf",
    "text-bold-italic.ttf",
    "emoji.ttf",
];

fn main() {
    let manifest = workspace_manifest();
    export_fonts();
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
    println!("cargo::rustc-env=APP_HOMEPAGE={}", field(app, "homepage"));
    println!("cargo::rustc-env=APP_SLUG={}", slug(&name));
    embed_windows_resources(&name, &slug(&name), &field(app, "copyright"));
}

/// On Windows the program carries its icon and its version information as
/// resources, which Explorer and the task bar read. The resource script is
/// written here from the manifest and compiled with the target's resource
/// compiler: the one `WINDRES` names, or the cross compiler's own. Other
/// systems take their icon from elsewhere and do nothing here.
fn embed_windows_resources(name: &str, slug: &str, copyright: &str) {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    println!("cargo::rerun-if-env-changed=WINDRES");
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("gnu") {
        println!("cargo::warning=the icon and version resources are only embedded by the gnu target");
        return;
    }
    let icon = crate_dir().join("..").join("..").join("assets").join("icon").join("icon.ico");
    assert!(icon.is_file(), "the build needs the program icon");
    println!("cargo::rerun-if-changed={}", icon.display());
    let version = std::env::var("CARGO_PKG_VERSION").expect("cargo sets the version");
    let mut parts = version.split(|c: char| !c.is_ascii_digit()).filter_map(|part| part.parse::<u16>().ok());
    let numbers = format!("{},{},{},0", parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0));
    let script = format!(
        "1 ICON \"{icon}\"\n\
         1 VERSIONINFO\nFILEVERSION {numbers}\nPRODUCTVERSION {numbers}\nFILEOS 0x40004\nFILETYPE 0x1\n\
         BEGIN\n BLOCK \"StringFileInfo\"\n BEGIN\n  BLOCK \"040904B0\"\n  BEGIN\n\
         \x20  VALUE \"FileDescription\", \"{name}\"\n   VALUE \"FileVersion\", \"{version}\"\n\
         \x20  VALUE \"InternalName\", \"{slug}\"\n   VALUE \"LegalCopyright\", \"{copyright}\"\n\
         \x20  VALUE \"OriginalFilename\", \"{slug}.exe\"\n   VALUE \"ProductName\", \"{name}\"\n\
         \x20  VALUE \"ProductVersion\", \"{version}\"\n  END\n END\n\
         \x20BLOCK \"VarFileInfo\"\n BEGIN\n  VALUE \"Translation\", 0x409, 1200\n END\nEND\n",
        icon = icon.display().to_string().replace('\\', "/"),
    );
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets the output folder"));
    let (rc, object) = (out.join("app.rc"), out.join("app_resources.o"));
    std::fs::write(&rc, script).expect("the resource script is written");
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let program = std::env::var("WINDRES").unwrap_or_else(|_| match arch.as_str() {
        "x86" => "i686-w64-mingw32-windres".to_string(),
        _ => "x86_64-w64-mingw32-windres".to_string(),
    });
    let status = std::process::Command::new(&program)
        .arg("--input")
        .arg(&rc)
        .arg("--output")
        .arg(&object)
        .arg("--output-format=coff")
        .status()
        .unwrap_or_else(|error| panic!("the resource compiler {program} did not start: {error}"));
    assert!(status.success(), "the resource compiler {program} failed");
    println!("cargo::rustc-link-arg-bins={}", object.display());
}

fn workspace_manifest() -> PathBuf {
    crate_dir().join("..").join("..").join("Cargo.toml")
}

fn crate_dir() -> PathBuf {
    let here = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest directory");
    PathBuf::from(here)
}

/// Tells the program where the faces are, so one `include` reaches them
/// wherever the build setting puts them.
fn export_fonts() {
    let folder = font_folder();
    for face in FACES {
        let file = folder.join(face);
        println!("cargo::rerun-if-changed={}", file.display());
        assert!(
            file.is_file(),
            "the build needs the font file {face}; see the project's build notes"
        );
    }
    println!("cargo::rustc-env=APP_FONT_DIR={}", folder.display());
}

/// The folder the fonts live in, from the one build setting that says so.
fn font_folder() -> PathBuf {
    let crate_dir = crate_dir();
    let project = crate_dir.join("..").join("..").join("..");
    let settings = project.join("build-support").join("build-settings.toml");
    println!("cargo::rerun-if-changed={}", settings.display());
    match setting(&settings, "font_source").as_deref() {
        Some("tracked") => crate_dir.join("..").join("..").join("assets").join("fonts"),
        _ => project.join("build-support").join("fonts"),
    }
}

/// One string setting of the build settings file, if the file is there.
fn setting(path: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let root: toml::Value = toml::from_str(&text).ok()?;
    root.get(key)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
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
