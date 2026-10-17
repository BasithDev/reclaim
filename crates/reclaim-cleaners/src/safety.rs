//! The only code allowed to delete anything. Enforces the protected-path deny list,
//! defaults to Trash/Recycle Bin, and runs privileged commands via pkexec (Linux) or UAC (Windows).

use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::util::{is_link_like, path_size};
use crate::{CleanError, Result};

/// Default timeout for external commands.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
/// Timeout for slow tools (docker, snap).
pub const LONG_TIMEOUT: Duration = Duration::from_secs(300);
/// Extra time granted when a command is elevated (the user has to type a password).
const ELEVATION_GRACE: Duration = Duration::from_secs(180);
/// Windows exit code for "the operation was cancelled by the user" (ERROR_CANCELLED).
const WIN_CANCELLED: i32 = 1223;

// ---------------------------------------------------------------------------------------------
// Protected paths
// ---------------------------------------------------------------------------------------------

/// Resolves symlinks in the parent directory (never the final component, so a symlink is removed
/// as a link and never followed). Returns None for relative paths or paths containing `..`.
fn normalize(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute() {
        return None;
    }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return None;
    }
    let lexical: PathBuf = path.components().filter(|c| !matches!(c, Component::CurDir)).collect();
    match (lexical.parent(), lexical.file_name()) {
        (Some(parent), Some(name)) => match parent.canonicalize() {
            Ok(p) => Some(strip_verbatim(p).join(name)),
            Err(_) => Some(lexical),
        },
        _ => Some(lexical),
    }
}

/// Windows canonicalize() returns `\\?\C:\...`; drop the prefix so comparisons work.
fn strip_verbatim(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        return PathBuf::from(rest.to_string());
    }
    p
}

/// Normal path components (prefix and root dropped), case-folded on Windows.
fn comps(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => {
                let s = s.to_string_lossy().into_owned();
                Some(if cfg!(windows) { s.to_lowercase() } else { s })
            }
            _ => None,
        })
        .collect()
}

fn starts_with_comps(c: &[String], prefix: &[String]) -> bool {
    c.len() >= prefix.len() && c[..prefix.len()] == *prefix
}

fn s(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| p.to_string()).collect()
}

fn has_prefix(c: &[String], prefix: &[&str]) -> bool {
    starts_with_comps(c, &s(prefix))
}

/// True if `path` lies strictly inside `root` (path != root).
pub fn is_strictly_within(path: &Path, root: &Path) -> bool {
    let (Some(p), Some(r)) = (normalize(path), normalize(root)) else { return false };
    let (pc, rc) = (comps(&p), comps(&r));
    pc.len() > rc.len() && starts_with_comps(&pc, &rc)
}

fn unix_rules(c: &[String]) -> bool {
    let first = c[0].as_str();
    match first {
        "bin" | "boot" | "dev" | "etc" | "lib" | "lib32" | "lib64" | "libx32" | "proc" | "run" | "sbin"
        | "sys" | "usr" | "snap" | "sysroot" | "efi" | "lost+found" => true,
        "var" => {
            let allowed = (has_prefix(c, &["var", "log"]) && c.len() >= 3 && !has_prefix(c, &["var", "log", "journal"]))
                || (has_prefix(c, &["var", "cache"]) && c.len() >= 4)
                || (has_prefix(c, &["var", "lib", "systemd", "coredump"]) && c.len() >= 5)
                || (has_prefix(c, &["var", "tmp"]) && c.len() >= 3);
            !allowed
        }
        "home" => c.len() <= 3 || (c.len() == 4 && c[2] == ".local" && (c[3] == "share" || c[3] == "state")),
        "root" => c.len() <= 2,
        "mnt" | "media" => c.len() <= 3,
        "opt" | "srv" => c.len() <= 2,
        _ => false,
    }
}

fn windows_rules(c: &[String]) -> bool {
    let first = c[0].as_str();
    match first {
        "windows" => {
            let allowed = (has_prefix(c, &["windows", "temp"]) && c.len() >= 3)
                || (has_prefix(c, &["windows", "softwaredistribution", "download"]) && c.len() >= 4)
                || (has_prefix(c, &["windows", "minidump"]) && c.len() >= 3)
                || (has_prefix(
                    c,
                    &[
                        "windows",
                        "serviceprofiles",
                        "networkservice",
                        "appdata",
                        "local",
                        "microsoft",
                        "windows",
                        "deliveryoptimization",
                        "cache",
                    ],
                ) && c.len() >= 10);
            !allowed
        }
        "program files" | "program files (x86)" | "programfiles" => true,
        "programdata" => !(has_prefix(c, &["programdata", "nvidia corporation", "nv_cache"]) && c.len() >= 4),
        "$recycle.bin" | "system volume information" | "recovery" | "boot" | "efi" | "msocache"
        | "documents and settings" | "config.msi" => true,
        "users" => {
            if c.len() <= 2 {
                return true;
            }
            if matches!(c[1].as_str(), "public" | "default" | "all users" | "default user") && c.len() <= 3 {
                return true;
            }
            // C:\Users\name\<top-level>, C:\Users\name\AppData\{Local,Roaming,LocalLow}
            c.len() <= 3 || (c[2] == "appdata" && c.len() <= 4)
        }
        _ => false,
    }
}

/// True for paths that must never be removed: filesystem roots, OS directories
/// (/, /bin, /boot, /etc, /usr, C:\Windows, C:\Program Files, ...), the user's home itself, etc.
/// Relative paths, paths with `..` and anything that cannot be normalized are protected.
pub fn is_protected(path: &Path) -> bool {
    let Some(p) = normalize(path) else { return true };
    let c = comps(&p);
    // Filesystem roots, drive roots and UNC share roots.
    if c.len() <= 1 {
        return true;
    }
    if cfg!(windows) {
        if windows_rules(&c) {
            return true;
        }
    } else if unix_rules(&c) {
        return true;
    }
    // The user's home, its ancestors, its top-level entries and sensitive subtrees.
    if let Some(home) = dirs::home_dir() {
        let home = normalize(&home).unwrap_or(home);
        let hc = comps(&home);
        if !hc.is_empty() {
            if c.len() <= hc.len() && starts_with_comps(&hc, &c) {
                return true; // the home dir itself or one of its ancestors
            }
            if starts_with_comps(&c, &hc) {
                let rel = &c[hc.len()..];
                if rel.len() <= 1 {
                    return true;
                }
                if matches!(rel[0].as_str(), ".ssh" | ".gnupg" | ".password-store" | ".aws" | ".kube") {
                    return true;
                }
                if rel.len() == 2 && rel[0] == ".local" && matches!(rel[1].as_str(), "share" | "state") {
                    return true;
                }
                if cfg!(windows) && rel[0] == "appdata" && rel.len() <= 2 {
                    return true;
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// Removal
// ---------------------------------------------------------------------------------------------

fn io_err(path: &Path, source: std::io::Error) -> CleanError {
    CleanError::Io { path: path.to_string_lossy().into_owned(), source }
}

/// Moves to Trash/Recycle Bin, or deletes when `permanent`. Refuses protected paths.
/// Returns bytes freed (allocated size measured before removal).
pub fn remove_path(path: &Path, permanent: bool) -> Result<u64> {
    if is_protected(path) {
        return Err(CleanError::Protected(path.to_string_lossy().into_owned()));
    }
    let md = std::fs::symlink_metadata(path).map_err(|e| io_err(path, e))?;
    let size = path_size(path);
    if permanent {
        remove_permanent(path, &md)?;
    } else {
        trash::delete(path).map_err(|e| CleanError::Other(format!("could not move {} to trash: {e}", path.display())))?;
    }
    Ok(size)
}

fn remove_permanent(path: &Path, md: &std::fs::Metadata) -> Result<()> {
    // remove_file removes a symlink itself; remove_dir_all never follows symlinks.
    let is_real_dir = md.is_dir() && !is_link_like(md);
    let attempt = |p: &Path| if is_real_dir { std::fs::remove_dir_all(p) } else { std::fs::remove_file(p) };
    match attempt(path) {
        Ok(()) => Ok(()),
        Err(e) if cfg!(windows) && e.kind() == std::io::ErrorKind::PermissionDenied && !is_real_dir => {
            // Read-only files on Windows: clear the flag and retry once.
            let mut perms = md.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            let _ = std::fs::set_permissions(path, perms);
            attempt(path).map_err(|e| io_err(path, e))
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied && is_real_dir => {
            // Read-only folders we own (e.g. Go/dotslash caches use mode 555): make them
            // writable, then retry once. Anything owned by another user still fails.
            make_tree_writable(path);
            attempt(path).map_err(|e| io_err(path, e))
        }
        Err(e) => Err(io_err(path, e)),
    }
}

/// Grants the owner write access to every directory under `path` (never following links),
/// so a permanent delete can remove read-only trees the user owns.
fn make_tree_writable(path: &Path) {
    for entry in walkdir::WalkDir::new(path).follow_links(false).into_iter().flatten() {
        let Ok(md) = entry.path().symlink_metadata() else { continue };
        if !md.is_dir() || is_link_like(&md) {
            continue;
        }
        let mut perms = md.permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(perms.mode() | 0o700);
        }
        #[cfg(windows)]
        {
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
        }
        let _ = std::fs::set_permissions(entry.path(), perms);
    }
}

/// True if the current user owns `path` (always true on Windows, where ACLs decide instead).
/// Used to keep other users' / root's files out of non-admin previews, since they can't be removed.
pub fn owned_by_current_user(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        static UID: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
        let me = *UID.get_or_init(|| std::fs::metadata("/proc/self").ok().map(|m| m.uid()));
        match (me, path.symlink_metadata()) {
            (Some(me), Ok(md)) => md.uid() == me,
            _ => true,
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        true
    }
}

/// Like `remove_path`, but additionally requires the path to lie strictly inside one of the
/// cleaner's own `roots`.
pub fn remove_path_in(path: &Path, roots: &[PathBuf], permanent: bool) -> Result<u64> {
    if !roots.iter().any(|r| is_strictly_within(path, r)) {
        return Err(CleanError::Protected(format!("{} (outside the cleaner's folders)", path.display())));
    }
    remove_path(path, permanent)
}

/// Outcome of an elevated batch removal.
#[derive(Debug, Default)]
pub struct ElevatedRemoval {
    pub freed: u64,
    pub removed: u64,
    pub errors: Vec<String>,
}

/// Permanently removes many paths with ONE elevated invocation (one password prompt).
/// Every path is checked against the deny list and `roots`; links are removed as links.
pub fn remove_paths_elevated(paths: &[PathBuf], roots: &[PathBuf]) -> Result<ElevatedRemoval> {
    let mut out = ElevatedRemoval::default();
    let mut valid: Vec<(PathBuf, u64)> = Vec::new();
    for p in paths {
        if is_protected(p) || !roots.iter().any(|r| is_strictly_within(p, r)) {
            out.errors.push(format!("refused protected path {}", p.display()));
            continue;
        }
        let Some(norm) = normalize(p) else { continue };
        if std::fs::symlink_metadata(&norm).is_err() {
            continue;
        }
        if norm.to_str().is_none() || (cfg!(windows) && !ps_safe(&norm)) {
            out.errors.push(format!("skipped path with unsupported characters: {}", p.display()));
            continue;
        }
        valid.push((norm.clone(), path_size(&norm)));
    }
    if valid.is_empty() {
        return Ok(out);
    }
    for chunk in valid.chunks(4000) {
        let list: Vec<&Path> = chunk.iter().map(|(p, _)| p.as_path()).collect();
        run_elevated_removal(&list)?;
    }
    for (p, size) in &valid {
        if std::fs::symlink_metadata(p).is_err() {
            out.removed += 1;
            out.freed = out.freed.saturating_add(*size);
        } else {
            out.errors.push(format!("could not remove {}", p.display()));
        }
    }
    Ok(out)
}

#[cfg(unix)]
fn run_elevated_removal(paths: &[&Path]) -> Result<()> {
    let mut args: Vec<&str> = vec!["-rf", "--one-file-system", "--"];
    args.extend(paths.iter().filter_map(|p| p.to_str()));
    run_command_timeout("rm", &args, true, LONG_TIMEOUT).map(|_| ())
}

#[cfg(not(unix))]
fn run_elevated_removal(paths: &[&Path]) -> Result<()> {
    let list: Vec<String> = paths.iter().map(|p| ps_quote(&p.to_string_lossy())).collect();
    let script = format!(
        "$ErrorActionPreference = 'SilentlyContinue'; Remove-Item -LiteralPath {} -Recurse -Force",
        list.join(",")
    );
    run_elevated_powershell(&script, LONG_TIMEOUT).map(|_| ())
}

// ---------------------------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------------------------

/// Runs a command, elevated if `admin` (pkexec on Linux, UAC prompt on Windows).
/// Returns stdout. Err(ElevationDenied) if the user cancels the prompt.
pub fn run_command(program: &str, args: &[&str], admin: bool) -> Result<String> {
    run_command_timeout(program, args, admin, DEFAULT_TIMEOUT)
}

/// Same as `run_command` with an explicit timeout (use `LONG_TIMEOUT` for docker/snap).
/// Arguments are passed as an array; no shell is involved.
pub fn run_command_timeout(program: &str, args: &[&str], admin: bool, timeout: Duration) -> Result<String> {
    if !admin {
        let out = exec(program, args, timeout)?;
        return check(program, args, out);
    }
    #[cfg(unix)]
    {
        let mut full: Vec<&str> = vec![program];
        full.extend_from_slice(args);
        let out = exec("pkexec", &full, timeout + ELEVATION_GRACE)?;
        // pkexec: 126 = not authorized / dialog dismissed, 127 = authentication failed or cancelled.
        if matches!(out.code, Some(126) | Some(127)) {
            return Err(CleanError::ElevationDenied);
        }
        check(program, args, out)
    }
    #[cfg(not(unix))]
    {
        let mut parts = vec![ps_quote(program)];
        parts.extend(args.iter().map(|a| ps_quote(a)));
        let script = format!("& {}; exit $LASTEXITCODE", parts.join(" "));
        run_elevated_powershell(&script, timeout)
    }
}

struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn check(program: &str, args: &[&str], out: Output) -> Result<String> {
    if out.code == Some(0) {
        return Ok(out.stdout);
    }
    let mut msg = out.stderr.trim().to_string();
    if msg.is_empty() {
        msg = out.stdout.trim().to_string();
    }
    if msg.len() > 500 {
        let start = msg.len() - 500;
        let start = (start..msg.len()).find(|i| msg.is_char_boundary(*i)).unwrap_or(0);
        msg = msg[start..].to_string();
    }
    if msg.is_empty() {
        msg = format!("exited with status {}", out.code.map_or("unknown".to_string(), |c| c.to_string()));
    }
    Err(CleanError::Command { command: display_command(program, args), message: msg })
}

/// Human-readable command line for previews and errors.
pub fn display_command(program: &str, args: &[&str]) -> String {
    let mut s = program.to_string();
    for a in args {
        s.push(' ');
        s.push_str(a);
    }
    s
}

fn exec(program: &str, args: &[&str], timeout: Duration) -> Result<Output> {
    let cmd_text = display_command(program, args);
    let resolved = which::which(program).map_err(|_| CleanError::Command {
        command: cmd_text.clone(),
        message: format!("`{program}` was not found"),
    })?;
    let mut cmd = Command::new(resolved);
    cmd.args(args)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| CleanError::Command { command: cmd_text.clone(), message: e.to_string() })?;
    let mut so = child.stdout.take();
    let mut se = child.stderr.take();
    let t_out = std::thread::spawn(move || {
        let mut b = Vec::new();
        if let Some(r) = so.as_mut() {
            let _ = r.read_to_end(&mut b);
        }
        b
    });
    let t_err = std::thread::spawn(move || {
        let mut b = Vec::new();
        if let Some(r) = se.as_mut() {
            let _ = r.read_to_end(&mut b);
        }
        b
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(CleanError::Command {
                        command: cmd_text,
                        message: format!("timed out after {} s", timeout.as_secs()),
                    });
                }
                std::thread::sleep(Duration::from_millis(40));
            }
            Err(e) => return Err(CleanError::Command { command: cmd_text, message: e.to_string() }),
        }
    };
    let stdout = String::from_utf8_lossy(&t_out.join().unwrap_or_default()).into_owned();
    let stderr = String::from_utf8_lossy(&t_err.join().unwrap_or_default()).into_owned();
    Ok(Output { code: status.code(), stdout, stderr })
}

// ---------------------------------------------------------------------------------------------
// Windows elevation helpers (type-checked on every OS, only called on Windows)
// ---------------------------------------------------------------------------------------------

/// Quotes a string as a PowerShell single-quoted literal.
#[cfg_attr(unix, allow(dead_code))]
fn ps_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// Paths we agree to embed in a PowerShell literal (no quotes, `$`, backticks or control chars).
#[cfg_attr(unix, allow(dead_code))]
fn ps_safe(p: &Path) -> bool {
    p.to_str()
        .is_some_and(|s| !s.chars().any(|c| c.is_control() || matches!(c, '\'' | '"' | '`' | '$' | ';' | '&' | '|')))
}

#[cfg_attr(unix, allow(dead_code))]
fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

/// Runs `script` in an elevated PowerShell via `Start-Process -Verb RunAs -Wait`.
/// The script is passed base64-encoded, so no quoting of its content is needed.
/// Elevated output cannot be captured; an empty string is returned on success.
#[cfg_attr(unix, allow(dead_code))]
fn run_elevated_powershell(script: &str, timeout: Duration) -> Result<String> {
    let inner = format!("try {{ {script} }} catch {{ exit 1 }}");
    let utf16: Vec<u8> = inner.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    let encoded = base64(&utf16);
    let outer = format!(
        "try {{ $p = Start-Process -FilePath 'powershell' -Verb RunAs -Wait -PassThru -WindowStyle Hidden \
         -ArgumentList '-NoProfile','-NonInteractive','-EncodedCommand','{encoded}'; exit $p.ExitCode }} \
         catch {{ exit {WIN_CANCELLED} }}"
    );
    let args = ["-NoProfile", "-NonInteractive", "-Command", outer.as_str()];
    let out = exec("powershell", &args, timeout + ELEVATION_GRACE)?;
    if out.code == Some(WIN_CANCELLED) {
        return Err(CleanError::ElevationDenied);
    }
    check("powershell (elevated)", &[], out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(path: &str) -> bool {
        let c: Vec<String> = path.split('\\').filter(|p| !p.is_empty() && !p.ends_with(':')).map(|p| p.to_lowercase()).collect();
        c.len() <= 1 || windows_rules(&c)
    }

    #[test]
    fn windows_rules_block_os_dirs() {
        for p in [
            r"C:\Windows",
            r"C:\Windows\System32",
            r"C:\Windows\System32\config",
            r"C:\Windows\Temp",
            r"C:\Windows\SoftwareDistribution",
            r"C:\Program Files",
            r"C:\Program Files\App\x.dll",
            r"C:\Program Files (x86)\App",
            r"C:\ProgramData",
            r"C:\ProgramData\Microsoft\x",
            r"C:\Users",
            r"C:\Users\bob",
            r"C:\Users\bob\Documents",
            r"C:\Users\bob\AppData",
            r"C:\Users\bob\AppData\Local",
            r"C:\Users\bob\AppData\Roaming",
            r"C:\Users\Public",
            r"C:\$Recycle.Bin\S-1\x",
            r"C:\System Volume Information\x",
            r"C:\Temp",
            r"C:",
        ] {
            assert!(win(p), "should be protected: {p}");
        }
    }

    #[test]
    fn windows_rules_allow_known_caches() {
        for p in [
            r"C:\Windows\Temp\a.tmp",
            r"C:\WINDOWS\TEMP\a.tmp",
            r"C:\Windows\SoftwareDistribution\Download\x",
            r"C:\Windows\Minidump\a.dmp",
            r"C:\Windows\ServiceProfiles\NetworkService\AppData\Local\Microsoft\Windows\DeliveryOptimization\Cache\x",
            r"C:\Users\bob\AppData\Local\Temp\x",
            r"C:\Users\bob\AppData\Local\CrashDumps\a.dmp",
            r"C:\Users\bob\Downloads\old.zip",
            r"C:\ProgramData\NVIDIA Corporation\NV_Cache\x",
        ] {
            assert!(!win(p), "should be allowed: {p}");
        }
    }

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn ps_quote_doubles_single_quotes() {
        assert_eq!(ps_quote("a'b"), "'a''b'");
    }
}
