//! Faster rebuilds of the user's animation project during preview reloads.
//!
//! Every reload runs the animation command (by default `cargo run ...`), which for a one file
//! crate is dominated by debuginfo generation and linking. This module sets environment
//! variables on that command that make the dev build cheaper, so it works for any project
//! without editing its `Cargo.toml` or entering a special shell:
//!
//! - `CARGO_PROFILE_DEV_DEBUG=line-tables-only`: panic backtraces keep their line numbers,
//!   the (large) variable debuginfo is not generated.
//! - `CARGO_PROFILE_DEV_SPLIT_DEBUGINFO=unpacked`: keeps debuginfo out of the link step.
//! - `CARGO_TARGET_<TRIPLE>_LINKER`: a tiny wrapper script that links with `mold` when it is
//!   installed. Unlike `RUSTFLAGS` this does not replace flags the project sets itself.
//!
//! A setting is never applied if the user already set it, either in the environment or (for
//! the linker) in a cargo config file. Set `SCAL_FAST_BUILD=0` to disable all of this.
//! Note that cargo environment profile settings take precedence over the project's own
//! `[profile.dev]`.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Applies the fast build settings to `cmd`.
pub fn apply(cmd: &mut Command) {
    if std::env::var("SCAL_FAST_BUILD").is_ok_and(|v| v == "0") {
        return;
    }
    set_unless_present(cmd, "CARGO_PROFILE_DEV_DEBUG", "line-tables-only");
    set_unless_present(cmd, "CARGO_PROFILE_DEV_SPLIT_DEBUGINFO", "unpacked");
    if let Some((var, script)) = mold_linker() {
        set_unless_present(cmd, &var, &script.to_string_lossy());
    }
}

fn set_unless_present(cmd: &mut Command, key: &str, value: &str) {
    if std::env::var_os(key).is_none() {
        cmd.env(key, value);
    }
}

/// The environment variable and wrapper script that make cargo link with mold, if mold is
/// available and the user has not configured a linker.
fn mold_linker() -> Option<(String, PathBuf)> {
    // The linker variable is per target triple; only the common Linux hosts are handled.
    let arch = match std::env::consts::ARCH {
        arch @ ("x86_64" | "aarch64") => arch,
        _ => return None,
    };
    if !cfg!(all(target_os = "linux", target_env = "gnu")) {
        return None;
    }
    let var = format!(
        "CARGO_TARGET_{}_UNKNOWN_LINUX_GNU_LINKER",
        arch.to_uppercase()
    );
    if std::env::var_os(&var).is_some() || linker_configured() {
        return None;
    }
    if !on_path("ld.mold") && !on_path("mold") {
        log::debug!("fast build: mold not found, using the default linker");
        return None;
    }
    static SCRIPT: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    let script = SCRIPT.get_or_init(write_linker_script).clone()?;
    Some((var, script))
}

fn write_linker_script() -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let path = std::env::temp_dir().join(format!("scal-mold-linker-{}.sh", current_uid()));
    let script = "#!/bin/sh\nexec cc -fuse-ld=mold \"$@\"\n";
    let up_to_date = std::fs::read_to_string(&path).is_ok_and(|existing| existing == script);
    if !up_to_date {
        // Write to a private name first so cargo never sees a half written script.
        let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
        std::fs::write(&tmp, script).ok()?;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o700)).ok()?;
        std::fs::rename(&tmp, &path).ok()?;
    }
    Some(path)
}

/// Owner of the script, so users sharing `/tmp` don't fight over (or run) each other's file.
fn current_uid() -> u32 {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self")
        .map(|m| m.uid())
        .unwrap_or(0)
}

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
}

/// Whether any cargo config file that applies to the current directory mentions a linker.
fn linker_configured() -> bool {
    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        for dir in cwd.ancestors() {
            files.push(dir.join(".cargo/config.toml"));
            files.push(dir.join(".cargo/config"));
        }
    }
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".cargo")));
    if let Some(home) = cargo_home {
        files.push(home.join("config.toml"));
        files.push(home.join("config"));
    }
    files
        .iter()
        .filter_map(|f| std::fs::read_to_string(f).ok())
        .any(|content| content.contains("linker"))
}
