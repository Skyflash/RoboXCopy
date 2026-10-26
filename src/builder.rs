//! The COPY BUILDER window: pick copy flags, selection rules and other switches with a description
//! next to each, and see the resulting command line.

use crate::options::{COPY_ALL, COPY_DEFAULT, COPY_FLAGS, DCOPY_FLAGS, OTHER, Options, RULES, VALUES};
use crate::theme::Theme;
use eframe::egui::{self, RichText};

fn section(ui: &mut egui::Ui, t: &Theme, title: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(title).color(t.heading).strong());
    ui.separator();
}

fn help(ui: &mut egui::Ui, t: &Theme, text: &str) {
    ui.label(RichText::new(text).small().color(t.label.gamma_multiply(0.75)));
}

pub fn show(ctx: &egui::Context, open: &mut bool, o: &mut Options, t: &Theme, elevated: bool, src: &str, dst: &str) {
    let max_height = (ctx.content_rect().height() - 90.0).max(200.0);
    egui::Window::new("COPY BUILDER").open(open).collapsible(false).default_width(560.0).show(ctx, |ui| {
        ui.set_max_width(560.0);
        egui::ScrollArea::vertical().max_height(max_height).auto_shrink([false, true]).show(ui, |ui| {
            section(ui, t, "WHAT TO COPY   /COPY");
            ui.horizontal_wrapped(|ui| {
                ui.label("Presets:");
                for (label, flags) in [("DAT  default", COPY_DEFAULT), ("DATS", "DATS"), ("DATSO", "DATSO"), ("ALL  /COPYALL", COPY_ALL)] {
                    if ui.selectable_label(o.copy_flags == flags, label).clicked() {
                        o.copy_flags = flags.into();
                    }
                }
            });
            for (c, name, text) in COPY_FLAGS {
                let mut on = o.copy_flags.contains(*c);
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut on, format!("{c}  {name}")).changed() {
                        o.toggle_copy_flag(*c);
                    }
                    help(ui, t, text);
                });
            }

            section(ui, t, "FOLDERS   /DCOPY");
            help(ui, t, "Leave all off to keep robocopy's own default.");
            for (c, name, text) in DCOPY_FLAGS {
                let mut on = o.dcopy.contains(*c);
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut on, format!("{c}  {name}")).changed() {
                        o.toggle_dcopy(*c);
                    }
                    help(ui, t, text);
                });
            }

            section(ui, t, "WHICH FILES   selection rules");
            ui.horizontal_wrapped(|ui| {
                ui.label("Presets:");
                if ui.selectable_label(o.update_label() == "ALL", "ALL FILES").clicked() {
                    for r in RULES {
                        o.set(r.flag, false);
                    }
                }
                if ui.selectable_label(o.update_label() == "NEWER", "ONLY NEWER  /XO").clicked() {
                    for r in RULES {
                        o.set(r.flag, r.flag == "/XO");
                    }
                }
            });
            switches(ui, t, o, RULES);

            section(ui, t, "OTHER");
            switches(ui, t, o, OTHER);
            for v in VALUES {
                ui.horizontal(|ui| {
                    ui.label(format!("{:<9}{}", v.label, v.flag.trim_end_matches(':')));
                    ui.add(egui::TextEdit::singleline(o.values.entry(v.flag).or_default()).hint_text(format!("e.g. {}", v.example)).desired_width(120.0));
                    help(ui, t, v.help);
                });
            }

            section(ui, t, "COMMAND");
            ui.add(egui::Label::new(RichText::new(o.preview(src, dst)).monospace().small().color(t.value)).wrap());
            if o.needs_admin() && !elevated {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("These options need administrator rights: use RUN AS ADMIN (top right).").color(t.sub),
                );
            }
            if o.threads_off_for_pause() {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Pause (/IPG) cannot be used with several threads: this copy will run on one.").color(t.sub),
                );
            }
        });
    });
}

fn switches(ui: &mut egui::Ui, t: &Theme, o: &mut Options, table: &'static [crate::options::Switch]) {
    for s in table {
        let mut on = o.has(s.flag);
        ui.horizontal(|ui| {
            if ui.checkbox(&mut on, format!("{:<16}{}", s.label, s.flag)).changed() {
                o.set(s.flag, on);
            }
            help(ui, t, s.help);
        });
    }
}
