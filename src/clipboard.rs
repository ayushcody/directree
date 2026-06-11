use std::io::Write;
use std::process::{Command, Stdio};

/// Copy text to the system clipboard.
/// Tries: pbcopy (macOS) → wl-copy (Wayland) → xclip → xsel → fallback print hint
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    // macOS
    if try_copy("pbcopy", &[], text) {
        return Ok(());
    }
    // Wayland
    if try_copy("wl-copy", &[], text) {
        return Ok(());
    }
    // X11 xclip
    if try_copy("xclip", &["-selection", "clipboard"], text) {
        return Ok(());
    }
    // X11 xsel
    if try_copy("xsel", &["--clipboard", "--input"], text) {
        return Ok(());
    }
    // Windows clip.exe (WSL / native)
    if try_copy("clip.exe", &[], text) {
        return Ok(());
    }

    Err("no clipboard tool found (install pbcopy / xclip / wl-copy / xsel)".into())
}

fn try_copy(cmd: &str, args: &[&str], text: &str) -> bool {
    let child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match child {
        Ok(mut c) => {
            if let Some(stdin) = c.stdin.as_mut() {
                let _ = stdin.write_all(text.as_bytes());
            }
            c.wait().map(|s| s.success()).unwrap_or(false)
        }
        Err(_) => false,
    }
}
