//! The name a sender gave an attachment is data from a stranger, and it is
//! made safe before it becomes a file name: only its last part is kept (a
//! `/` or a `\` ends a folder), control characters and the characters that
//! change how a name is drawn or read (direction overrides, zero-width
//! spaces) are taken out, anything a file system or a shell treats specially
//! is replaced, a leading dot, a leading dash and a trailing dot or blank are
//! dropped, and the length is limited with the extension kept. A name that
//! comes to nothing, or to a name Windows reserves for a device, becomes
//! `attachment` with the extension its type suggests.
//!
//! Which types are never handed to the system to open is decided here too:
//! programs, scripts, shortcuts, installers and documents that run code
//! (a double extension such as `invoice.pdf.exe` counts by every part).

use model::media_types::extension_for_type;

/// The most characters of a saved name; `name (2)` and the like come on top.
const MAX_CHARS: usize = 100;

/// The most bytes of a saved name (most file systems allow 255).
const MAX_BYTES: usize = 180;

/// The longest extension that is kept apart when a name is cut short.
const MAX_EXTENSION: usize = 16;

/// What a name with nothing left in it is called.
const NEUTRAL: &str = "attachment";

/// Characters taken out whole: they have no business in a file name and some
/// of them hide what a name really says.
fn is_dropped(ch: char) -> bool {
    ch.is_control()
        || matches!(
            ch,
            '\u{00AD}'
                | '\u{061C}'
                | '\u{180E}'
                | '\u{200B}'..='\u{200F}'
                | '\u{2028}'..='\u{202E}'
                | '\u{2060}'..='\u{206F}'
                | '\u{FEFF}'
                | '\u{FFF9}'..='\u{FFFB}'
                | '\u{E0000}'..='\u{E007F}'
        )
}

/// Characters a file system or a shell reads as something else than a letter
/// of a name: replaced with `_`.
fn is_replaced(ch: char) -> bool {
    matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*' | '/' | '\\')
}

/// The device names Windows will not let a file have, whatever its extension.
fn is_reserved(stem: &str) -> bool {
    let upper = stem.trim_end_matches([' ', '.']).to_ascii_uppercase();
    let numbered = |prefix: &str| upper.strip_prefix(prefix).is_some_and(|rest| matches!(rest.as_bytes(), [b'1'..=b'9']));
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$") || numbered("COM") || numbered("LPT")
}

/// The extension with its dot (`.pdf`), or an empty text.
fn extension_of(name: &str) -> &str {
    match name.rfind('.') {
        Some(at) if at > 0 && name.len() - at <= MAX_EXTENSION + 1 && name[at + 1..].chars().all(|ch| ch.is_ascii_alphanumeric()) => &name[at..],
        _ => "",
    }
}

/// The name to save an attachment under, made from the name the sender gave
/// and its content type (which only supplies an extension for a name that is
/// empty or unusable).
pub fn safe_name(raw: &str, content_type: &str) -> String {
    // Only the last part of a path: both kinds of separator end a folder.
    let last = raw.rsplit(['/', '\\']).next().unwrap_or("");
    let cleaned: String = last.chars().filter(|ch| !is_dropped(*ch)).map(|ch| if is_replaced(ch) { '_' } else { ch }).collect();
    let trimmed = cleaned.trim_matches(|ch: char| ch == '.' || ch.is_whitespace()).trim_start_matches('-');
    let trimmed = trimmed.trim_start_matches(|ch: char| ch == '.' || ch.is_whitespace());
    let stem_end = trimmed.find('.').unwrap_or(trimmed.len());
    if trimmed.is_empty() || is_reserved(&trimmed[..stem_end]) {
        return neutral(content_type);
    }
    shorten(trimmed)
}

fn neutral(content_type: &str) -> String {
    match extension_for_type(content_type) {
        Some(extension) => format!("{NEUTRAL}.{extension}"),
        None => NEUTRAL.to_owned(),
    }
}

/// `name` cut to the limits, its extension kept.
fn shorten(name: &str) -> String {
    if name.chars().count() <= MAX_CHARS && name.len() <= MAX_BYTES {
        return name.to_owned();
    }
    let extension = extension_of(name);
    let stem = &name[..name.len() - extension.len()];
    let mut kept = String::new();
    for ch in stem.chars() {
        if kept.chars().count() + extension.chars().count() >= MAX_CHARS || kept.len() + ch.len_utf8() + extension.len() > MAX_BYTES {
            break;
        }
        kept.push(ch);
    }
    let kept = kept.trim_end_matches(|ch: char| ch == '.' || ch.is_whitespace()).to_owned();
    if kept.is_empty() { format!("{NEUTRAL}{extension}") } else { format!("{kept}{extension}") }
}

/// `name` with ` (2)` before its extension: `report.pdf` becomes
/// `report (2).pdf`.
pub fn numbered(name: &str, number: u32) -> String {
    let extension = extension_of(name);
    let stem = &name[..name.len() - extension.len()];
    format!("{stem} ({number}){extension}")
}

/// Extensions of programs, scripts, shortcuts, installers, disk images and
/// documents that carry code or run in a browser: such a file is saved and
/// never opened by this program.
const NEVER_OPENED: &[&str] = &[
    // Programs and installers.
    "exe", "com", "scr", "pif", "bat", "cmd", "msi", "msp", "mst", "jar", "jnlp", "apk", "app", "command", "desktop", "run", "bin",
    "elf", "out", "appimage", "deb", "rpm", "pkg", "mpkg", "dmg", "dll", "sys", "cpl", "ocx", "drv", "application", "xbap", "gadget",
    // Scripts.
    "sh", "bash", "zsh", "csh", "ksh", "fish", "ps1", "psm1", "psd1", "vbs", "vbe", "js", "jse", "wsf", "wsh", "hta", "py", "pyw",
    "pl", "pm", "rb", "php", "lua", "tcl", "scpt", "scptd", "applescript", "workflow", "action",
    // Shortcuts and settings that start things.
    "lnk", "url", "scf", "reg", "inf", "msc", "terminal", "webloc", "inetloc", "fileloc", "service", "appref-ms", "settingcontent-ms",
    "library-ms", "search-ms", "diagcab", "chm",
    // Disk images, which mount and may start programs.
    "iso", "img", "vhd", "vhdx",
    // Pages that run script in a browser.
    "html", "htm", "xhtml", "xht", "svg", "mht", "mhtml", "swf",
    // Documents with macros.
    "docm", "dotm", "xlsm", "xltm", "xlam", "pptm", "potm", "ppam", "ppsm", "sldm",
];

/// Content types that are programs, scripts or active pages whatever the file
/// is called.
const NEVER_OPENED_TYPES: &[&str] = &[
    "application/x-msdownload",
    "application/x-msdos-program",
    "application/x-dosexec",
    "application/x-executable",
    "application/x-sharedlib",
    "application/x-sh",
    "application/x-shellscript",
    "application/x-csh",
    "application/x-powershell",
    "application/java-archive",
    "application/x-java-archive",
    "application/x-java-jnlp-file",
    "application/hta",
    "application/x-ms-shortcut",
    "application/x-msi",
    "application/vnd.microsoft.portable-executable",
    "application/vnd.android.package-archive",
    "application/x-apple-diskimage",
    "application/x-iso9660-image",
    "application/javascript",
    "application/x-javascript",
    "text/javascript",
    "text/html",
    "application/xhtml+xml",
    "image/svg+xml",
    "text/x-shellscript",
    "text/x-python",
    "text/x-perl",
    "application/x-python-code",
    "application/x-desktop",
];

/// Whether a file with this name and content type must never be handed to the
/// system to open. The name is judged by every part after its first: a double
/// extension is as risky as its worst part (`invoice.pdf.exe`,
/// `invoice.exe.pdf`), and a name that is only an extension (`.exe`) counts.
pub fn is_never_opened(name: &str, content_type: &str) -> bool {
    let last = name.rsplit(['/', '\\']).next().unwrap_or("");
    let parts: Vec<String> = last.trim().split('.').map(|part| part.trim().to_ascii_lowercase()).collect();
    let tail = if last.trim_start().starts_with('.') { &parts[..] } else { parts.get(1..).unwrap_or(&[]) };
    if tail.iter().any(|part| NEVER_OPENED.contains(&part.as_str())) {
        return true;
    }
    let bare = content_type.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    NEVER_OPENED_TYPES.contains(&bare.as_str()) || bare.contains("macroenabled")
}

#[cfg(test)]
include!("attachment_name.tests.rs");
