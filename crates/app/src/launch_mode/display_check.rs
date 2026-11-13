//! Is there a screen to open a window on?
//!
//! Only the systems that can be headless are asked; elsewhere a window is
//! always possible and the window host reports its own failure if not.

/// True when a window could open.
pub fn has_display() -> bool {
    if cfg!(any(target_os = "linux", target_os = "freebsd")) {
        named_display(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()))
    } else {
        true
    }
}

/// The variables that name a display, so the check can be tried without
/// touching the environment.
pub fn named_display(is_set: impl Fn(&str) -> bool) -> bool {
    ["WAYLAND_DISPLAY", "DISPLAY"].iter().any(|name| is_set(name))
}

#[cfg(test)]
include!("display_check.tests.rs");
