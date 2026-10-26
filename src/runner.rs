//! Runs robocopy on a background thread and turns its output into messages.

use std::io::Read;
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub enum Msg {
    Line(String),
    Done(i32),
    Stopped,
}

#[derive(Default)]
pub struct Runner {
    rx: Option<Receiver<Msg>>,
    child: Arc<Mutex<Option<Child>>>,
    stop_flag: Arc<AtomicBool>,
}

impl Runner {
    pub fn is_running(&self) -> bool {
        self.rx.is_some()
    }

    pub fn start(&mut self, args: Vec<String>) -> std::io::Result<()> {
        let mut child = Command::new("robocopy")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;
        let mut stdout = child.stdout.take().expect("piped stdout");
        *self.child.lock().unwrap() = Some(child);
        self.stop_flag.store(false, Ordering::SeqCst);

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        let child = Arc::clone(&self.child);
        let stop = Arc::clone(&self.stop_flag);
        thread::spawn(move || {
            // robocopy ends progress updates with \r, not \n, so split on both.
            let mut pending = Vec::new();
            let mut buf = [0u8; 4096];
            let flush = |pending: &mut Vec<u8>| {
                if !pending.is_empty() {
                    let _ = tx.send(Msg::Line(decode_oem(pending)));
                    pending.clear();
                }
            };
            while let Ok(n) = stdout.read(&mut buf) {
                if n == 0 {
                    break;
                }
                for &b in &buf[..n] {
                    if b == b'\r' || b == b'\n' {
                        flush(&mut pending);
                    } else {
                        pending.push(b);
                    }
                }
            }
            flush(&mut pending);
            let code = child.lock().unwrap().take().and_then(|mut c| c.wait().ok()).and_then(|s| s.code());
            let _ = tx.send(if stop.load(Ordering::SeqCst) { Msg::Stopped } else { Msg::Done(code.unwrap_or(-1)) });
        });
        Ok(())
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(c) = self.child.lock().unwrap().as_mut() {
            let _ = c.kill();
        }
    }

    /// Drains pending messages; drops the channel once the worker has finished.
    pub fn poll(&mut self) -> Vec<Msg> {
        let mut out = Vec::new();
        let mut finished = false;
        if let Some(rx) = &self.rx {
            while let Ok(m) = rx.try_recv() {
                finished |= matches!(m, Msg::Done(_) | Msg::Stopped);
                out.push(m);
            }
        }
        if finished {
            self.rx = None;
        }
        out
    }
}

/// A robocopy file line looks like `<tabs>  New File <tab> 1234 <tab> path`; the size field is
/// numeric (we pass /BYTES). Checking that shape keeps this independent of the OS language.
pub fn file_line_path(line: &str) -> Option<&str> {
    let fields: Vec<&str> = line.split('\t').map(str::trim).filter(|f| !f.is_empty()).collect();
    if fields.len() < 3 {
        return None;
    }
    let size = fields[fields.len() - 2];
    (size.bytes().all(|b| b.is_ascii_digit())).then(|| fields[fields.len() - 1])
}

/// robocopy reports "ERROR 5 (0x00000005) ... Access is denied" in the user's language; the hex
/// code is the only language-independent part.
pub fn access_denied(line: &str) -> bool {
    line.contains("(0x00000005)")
}

pub fn describe_exit(code: i32) -> String {
    if code < 0 || code >= 16 {
        return format!("Fatal error (exit code {code})");
    }
    if code >= 8 {
        return format!("Finished with FAILED copies (exit code {code})");
    }
    let mut parts = Vec::new();
    if code & 1 != 0 {
        parts.push("files copied");
    }
    if code & 2 != 0 {
        parts.push("extra files/dirs in destination");
    }
    if code & 4 != 0 {
        parts.push("mismatches");
    }
    if parts.is_empty() {
        "Done: nothing to copy, already in sync".into()
    } else {
        format!("Done: {} (exit code {code})", parts.join(", "))
    }
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetOEMCP() -> u32;
    fn MultiByteToWideChar(cp: u32, flags: u32, src: *const u8, srclen: i32, dst: *mut u16, dstlen: i32) -> i32;
}

/// robocopy writes to a pipe using the OEM code page, not UTF-8.
fn decode_oem(bytes: &[u8]) -> String {
    if bytes.is_ascii() {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    unsafe {
        let cp = GetOEMCP();
        let len = MultiByteToWideChar(cp, 0, bytes.as_ptr(), bytes.len() as i32, std::ptr::null_mut(), 0);
        if len <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        let mut wide = vec![0u16; len as usize];
        MultiByteToWideChar(cp, 0, bytes.as_ptr(), bytes.len() as i32, wide.as_mut_ptr(), len);
        String::from_utf16_lossy(&wide)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{OTHER, Options, RULES, VALUES};

    #[test]
    fn detects_file_lines_in_any_language() {
        assert_eq!(file_line_path("\t\t    New File \t\t  1234\tC:\\a\\b.txt"), Some(r"C:\a\b.txt"));
        assert_eq!(file_line_path("\t\t    File nuovo \t\t  0\tx.txt"), Some("x.txt"));
        assert_eq!(file_line_path("\t  New Dir          3\tC:\\a\\"), None);
        assert_eq!(file_line_path("  Total    Copied"), None);
    }

    /// Runs robocopy on a small tree and returns (exit code, lines that look like files).
    fn run_real(name: &str, opts: &Options) -> (Option<i32>, usize) {
        let base = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("src").join("sub")).unwrap();
        std::fs::write(base.join("src").join("a.txt"), "hello").unwrap();
        std::fs::write(base.join("src").join("sub").join("b.txt"), "world").unwrap();
        let args = opts.build_args(base.join("src").to_str().unwrap(), base.join("dst").to_str().unwrap());
        let mut runner = Runner::default();
        runner.start(args).unwrap();
        let (mut files, mut code) = (0, None);
        for _ in 0..200 {
            for m in runner.poll() {
                match m {
                    Msg::Line(l) if file_line_path(&l).is_some() => files += 1,
                    Msg::Done(c) => code = Some(c),
                    _ => {}
                }
            }
            if code.is_some() {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
        (code, files)
    }

    /// A wrong switch makes robocopy exit with 16 (or 8+) and copy nothing, so this proves that every
    /// switch the builder can produce is spelled the way robocopy wants it.
    #[test]
    fn robocopy_accepts_every_builder_switch() {
        let mut o = Options { copy_flags: "DATS".into(), dcopy: "DATEX".into(), ..Default::default() };
        for s in RULES.iter().chain(OTHER) {
            // Rules that exclude everything are fine here: we only look for a syntax error.
            o.set(s.flag, true);
        }
        for v in VALUES {
            o.values.insert(v.flag, v.example.into());
        }
        let (code, _) = run_real("rxc_all_switches", &o);
        let code = code.expect("robocopy finished");
        assert!((0..8).contains(&code), "robocopy rejected the command, exit code {code}: {}", o.preview("S", "D"));
    }

    /// Security and owner copy fine for your own files; this is the widest set that needs no admin.
    #[test]
    fn robocopy_accepts_security_owner_and_dcopy_without_admin() {
        let o = Options { copy_flags: "DATSO".into(), dcopy: "DT".into(), ..Default::default() };
        let (code, files) = run_real("rxc_owner", &o);
        assert!(matches!(code, Some(c) if c < 8), "unexpected exit code {code:?}");
        assert_eq!(files, 2);
    }

    #[test]
    fn runs_real_robocopy_end_to_end() {
        let base = std::env::temp_dir().join("rxc_it");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("src").join("sub")).unwrap();
        std::fs::write(base.join("src").join("a.txt"), "hello").unwrap();
        std::fs::write(base.join("src").join("sub").join("b.txt"), "world").unwrap();
        let args = Options::default().build_args(
            base.join("src").to_str().unwrap(),
            base.join("dst").to_str().unwrap(),
        );
        let mut runner = Runner::default();
        runner.start(args).unwrap();
        let (mut files, mut code) = (0, None);
        for _ in 0..200 {
            for m in runner.poll() {
                match m {
                    Msg::Line(l) if file_line_path(&l).is_some() => files += 1,
                    Msg::Done(c) => code = Some(c),
                    _ => {}
                }
            }
            if code.is_some() {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
        assert_eq!(code, Some(1), "exit code 1 = files copied");
        assert_eq!(files, 2);
        assert!(base.join("dst").join("sub").join("b.txt").exists());
        assert!(!runner.is_running());
    }

    #[test]
    fn access_denied_is_found_by_its_hex_code() {
        assert!(access_denied("ERRORE 5 (0x00000005) Copia del file Accesso negato."));
        assert!(!access_denied("New File 12 a.txt"));
    }

    #[test]
    fn writes_log_file_even_with_spaces_in_path() {
        let base = std::env::temp_dir().join("rxc log it");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("src")).unwrap();
        std::fs::create_dir_all(base.join("my logs")).unwrap();
        std::fs::write(base.join("src").join("a.txt"), "hello").unwrap();
        let log = base.join("my logs").join("run.log");
        let opts = Options { log_file: log.display().to_string(), ..Default::default() };
        let args = opts.build_args(base.join("src").to_str().unwrap(), base.join("dst").to_str().unwrap());
        let mut runner = Runner::default();
        runner.start(args).unwrap();
        let (mut live_files, mut code) = (0, None);
        for _ in 0..200 {
            for m in runner.poll() {
                match m {
                    Msg::Line(l) if file_line_path(&l).is_some() => live_files += 1,
                    Msg::Done(c) => code = Some(c),
                    _ => {}
                }
            }
            if code.is_some() {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
        assert_eq!(code, Some(1));
        assert_eq!(live_files, 1, "/TEE keeps the live output");
        let text = std::fs::read(&log).expect("log file written");
        assert!(String::from_utf8_lossy(&text).contains("a.txt"));
    }

    /// `/UNILOG` writes UTF-16: an accented file name must come out readable, which is the point of the option.
    #[test]
    fn unicode_log_keeps_accented_names() {
        let base = std::env::temp_dir().join("rxc uni");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("src")).unwrap();
        std::fs::create_dir_all(base.join("logs")).unwrap();
        std::fs::write(base.join("src").join("città-è.txt"), "ciao").unwrap();
        let log = base.join("logs").join("uni.log");
        let opts = Options { log_file: log.display().to_string(), log_unicode: true, ..Default::default() };
        let args = opts.build_args(base.join("src").to_str().unwrap(), base.join("dst").to_str().unwrap());
        assert!(args.iter().any(|a| a.starts_with("/UNILOG:")));
        let mut runner = Runner::default();
        runner.start(args).unwrap();
        let mut code = None;
        for _ in 0..200 {
            for m in runner.poll() {
                if let Msg::Done(c) = m {
                    code = Some(c);
                }
            }
            if code.is_some() {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
        assert_eq!(code, Some(1));
        let bytes = std::fs::read(&log).expect("log written");
        let units: Vec<u16> = bytes.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect();
        assert!(String::from_utf16_lossy(&units).contains("città-è.txt"));
    }

    #[test]
    fn exit_codes() {
        assert!(describe_exit(0).contains("in sync"));
        assert!(describe_exit(8).contains("FAILED"));
        assert!(describe_exit(16).contains("Fatal"));
    }
}
