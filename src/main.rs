#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod builder;
mod icon;
mod options;
mod runner;
#[cfg(feature = "screenshot")]
mod shot;
mod sys;
mod theme;
mod ui;

use eframe::egui::{self, Align2, Color32, RichText};
use options::Options;
use runner::{Msg, Runner};
use std::time::{Duration, Instant};
use theme::THEMES;
use ui::{Canvas, Kind};

const MAX_DRIVES: usize = 8;
const HOWTO_URL: &str = "https://roboxcopy.com/howto.html";
const PROJECT_URL: &str = "https://github.com/Skyflash/RoboXCopy";

/// Asks the user about administrator rights. `then_start` = the START that triggered the question
/// (`Some(dry_run)`), so CONTINUE ANYWAY can run it unelevated.
struct AdminPrompt {
    reason: String,
    then_start: Option<bool>,
}

enum AdminChoice {
    Restart,
    Continue,
    Cancel,
}

fn main() -> eframe::Result {
    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Robo-XCopy Professional")
            .with_icon(icon::window_icon())
            .with_inner_size([960.0, 495.0])
            .with_min_inner_size([640.0, 330.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Robo-XCopy",
        native,
        Box::new(|cc| {
            #[allow(unused_mut)]
            let mut theme = theme::load_index();
            #[cfg(feature = "screenshot")]
            let shot = shot::Shot::from_args();
            #[cfg(feature = "screenshot")]
            if let Some(t) = shot.as_ref().and_then(|s| s.theme) {
                theme = t.min(THEMES.len() - 1);
            }
            theme::apply(&cc.egui_ctx, THEMES[theme]);
            #[allow(unused_mut)]
            let mut app = App::new(theme);
            #[cfg(feature = "screenshot")]
            if let Some(shot) = shot {
                app.start_shot(shot);
            }
            Ok(Box::new(app))
        }),
    )
}

struct App {
    source: String,
    dest: String,
    opts: Options,
    runner: Runner,
    drives: Vec<char>,
    log: Vec<String>,
    status: String,
    file_percent: f32,
    files_done: usize,
    current: String,
    last_args: Option<Vec<String>>,
    started: Option<Instant>,
    elapsed: Duration,
    theme: usize,
    show_filters: bool,
    show_log: bool,
    show_help: bool,
    show_builder: bool,
    elevated: bool,
    admin_prompt: Option<AdminPrompt>,
    /// The user chose to go on without admin rights; do not ask again this session.
    admin_declined: bool,
    /// robocopy reported "access denied" (error 5) during the current run.
    saw_denied: bool,
    /// Pretend a copy is running (only ever set by the `screenshot` feature).
    demo: bool,
    #[cfg(feature = "screenshot")]
    shot: Option<shot::Shot>,
}

impl App {
    fn is_busy(&self) -> bool {
        self.runner.is_running() || self.demo
    }

    fn new(theme: usize) -> Self {
        // A: and B: are skipped on purpose: probing an empty floppy letter can stall startup.
        let drives = ('C'..='Z')
            .filter(|l| std::path::Path::new(&format!("{l}:\\")).exists())
            .take(MAX_DRIVES)
            .collect();
        // `--source` / `--dest` let the elevated copy of the app pick up where this one left off.
        let args: Vec<String> = std::env::args().collect();
        let arg = |flag: &str| {
            args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned().unwrap_or_default()
        };
        Self {
            source: arg("--source"),
            dest: arg("--dest"),
            opts: Options::default(),
            runner: Runner::default(),
            drives,
            log: Vec::new(),
            status: "Select source and target, then press START".into(),
            file_percent: 0.0,
            files_done: 0,
            current: String::new(),
            last_args: None,
            started: None,
            elapsed: Duration::ZERO,
            theme,
            show_filters: false,
            show_log: false,
            show_help: false,
            show_builder: false,
            elevated: sys::is_elevated(),
            admin_prompt: None,
            admin_declined: false,
            saw_denied: false,
            demo: false,
            #[cfg(feature = "screenshot")]
            shot: None,
        }
    }

    fn start(&mut self, dry_run: bool) {
        if self.runner.is_running() {
            return;
        }
        if self.source.trim().is_empty() || self.dest.trim().is_empty() {
            self.status = "Select a source and a target first".into();
            return;
        }
        if !std::path::Path::new(self.source.trim()).is_dir() {
            self.status = "Source folder does not exist".into();
            return;
        }
        // robocopy will not create the folder of its log file; it would stop with a fatal error.
        let log = self.opts.log_file.trim().trim_matches('"');
        if !log.is_empty() && !std::path::Path::new(log).parent().is_some_and(|d| d.as_os_str().is_empty() || d.is_dir()) {
            self.status = "The folder of the log file does not exist".into();
            return;
        }
        let mut opts = self.opts.clone();
        opts.list_only |= dry_run;
        if !dry_run && !self.elevated && !self.admin_declined && opts.needs_admin() {
            self.admin_prompt = Some(AdminPrompt {
                reason: "Backup mode (/B) and the auditing copy flag (U, part of /COPYALL) need administrator rights: \
                         without them robocopy does not even start."
                    .into(),
                then_start: Some(dry_run),
            });
            return;
        }
        let args = opts.build_args(&self.source, &self.dest);
        self.launch(args);
    }

    /// Relaunches this exe elevated (UAC prompt), keeping source and target. Closes this window
    /// only if the elevated copy really started.
    fn restart_as_admin(&mut self, ctx: &egui::Context) {
        let mut args = String::new();
        for (flag, path) in [("--source", &self.source), ("--dest", &self.dest)] {
            let p = path.trim();
            if !p.is_empty() {
                // A trailing backslash would escape the closing quote; a drive root keeps a "." instead.
                let p = if p.len() <= 3 && p.ends_with('\\') { format!("{p}.") } else { p.trim_end_matches('\\').to_string() };
                args.push_str(&format!("{flag} \"{p}\" "));
            }
        }
        match sys::relaunch_as_admin(args.trim_end()) {
            Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Err(e) => self.status = e,
        }
    }

    fn launch(&mut self, args: Vec<String>) {
        let dry = args.iter().any(|a| a == "/L");
        self.log.clear();
        self.files_done = 0;
        self.file_percent = 0.0;
        self.current.clear();
        self.elapsed = Duration::ZERO;
        self.saw_denied = false;
        self.log.push(format!("> robocopy {}", args.join(" ")));
        match self.runner.start(args.clone()) {
            Ok(()) => {
                self.started = Some(Instant::now());
                self.last_args = Some(args);
                self.status = if dry { "Dry run in progress..." } else { "Copying..." }.into();
            }
            Err(e) => self.status = format!("Cannot start robocopy: {e}"),
        }
    }

    fn pump(&mut self) {
        for msg in self.runner.poll() {
            match msg {
                Msg::Line(line) => self.on_line(line),
                Msg::Done(code) => {
                    self.finish();
                    self.status = runner::describe_exit(code);
                    self.log.push(format!("> {}", self.status));
                    if !self.elevated && self.admin_prompt.is_none() {
                        // 16 = robocopy refused to start; with admin-only options that is a rights problem.
                        let reason = if code == 16 && self.opts.needs_admin() {
                            Some("robocopy stopped: the selected options need administrator rights.")
                        } else if code >= 8 && self.saw_denied {
                            Some("Some files were not copied: access denied.")
                        } else {
                            None
                        };
                        self.admin_prompt = reason.map(|r| AdminPrompt { reason: r.into(), then_start: None });
                    }
                }
                Msg::Stopped => {
                    self.finish();
                    self.status = "Stopped by user".into();
                    self.log.push("> Stopped by user".into());
                }
            }
        }
        if let Some(t) = self.started {
            self.elapsed = t.elapsed();
        }
    }

    fn finish(&mut self) {
        self.started = None;
        self.file_percent = 0.0;
    }

    fn on_line(&mut self, line: String) {
        let t = line.trim();
        if t.is_empty() {
            return;
        }
        if let Some(p) = t.strip_suffix('%').and_then(|n| n.trim().parse::<f32>().ok()) {
            self.file_percent = p / 100.0;
            return;
        }
        self.saw_denied |= runner::access_denied(&line);
        if let Some(path) = runner::file_line_path(&line) {
            self.files_done += 1;
            self.current = path.to_string();
            self.file_percent = 0.0;
        }
        self.log.push(line);
        if self.log.len() > 20_000 {
            self.log.drain(..5_000);
        }
    }

    fn status_line(&self) -> String {
        let text = if self.is_busy() && !self.current.is_empty() {
            format!("{}  {}", self.status, self.current)
        } else {
            self.status.clone()
        };
        const MAX: usize = 96;
        let n = text.chars().count();
        if n <= MAX {
            text
        } else {
            let tail: String = text.chars().skip(n - (MAX - 3)).collect();
            format!("...{tail}")
        }
    }

    fn cycle_theme(&mut self, ctx: &egui::Context) {
        self.theme = (self.theme + 1) % THEMES.len();
        theme::apply(ctx, THEMES[self.theme]);
        theme::save_index(self.theme);
        self.status = format!("Theme: {}", THEMES[self.theme].name);
    }

    fn draw(&mut self, ui: &mut egui::Ui, c: &Canvas) {
        let t = c.t;
        let running = self.is_busy();
        let idle = !running;

        // Banner
        let banner = c.rect(8.0, 6.0, 624.0, 44.0);
        ui.painter().rect_filled(banner, 0.0, t.panel);
        ui::frame(ui, c, banner, t.frame);
        ui::text(ui, c, 322.5, 25.5, Align2::CENTER_CENTER, 30.0, t.title_shadow, "ROBO-XCOPY");
        ui::text(ui, c, 319.0, 22.0, Align2::CENTER_CENTER, 30.0, t.title_hi, "ROBO-XCOPY");
        ui::text(ui, c, 320.0, 23.0, Align2::CENTER_CENTER, 30.0, t.title, "ROBO-XCOPY");
        ui::text(ui, c, 400.0, 41.0, Align2::LEFT_CENTER, 10.0, t.sub, "professional");
        ui::text(ui, c, 16.0, 16.0, Align2::LEFT_CENTER, 8.0, t.label, "(c)2026");
        ui::text(ui, c, 16.0, 28.0, Align2::LEFT_CENTER, 8.0, t.label, "robocopy");
        ui::text(ui, c, 16.0, 40.0, Align2::LEFT_CENTER, 8.0, t.label, "front end");
        ui::text(
            ui,
            c,
            624.0,
            40.0,
            Align2::RIGHT_CENTER,
            8.0,
            t.label,
            &format!("v{}", env!("CARGO_PKG_VERSION")),
        );
        // Rights indicator: a gadget to restart elevated, or a plain mark when already admin.
        if self.elevated {
            ui::text(ui, c, 624.0, 18.0, Align2::RIGHT_CENTER, 9.0, t.value, "* ADMIN *");
        } else if ui::button(ui, c, c.rect(512.0, 10.0, 112.0, 16.0), "RUN AS ADMIN", Kind::Alt, !self.is_busy()) {
            self.admin_prompt = Some(AdminPrompt {
                reason: "Some folders and options (backup mode, owner and ACLs) need administrator rights.".into(),
                then_start: None,
            });
        }

        // Parameter row
        let params = c.rect(8.0, 54.0, 624.0, 44.0);
        ui::frame(ui, c, params, t.frame);
        ui::sep(ui, c, 10.0, 65.0, 620.0);
        let fx = |i: usize| 14.0 + i as f32 * 86.0;
        // Fields that toggle on click get the little orange dot, as on the original screen.
        let label = |ui: &egui::Ui, i: usize, s: &str, toggles: bool| {
            ui::text(ui, c, fx(i) + 40.0, 60.0, Align2::CENTER_CENTER, 8.0, t.label, s);
            if toggles {
                ui::dot(ui, c, fx(i) + 40.0 - s.len() as f32 * 2.4 - 6.0, 60.0);
            }
        };
        let o = &mut self.opts;

        label(ui, 0, "MODE", true);
        let mode = if o.mirror { "MIRROR" } else if o.move_files { "MOVE" } else { "COPY" };
        if ui::value_box(ui, c, c.rect(fx(0), 73.0, 80.0, 14.0), "mode", mode, idle) {
            (o.mirror, o.move_files) = match (o.mirror, o.move_files) {
                (false, false) => (true, false),
                (true, _) => (false, true),
                _ => (false, false),
            };
        }

        label(ui, 1, "SUBDIRS", true);
        let subdirs = if o.subdirs || o.mirror { "ALL" } else { "NONE" };
        if ui::value_box(ui, c, c.rect(fx(1), 73.0, 80.0, 14.0), "subdirs", subdirs, idle && !o.mirror) {
            o.subdirs = !o.subdirs;
        }

        let numeric = |ui: &mut egui::Ui, i: usize, name: &str, v: &mut u32, min: u32, max: u32| {
            label(ui, i, &name.to_uppercase(), false);
            let (x, key) = (fx(i), name);
            if ui::arrow(ui, c, c.rect(x + 30.0, 67.0, 20.0, 5.0), key, true, idle) {
                *v = (*v + 1).min(max);
            }
            // Read-only box: the arrows change the value, a click on the box does nothing.
            ui::value_box(ui, c, c.rect(x, 73.0, 80.0, 14.0), key, &format!("{v:02}"), idle);
            if ui::arrow(ui, c, c.rect(x + 30.0, 88.0, 20.0, 5.0), key, false, idle) {
                *v = v.saturating_sub(1).max(min);
            }
        };
        numeric(ui, 2, "threads", &mut o.threads, 1, 128);
        o.multithread = o.threads > 1;
        numeric(ui, 3, "retry", &mut o.retries, 0, 999);
        numeric(ui, 4, "wait", &mut o.wait, 0, 999);

        // These two summarise the COPY BUILDER; a click opens it.
        label(ui, 5, "ATTRIBS", true);
        if ui::value_box(ui, c, c.rect(fx(5), 73.0, 80.0, 14.0), "attr", &o.copy_label(), idle) {
            self.show_builder = true;
        }
        label(ui, 6, "UPDATE", true);
        if ui::value_box(ui, c, c.rect(fx(6), 73.0, 80.0, 14.0), "upd", &o.update_label(), idle) {
            self.show_builder = true;
        }

        // Gadget row
        let br = |i: usize| c.rect(8.0 + i as f32 * 78.0, 104.0, 72.0, 18.0);
        if ui::button(ui, c, br(0), "START", Kind::Normal, idle) {
            self.start(false);
        }
        if ui::button(ui, c, br(1), "REPEAT", Kind::Normal, idle && self.last_args.is_some()) {
            if let Some(args) = self.last_args.clone() {
                self.launch(args);
            }
        }
        if ui::button(ui, c, br(2), "STOP", Kind::Normal, running) {
            self.runner.stop();
        }
        if ui::button(ui, c, br(3), "DRY RUN", Kind::Alt, idle) {
            self.start(true);
        }
        if ui::button(ui, c, br(4), "FILTERS", Kind::Alt, true) {
            self.show_filters = !self.show_filters;
        }
        if ui::button(ui, c, br(5), "LOG", Kind::Alt, true) {
            self.show_log = !self.show_log;
        }
        if ui::button(ui, c, br(6), "THEME", Kind::Alt, true) {
            self.cycle_theme(&ui.ctx().clone());
        }
        if ui::button(ui, c, br(7), "HELP", Kind::Danger, true) {
            self.show_help = !self.show_help;
        }

        // Drives and paths
        let drive_block = |ui: &mut egui::Ui, title: &str, y: f32, path: &mut String, drives: &[char]| {
            ui::text(ui, c, 155.0, y, Align2::CENTER_CENTER, 8.0, t.heading, title);
            ui::sep(ui, c, 8.0, y + 5.5, 294.0);
            let current = path.chars().next().filter(|_| path.chars().nth(1) == Some(':'));
            for (i, &d) in drives.iter().enumerate() {
                let lit = current.is_some_and(|p| p.eq_ignore_ascii_case(&d));
                if ui::drive_icon(ui, c, 14.0 + i as f32 * 36.0, y + 8.0, d, lit, idle) {
                    *path = format!("{d}:\\");
                }
            }
            let edit = c.rect(14.0, y + 50.0, 218.0, 18.0);
            ui.painter().rect_filled(edit, 0.0, Color32::BLACK);
            ui::frame(ui, c, edit, t.frame_dim);
            ui.add_enabled_ui(idle, |ui| {
                ui.put(
                    edit.shrink(2.0),
                    egui::TextEdit::singleline(path)
                        .font(c.font(10.0))
                        .text_color(t.value)
                        .frame(egui::Frame::NONE)
                        .align(Align2::LEFT_CENTER),
                );
            });
            if ui::button(ui, c, c.rect(238.0, y + 50.0, 64.0, 18.0), "BROWSE", Kind::Alt, idle) {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    *path = dir.display().to_string();
                }
            }
        };
        drive_block(ui, "SOURCE DRIVE", 140.0, &mut self.source, &self.drives);
        drive_block(ui, "TARGET DRIVES", 214.0, &mut self.dest, &self.drives);

        // Grids
        ui::text(ui, c, 397.0, 140.0, Align2::CENTER_CENTER, 8.0, t.heading, "FILES");
        ui::text(ui, c, 557.0, 140.0, Align2::CENTER_CENTER, 8.0, t.heading, "CURRENT FILE");
        ui::sep(ui, c, 322.0, 145.5, 310.0);
        let per_lap = ui::GRID_CELLS + 1;
        ui::grid(ui, c, 322.0, 158.0, self.files_done % per_lap, self.files_done / per_lap);
        ui::grid(ui, c, 482.0, 158.0, (self.file_percent * ui::GRID_CELLS as f32).round() as usize, 0);

        let secs = self.elapsed.as_secs();
        ui::text(
            ui,
            c,
            322.0,
            286.0,
            Align2::LEFT_CENTER,
            9.0,
            t.value,
            &format!("FILES {:>6}", self.files_done),
        );
        ui::text(
            ui,
            c,
            632.0,
            286.0,
            Align2::RIGHT_CENTER,
            9.0,
            t.value,
            &format!("TIME: {:02}:{:02}", secs / 60, secs % 60),
        );

        // Status bar
        let bar = c.rect(8.0, 298.0, 624.0, 22.0);
        ui.painter().rect_filled(bar, 0.0, t.panel);
        ui::frame(ui, c, bar, t.frame);
        ui::text(ui, c, 16.0, 309.0, Align2::LEFT_CENTER, 9.0, t.status_text, &self.status_line());
    }

    fn windows(&mut self, ctx: &egui::Context) {
        let t = THEMES[self.theme];
        builder::show(ctx, &mut self.show_builder, &mut self.opts, t, self.elevated, &self.source, &self.dest);

        egui::Window::new("FILTERS").open(&mut self.show_filters).collapsible(false).show(ctx, |ui| {
            if ui.button("COPY BUILDER...").clicked() {
                self.show_builder = true;
            }
            ui.separator();
            let o = &mut self.opts;
            ui.checkbox(&mut o.restartable, "Restartable mode (/Z)");
            ui.checkbox(&mut o.backup, "Backup mode (/B)");
            ui.checkbox(&mut o.list_only, "List only, copy nothing (/L)");
            ui.separator();
            for (label, value, hint) in [
                ("Files     ", &mut o.files, "e.g. *.doc *.txt (empty = all)"),
                ("Excl.files", &mut o.exclude_files, "e.g. *.tmp *.bak"),
                ("Excl.dirs ", &mut o.exclude_dirs, "e.g. node_modules .git"),
                ("Extra args", &mut o.extra, "raw robocopy switches"),
            ] {
                ui.horizontal(|ui| {
                    ui.label(label);
                    ui.add(egui::TextEdit::singleline(value).hint_text(hint).desired_width(260.0));
                });
            }
        });

        egui::Window::new("LOG").open(&mut self.show_log).default_size([560.0, 300.0]).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Log file");
                ui.add(
                    egui::TextEdit::singleline(&mut self.opts.log_file)
                        .hint_text("not saved to disk")
                        .desired_width(280.0),
                );
                if ui.button("CHOOSE").clicked() {
                    let mut dialog = rfd::FileDialog::new()
                        .set_file_name("RoboXCopy.log")
                        .add_filter("Log file", &["log", "txt"]);
                    if let Some(dir) = std::path::Path::new(self.opts.log_file.trim()).parent().filter(|d| d.is_dir()) {
                        dialog = dialog.set_directory(dir);
                    }
                    if let Some(path) = dialog.save_file() {
                        self.opts.log_file = path.display().to_string();
                    }
                }
                if ui.button("NONE").clicked() {
                    self.opts.log_file.clear();
                }
            });
            ui.checkbox(&mut self.opts.log_append, "Append to the file instead of replacing it");
            ui.checkbox(&mut self.opts.log_unicode, "Unicode file (UTF-16): keeps accented and non-Latin names");
            ui.label(RichText::new("Applies to the next START. This window keeps showing the live output.").small());
            ui.separator();
            // Leave room for the CLEAR button under the scroll area.
            egui::ScrollArea::both()
                .max_height((ui.available_height() - 34.0).max(60.0))
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show_rows(
                ui,
                ui.text_style_height(&egui::TextStyle::Monospace),
                self.log.len(),
                |ui, range| {
                    for line in &self.log[range] {
                        ui.add(egui::Label::new(RichText::new(line).small()).wrap_mode(egui::TextWrapMode::Extend));
                    }
                },
            );
            if ui.button("CLEAR").clicked() {
                self.log.clear();
            }
        });

        let rights = if self.elevated { "running as ADMIN" } else { "running as normal user" };
        egui::Window::new("HELP").open(&mut self.show_help).collapsible(false).resizable(false).show(ctx, |ui| {
            ui.set_max_width(420.0);
            ui.label(RichText::new("ROBO-XCOPY professional").color(t.value).heading());
            ui.label(format!("version {}  -  {rights}", env!("CARGO_PKG_VERSION")));
            ui.label("A front end for Windows robocopy, in the spirit of Amiga X-Copy.");
            ui.separator();
            ui.label("1. Pick SOURCE and TARGET: click a drive icon, type a path or press BROWSE.");
            ui.label("2. Set MODE and the other fields: click a value, or use its arrows.");
            ui.label("   ATTRIBS and UPDATE open the COPY BUILDER, with every flag explained.");
            ui.label("3. Press DRY RUN to preview, then START.");
            ui.label("4. The grids show the progress; LOG has the full output and the log file.");
            ui.label("Tip: MIRROR deletes files in the target that are not in the source. Always DRY RUN first.");
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("HOW-TO ONLINE").clicked() {
                    sys::open_url(HOWTO_URL);
                }
                if ui.button("PROJECT PAGE").clicked() {
                    sys::open_url(PROJECT_URL);
                }
            });
            ui.label(RichText::new(format!("Theme: {} (press THEME to change)", t.name)).small());
            ui.label(RichText::new("(c) 2026 Cristian Castellari - MIT license").small());
        });

        if let Some(prompt) = self.admin_prompt.take() {
            let mut choice = None;
            egui::Window::new("ADMINISTRATOR RIGHTS")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_max_width(420.0);
                    ui.label(&prompt.reason);
                    ui.label(
                        "Restart Robo-XCopy as administrator? Source and target are kept, \
                         and Windows will ask you to confirm.",
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("RESTART AS ADMIN").clicked() {
                            choice = Some(AdminChoice::Restart);
                        }
                        if prompt.then_start.is_some() && ui.button("CONTINUE ANYWAY").clicked() {
                            choice = Some(AdminChoice::Continue);
                        }
                        if ui.button("CANCEL").clicked() {
                            choice = Some(AdminChoice::Cancel);
                        }
                    });
                });
            match choice {
                None => self.admin_prompt = Some(prompt),
                Some(AdminChoice::Restart) => self.restart_as_admin(ctx),
                Some(AdminChoice::Continue) => {
                    self.admin_declined = true;
                    if let Some(dry_run) = prompt.then_start {
                        self.start(dry_run);
                    }
                }
                Some(AdminChoice::Cancel) => {}
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.pump();
        let ctx = ui.ctx().clone();
        #[cfg(feature = "screenshot")]
        self.shot_tick(&ctx);
        if self.runner.is_running() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        let t = THEMES[self.theme];
        egui::CentralPanel::default().frame(egui::Frame::new().fill(t.bg)).show(ui, |ui| {
            let c = Canvas::fit(ui.max_rect(), t);
            self.draw(ui, &c);
        });
        self.windows(&ctx);
    }
}
