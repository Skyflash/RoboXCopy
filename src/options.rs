//! What the user chose, and how it becomes a robocopy command line.
//!
//! The less common switches are described by tables (`RULES`, `OTHER`, `VALUES`, `COPY_FLAGS`,
//! `DCOPY_FLAGS`); the COPY BUILDER window draws itself from the same tables, so a switch added here
//! shows up in the window and in the command line without further code.

use std::collections::{BTreeMap, BTreeSet};

/// A yes/no switch with a description shown in the builder.
pub struct Switch {
    pub flag: &'static str,
    pub label: &'static str,
    pub help: &'static str,
}

/// A switch that takes a value (`/MAXAGE:30`).
pub struct ValueSwitch {
    pub flag: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    /// A valid sample, shown as the hint in the field and used by the tests.
    pub example: &'static str,
}

/// Which files are selected, compared with the copy already in the target.
pub const RULES: &[Switch] = &[
    Switch { flag: "/XO", label: "Exclude older", help: "Skip a file when the target's copy has the same date or is newer." },
    Switch { flag: "/XN", label: "Exclude newer", help: "Skip source files that are newer than the copy in the target." },
    Switch { flag: "/XC", label: "Exclude changed", help: "Skip files whose size or time differs from the target's copy." },
    Switch { flag: "/XL", label: "Exclude lonely", help: "Skip files that exist only in the source (copy nothing new)." },
    Switch { flag: "/XX", label: "Exclude extra", help: "Ignore files that exist only in the target. Wins over PURGE and MIRROR." },
    Switch { flag: "/IS", label: "Include same", help: "Copy files even when they are identical in both places." },
    Switch { flag: "/IT", label: "Include tweaked", help: "Copy files that differ only in attributes." },
];

pub const OTHER: &[Switch] = &[
    Switch { flag: "/PURGE", label: "Purge", help: "Delete target files that no longer exist in the source (MIRROR already does)." },
    Switch { flag: "/FFT", label: "FAT file times", help: "Assume 2-second time accuracy: use with NAS boxes and FAT or exFAT drives." },
    Switch { flag: "/DST", label: "Daylight saving", help: "Tolerate the one-hour difference caused by daylight saving time." },
    Switch { flag: "/XJ", label: "Skip junctions", help: "Do not follow junction points and folder links (avoids loops)." },
    Switch { flag: "/J", label: "Unbuffered I/O", help: "Faster for very large files." },];

pub const VALUES: &[ValueSwitch] = &[
    ValueSwitch { flag: "/MAXAGE:", label: "Max age", help: "Skip files older than this many days (or a date, YYYYMMDD).", example: "30" },
    ValueSwitch { flag: "/MINAGE:", label: "Min age", help: "Skip files newer than this many days (or a date, YYYYMMDD).", example: "7" },
    ValueSwitch { flag: "/MAX:", label: "Max size", help: "Skip files bigger than this many bytes.", example: "1073741824" },
    ValueSwitch { flag: "/MIN:", label: "Min size", help: "Skip files smaller than this many bytes.", example: "1024" },
    ValueSwitch { flag: "/IA:", label: "Only attrs", help: "Copy only files that have one of these attributes (R A S H C N E T O).", example: "A" },
    ValueSwitch { flag: "/XA:", label: "Skip attrs", help: "Skip files that have one of these attributes (R A S H C N E T O).", example: "H" },
    ValueSwitch { flag: "/IPG:", label: "Pause", help: "Milliseconds to wait between 64 KB blocks, to spare the network. Single thread only.", example: "5" },
];

/// `(letter, name, description)` for `/COPY:`. The order is robocopy's own.
pub const COPY_FLAGS: &[(char, &str, &str)] = &[
    ('D', "Data", "The file contents."),
    ('A', "Attributes", "Read-only, hidden, archive and so on."),
    ('T', "Timestamps", "Created, modified and accessed dates."),
    ('S', "Security", "NTFS permissions (ACLs). Works on your own files without admin."),
    ('O', "Owner", "Owner information. Own files are fine; other owners need admin."),
    ('U', "Auditing", "Auditing information. Needs administrator rights, or robocopy will not start."),
];

pub const DCOPY_FLAGS: &[(char, &str, &str)] = &[
    ('D', "Data", "Create the folders."),
    ('A', "Attributes", "Folder attributes."),
    ('T', "Timestamps", "Folder dates (otherwise they show the time of the copy)."),
    ('E', "Extended attrs", "Extended attributes of the folders."),
    ('X', "Skip streams", "Do not copy alternate data streams."),
];

pub const COPY_DEFAULT: &str = "DAT";
pub const COPY_ALL: &str = "DATSOU";

#[derive(Clone)]
pub struct Options {
    pub subdirs: bool,
    pub mirror: bool,
    pub move_files: bool,
    pub list_only: bool,
    pub restartable: bool,
    pub backup: bool,
    /// Letters of `/COPY:`, always in robocopy's order and never empty.
    pub copy_flags: String,
    /// Letters of `/DCOPY:`; empty = leave robocopy's own default.
    pub dcopy: String,
    /// Enabled switches from `RULES` and `OTHER`.
    pub switches: BTreeSet<&'static str>,
    /// Values for `VALUES` switches; blank = not used.
    pub values: BTreeMap<&'static str, String>,
    pub multithread: bool,
    pub threads: u32,
    pub retries: u32,
    pub wait: u32,
    pub files: String,
    pub exclude_files: String,
    pub exclude_dirs: String,
    pub extra: String,
    /// Empty = no log file. Chosen from the LOG window.
    pub log_file: String,
    pub log_append: bool,
    /// Write the log as UTF-16 (`/UNILOG`), so accented and non-Latin file names survive.
    pub log_unicode: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            subdirs: true,
            mirror: false,
            move_files: false,
            list_only: false,
            restartable: false,
            backup: false,
            copy_flags: COPY_DEFAULT.into(),
            dcopy: String::new(),
            switches: BTreeSet::new(),
            values: BTreeMap::new(),
            multithread: true,
            threads: 8,
            retries: 3,
            wait: 2,
            files: String::new(),
            exclude_files: String::new(),
            exclude_dirs: String::new(),
            extra: String::new(),
            log_file: String::new(),
            log_append: false,
            log_unicode: false,
        }
    }
}

/// Keeps `letters` in the canonical order of `table`, dropping anything unknown.
fn canonical(table: &[(char, &str, &str)], letters: impl Fn(char) -> bool) -> String {
    table.iter().map(|t| t.0).filter(|c| letters(*c)).collect()
}

impl Options {
    /// Backup mode and the auditing flag make robocopy refuse to start (exit 16) without administrator
    /// rights. Security and owner copy fine for your own files, so those only get the after-the-fact
    /// "access denied" prompt.
    pub fn needs_admin(&self) -> bool {
        self.backup || self.copy_flags.contains('U')
    }

    /// The text typed into a `VALUES` field, if it is usable on a command line.
    pub fn value_of(&self, flag: &str) -> Option<&str> {
        let token = self.values.get(flag)?.split_whitespace().next()?;
        // Only plain characters: this is typed text going onto a command line.
        token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-').then_some(token)
    }

    /// robocopy refuses `/IPG` together with `/MT` (exit 16, nothing copied), so a pause wins and the
    /// copy runs on one thread.
    pub fn threads_off_for_pause(&self) -> bool {
        self.multithread && self.value_of("/IPG:").is_some()
    }

    pub fn has(&self, flag: &str) -> bool {
        self.switches.contains(flag)
    }

    pub fn set(&mut self, flag: &'static str, on: bool) {
        if on {
            self.switches.insert(flag);
        } else {
            self.switches.remove(flag);
        }
    }

    pub fn toggle_copy_flag(&mut self, c: char) {
        let now = self.copy_flags.contains(c);
        let next = canonical(COPY_FLAGS, |x| (x == c) != self.copy_flags.contains(x));
        // /COPY: with no letters is invalid, so the last one cannot be switched off.
        if !(now && next.is_empty()) {
            self.copy_flags = next;
        }
    }

    pub fn toggle_dcopy(&mut self, c: char) {
        self.dcopy = canonical(DCOPY_FLAGS, |x| (x == c) != self.dcopy.contains(x));
    }

    /// Value shown in the ATTRIBS field.
    pub fn copy_label(&self) -> String {
        if self.copy_flags == COPY_ALL { "ALL".into() } else { self.copy_flags.clone() }
    }

    /// Value shown in the UPDATE field.
    pub fn update_label(&self) -> String {
        let active = RULES.iter().filter(|r| self.has(r.flag)).count();
        match (active, self.has("/XO")) {
            (0, _) => "ALL".into(),
            (1, true) => "NEWER".into(),
            (n, _) => format!("{n} RULES"),
        }
    }

    pub fn build_args(&self, source: &str, dest: &str) -> Vec<String> {
        let mut a = vec![clean_path(source), clean_path(dest)];
        a.extend(self.files.split_whitespace().map(String::from));
        // /MIR already implies /E; passing both is harmless but noisy.
        if self.mirror {
            a.push("/MIR".into());
        } else if self.subdirs {
            a.push("/E".into());
        }
        let flags = [
            (self.move_files, "/MOVE"),
            (self.list_only, "/L"),
            (self.restartable, "/Z"),
            (self.backup, "/B"),
        ];
        a.extend(flags.iter().filter(|(on, _)| *on).map(|(_, f)| f.to_string()));
        if self.copy_flags == COPY_ALL {
            a.push("/COPYALL".into());
        } else if !self.copy_flags.is_empty() {
            a.push(format!("/COPY:{}", self.copy_flags));
        }
        if !self.dcopy.is_empty() {
            a.push(format!("/DCOPY:{}", self.dcopy));
        }
        // Table order, so the command line does not depend on the order of the clicks.
        a.extend(RULES.iter().chain(OTHER).filter(|s| self.has(s.flag)).map(|s| s.flag.to_string()));
        for v in VALUES {
            if let Some(token) = self.value_of(v.flag) {
                a.push(format!("{}{token}", v.flag));
            }
        }
        if self.multithread && !self.threads_off_for_pause() {
            a.push(format!("/MT:{}", self.threads));
        }
        a.push(format!("/R:{}", self.retries));
        a.push(format!("/W:{}", self.wait));
        if !self.exclude_files.trim().is_empty() {
            a.push("/XF".into());
            a.extend(self.exclude_files.split_whitespace().map(String::from));
        }
        if !self.exclude_dirs.trim().is_empty() {
            a.push("/XD".into());
            a.extend(self.exclude_dirs.split_whitespace().map(String::from));
        }
        a.extend(self.extra.split_whitespace().map(String::from));
        let log = self.log_file.trim().trim_matches('"');
        if !log.is_empty() {
            // /LOG alone would silence stdout; /TEE keeps the live display working.
            let name = match (self.log_unicode, self.log_append) {
                (false, false) => "/LOG",
                (false, true) => "/LOG+",
                (true, false) => "/UNILOG",
                (true, true) => "/UNILOG+",
            };
            a.push(format!("{name}:{log}"));
            a.push("/TEE".into());
        }
        // No header/job summary noise, sizes in plain bytes so they stay parseable.
        a.extend(["/NJH", "/BYTES"].map(String::from));
        a
    }

    /// The command as it would be typed, for the builder's preview line.
    pub fn preview(&self, source: &str, dest: &str) -> String {
        let quote = |s: String| if s.contains(' ') { format!("\"{s}\"") } else { s };
        let src = if source.trim().is_empty() { "<source>" } else { source };
        let dst = if dest.trim().is_empty() { "<target>" } else { dest };
        let args: Vec<String> = self.build_args(src, dst).into_iter().map(quote).collect();
        format!("robocopy {}", args.join(" "))
    }
}

/// robocopy mis-parses a trailing backslash before the closing quote ("C:\" eats the quote).
pub fn clean_path(p: &str) -> String {
    let t = p.trim().trim_matches('"');
    let t = t.trim_end_matches(['\\', '/']);
    if t.len() == 2 && t.ends_with(':') {
        format!("{t}\\.")
    } else {
        t.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(o: &Options) -> Vec<String> {
        o.build_args(r"C:\s", r"C:\d")
    }

    #[test]
    fn root_and_trailing_slash_paths() {
        assert_eq!(clean_path(r"C:\"), r"C:\.");
        assert_eq!(clean_path(r#""D:\data\""#), r"D:\data");
    }

    #[test]
    fn mirror_replaces_subdirs_flag() {
        let a = args(&Options { mirror: true, ..Default::default() });
        assert!(a.contains(&"/MIR".to_string()) && !a.contains(&"/E".to_string()));
    }

    #[test]
    fn copy_flags_default_all_and_custom() {
        let mut o = Options::default();
        assert!(args(&o).contains(&"/COPY:DAT".to_string()), "explicit /COPY:DAT by default");
        assert_eq!(o.copy_label(), "DAT");

        o.toggle_copy_flag('S');
        assert_eq!(o.copy_flags, "DATS");
        assert!(args(&o).contains(&"/COPY:DATS".to_string()));

        for c in ['O', 'U'] {
            o.toggle_copy_flag(c);
        }
        assert_eq!(o.copy_label(), "ALL");
        let a = args(&o);
        assert!(a.contains(&"/COPYALL".to_string()) && !a.iter().any(|x| x.starts_with("/COPY:")));
    }

    #[test]
    fn copy_flags_keep_robocopy_order_and_never_go_empty() {
        let mut o = Options::default();
        o.toggle_copy_flag('U');
        o.toggle_copy_flag('S');
        assert_eq!(o.copy_flags, "DATSU", "order does not depend on the clicks");
        for c in ['D', 'A', 'T', 'S'] {
            o.toggle_copy_flag(c);
        }
        o.toggle_copy_flag('U');
        assert_eq!(o.copy_flags, "U", "the last letter stays on");
    }

    #[test]
    fn dcopy_is_optional() {
        let mut o = Options::default();
        assert!(!args(&o).iter().any(|x| x.starts_with("/DCOPY")));
        o.toggle_dcopy('T');
        o.toggle_dcopy('D');
        assert!(args(&o).contains(&"/DCOPY:DT".to_string()));
        o.toggle_dcopy('T');
        o.toggle_dcopy('D');
        assert!(o.dcopy.is_empty());
    }

    #[test]
    fn selection_rules_labels_and_order() {
        let mut o = Options::default();
        assert_eq!(o.update_label(), "ALL");
        o.set("/XO", true);
        assert_eq!(o.update_label(), "NEWER");
        o.set("/IS", true);
        o.set("/XL", true);
        assert_eq!(o.update_label(), "3 RULES");
        let a = args(&o);
        let pos = |f: &str| a.iter().position(|x| x == f).unwrap();
        assert!(pos("/XO") < pos("/XL") && pos("/XL") < pos("/IS"), "table order: {a:?}");
        o.set("/XO", false);
        assert!(!args(&o).contains(&"/XO".to_string()));
    }

    #[test]
    fn value_switches_accept_plain_tokens_only() {
        let mut o = Options::default();
        o.values.insert("/MAXAGE:", "30".into());
        o.values.insert("/MIN:", "  1024 extra words ".into());
        o.values.insert("/IPG:", "5 & calc".into());
        o.values.insert("/MAX:", "1;2".into());
        let a = args(&o);
        assert!(a.contains(&"/MAXAGE:30".to_string()));
        assert!(a.contains(&"/MIN:1024".to_string()), "only the first word is used");
        assert!(a.contains(&"/IPG:5".to_string()));
        assert!(!a.iter().any(|x| x.starts_with("/MAX:")), "odd characters are dropped: {a:?}");
    }

    #[test]
    fn admin_is_needed_for_auditing_and_backup_only() {
        assert!(!Options::default().needs_admin());
        let with = |f: &str| Options { copy_flags: f.into(), ..Default::default() };
        assert!(!with("DATS").needs_admin());
        assert!(!with("DATSO").needs_admin(), "owner works for your own files");
        assert!(with("DATU").needs_admin());
        assert!(with(COPY_ALL).needs_admin());
        assert!(Options { backup: true, ..Default::default() }.needs_admin());
    }

    #[test]
    fn pause_and_threads_never_go_together() {
        let mut o = Options::default();
        assert!(args(&o).contains(&"/MT:8".to_string()));
        o.values.insert("/IPG:", "5".into());
        assert!(o.threads_off_for_pause());
        let a = args(&o);
        assert!(a.contains(&"/IPG:5".to_string()) && !a.iter().any(|x| x.starts_with("/MT")), "{a:?}");
        o.values.insert("/IPG:", "".into());
        assert!(args(&o).contains(&"/MT:8".to_string()));
    }

    #[test]
    fn log_file_args() {
        let mut o = Options { log_file: r"C:\my logs\run.log".into(), ..Default::default() };
        assert!(args(&o).contains(&r"/LOG:C:\my logs\run.log".to_string()) && args(&o).contains(&"/TEE".to_string()));
        o.log_append = true;
        assert!(args(&o).contains(&r"/LOG+:C:\my logs\run.log".to_string()));
        o.log_unicode = true;
        assert!(args(&o).contains(&r"/UNILOG+:C:\my logs\run.log".to_string()));
        o.log_append = false;
        assert!(args(&o).contains(&r"/UNILOG:C:\my logs\run.log".to_string()));
        assert!(!args(&Options::default()).iter().any(|x| x.starts_with("/LOG")));
    }

    #[test]
    fn preview_quotes_and_uses_placeholders() {
        let o = Options { log_file: r"C:\my logs\run.log".into(), ..Default::default() };
        let p = o.preview("", r"E:\Backup dir");
        assert!(p.starts_with("robocopy <source> \"E:\\Backup dir\" /E"), "{p}");
        assert!(p.contains("\"/LOG:C:\\my logs\\run.log\""), "{p}");
    }
}
