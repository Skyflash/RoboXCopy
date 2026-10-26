//! Docs-only helper, compiled with `--features screenshot`; the release exe does not contain it.
//!
//! `RoboXCopy --screenshot out.png [--theme N]` draws a demo state, saves that frame as a PNG straight
//! from the framebuffer (no other window can end up in it) and quits.

use crate::App;
use eframe::egui;
use std::path::PathBuf;
use std::time::Duration;

/// Frames to let layout and fonts settle before grabbing the picture.
const SETTLE_FRAMES: u32 = 8;

pub struct Shot {
    path: PathBuf,
    pub theme: Option<usize>,
    /// `--open help|log|filters|builder|admin`: also show that window.
    open: Option<String>,
    frames: u32,
    requested: bool,
}

impl Shot {
    pub fn from_args() -> Option<Self> {
        let args: Vec<String> = std::env::args().collect();
        let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1));
        Some(Self {
            path: PathBuf::from(value("--screenshot")?),
            theme: value("--theme").and_then(|t| t.parse().ok()),
            open: value("--open").cloned(),
            frames: 0,
            requested: false,
        })
    }
}

impl App {
    pub fn start_shot(&mut self, shot: Shot) {
        self.drives = vec!['C', 'D', 'E', 'F'];
        self.source = r"C:\Users\me\Photos".into();
        self.dest = r"E:\Backup\Photos".into();
        self.files_done = 52;
        self.file_percent = 0.6;
        self.current = r"C:\Users\me\Photos\2026\holiday\IMG_0452.jpg".into();
        self.status = "Copying...".into();
        self.elapsed = Duration::from_secs(26);
        self.demo = true;
        self.opts.log_file = r"D:\Logs\photos-backup.log".into();
        match shot.open.as_deref() {
            Some("help") => self.show_help = true,
            Some("log") => {
                self.show_log = true;
                self.log = vec![
                    r"> robocopy C:\Users\me\Photos E:\Backup\Photos /E /MT:8 /R:3 /W:2 /LOG:D:\Logs\photos-backup.log /TEE".into(),
                    "\t\t    New File \t\t 2845021\tIMG_0452.jpg".into(),
                    "\t\t    New File \t\t 3011840\tIMG_0453.jpg".into(),
                    "\t\t    New File \t\t  118233\tnotes.txt".into(),
                ];
            }
            Some("filters") => self.show_filters = true,
            Some("builder") => {
                self.demo = false;
                self.show_builder = true;
                self.opts.copy_flags = "DATS".into();
                self.opts.dcopy = "T".into();
                self.opts.set("/XO", true);
                self.opts.set("/FFT", true);
                self.opts.values.insert("/MAXAGE:", "30".into());
                self.opts.log_file.clear();
            }
            Some("admin") => {
                self.demo = false;
                self.admin_prompt = Some(crate::AdminPrompt {
                    reason: "Backup mode (/B) and the auditing copy flag (U, part of /COPYALL) need administrator rights: \
                             without them robocopy does not even start."
                        .into(),
                    then_start: Some(false),
                });
            }
            _ => {}
        }
        self.shot = Some(shot);
    }

    pub fn shot_tick(&mut self, ctx: &egui::Context) {
        let Some(shot) = self.shot.as_mut() else { return };
        ctx.request_repaint();
        shot.frames += 1;
        if !shot.requested && shot.frames >= SETTLE_FRAMES {
            shot.requested = true;
            let (path, done_ctx) = (shot.path.clone(), ctx.clone());
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::ScreenshotCallback::new(move |image| {
                let [w, h] = image.size;
                let saved = image::save_buffer(&path, image.as_raw(), w as u32, h as u32, image::ColorType::Rgba8);
                if let Err(e) = saved {
                    eprintln!("cannot save {}: {e}", path.display());
                }
                done_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            })));
        }
    }
}
