use std::path::PathBuf;

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Id, Margin, Order, Painter, Pos2, Rect, RichText,
    Rounding, Sense, Stroke, Ui,
};

use crate::project::{format_duration, Aspect, Project, Resolution, Store, FRAME_RATES};
use crate::theme;

const BACKGROUNDS: [[u8; 3]; 6] = [
    [0x00, 0x00, 0x00],
    [0x1C, 0x1B, 0x1F],
    [0xFF, 0xFF, 0xFF],
    [0x4F, 0x37, 0x8B],
    [0x0B, 0x4F, 0x6C],
    [0x7D, 0x26, 0x3C],
];

// Status bar and camera cutout aren't reported to us, so leave room at the top
const TOP_INSET: f32 = 36.0;

#[derive(Clone)]
enum Screen {
    Hub,
    Editor(String),
}

struct Draft {
    name: String,
    aspect: Aspect,
    resolution: Resolution,
    fps: u32,
    background: usize,
    blur: bool,
}

enum CardEvent {
    Open,
    Menu,
}

enum MenuAction {
    Duplicate,
    Delete,
    Close,
}

pub struct SplyceApp {
    store: Option<Store>,
    projects: Vec<Project>,
    screen: Screen,
    query: String,
    filter: Option<Aspect>,
    setup: Option<Draft>,
    menu_for: Option<String>,
    notice: Option<String>,
    keyboard: Box<dyn Fn(bool)>,
    keyboard_up: bool,
}

impl SplyceApp {
    pub fn new(data_dir: PathBuf, keyboard: Box<dyn Fn(bool)>) -> Self {
        let store = match Store::open(data_dir) {
            Ok(s) => Some(s),
            Err(e) => {
                log::error!("can't open project storage: {e}");
                None
            }
        };
        let projects = store.as_ref().map(|s| s.load_all()).unwrap_or_default();
        Self {
            store,
            projects,
            screen: Screen::Hub,
            query: String::new(),
            filter: None,
            setup: None,
            menu_for: None,
            notice: None,
            keyboard,
            keyboard_up: false,
        }
    }

    fn persist(&mut self, project: &Project) {
        let Some(store) = &self.store else {
            self.notice = Some("Storage unavailable, changes won't be saved".into());
            return;
        };
        if let Err(e) = store.save(project) {
            log::error!("save failed: {e}");
            self.notice = Some("Couldn't save the project".into());
        }
    }

    fn create_project(&mut self, d: Draft) {
        let name = d.name.trim();
        let name = if name.is_empty() { "Untitled".to_string() } else { name.to_string() };
        let project = Project::new(
            name,
            d.aspect,
            d.resolution,
            d.fps,
            BACKGROUNDS[d.background],
            d.blur,
        );
        self.persist(&project);
        self.screen = Screen::Editor(project.id.clone());
        self.projects.insert(0, project);
    }

    fn duplicate_project(&mut self, id: &str) {
        let Some(copy) = self.projects.iter().find(|p| p.id == id).map(|p| p.duplicate()) else {
            return;
        };
        self.persist(&copy);
        self.projects.insert(0, copy);
    }

    fn delete_project(&mut self, id: &str) {
        if let Some(store) = &self.store {
            if let Err(e) = store.delete(id) {
                log::error!("delete failed: {e}");
            }
        }
        self.projects.retain(|p| p.id != id);
    }

    fn sync_keyboard(&mut self, ctx: &egui::Context) {
        let wanted = ctx.wants_keyboard_input();
        if wanted != self.keyboard_up {
            (self.keyboard)(wanted);
            self.keyboard_up = wanted;
        }
    }

    fn hub(&mut self, ctx: &egui::Context) {
        let mut open = None;
        let mut menu = None;
        let mut new_project = false;

        egui::CentralPanel::default()
            .frame(page_frame())
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(RichText::new("Splyce").size(44.0).strong().color(theme::ACCENT));
                    ui.label(RichText::new("Cut. Animate. Ship.").color(theme::TEXT_DIM));
                    ui.add_space(14.0);

                    let hero = egui::Button::new(
                        RichText::new("+  New Project").size(24.0).strong().color(theme::ON_ACCENT),
                    )
                    .fill(theme::ACCENT)
                    .rounding(Rounding::same(28.0))
                    .min_size(vec2(ui.available_width(), 96.0));
                    if ui.add(hero).clicked() {
                        new_project = true;
                    }

                    ui.add_space(6.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text("Search projects")
                            .desired_width(f32::INFINITY),
                    );

                    ui.horizontal_wrapped(|ui| {
                        if ui.selectable_label(self.filter.is_none(), "All").clicked() {
                            self.filter = None;
                        }
                        for aspect in Aspect::ALL {
                            let on = self.filter == Some(aspect);
                            if ui.selectable_label(on, aspect.label()).clicked() {
                                self.filter = if on { None } else { Some(aspect) };
                            }
                        }
                    });

                    if let Some(msg) = &self.notice {
                        ui.colored_label(Color32::from_rgb(0xFF, 0x8A, 0x80), msg);
                    }

                    let query = self.query.trim().to_lowercase();
                    let visible: Vec<&Project> = self
                        .projects
                        .iter()
                        .filter(|p| self.filter.map_or(true, |a| p.aspect == a))
                        .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query))
                        .collect();

                    if visible.is_empty() {
                        ui.add_space(24.0);
                        let text = if self.projects.is_empty() {
                            "No projects yet. Tap New Project to start."
                        } else {
                            "Nothing matches that search."
                        };
                        ui.label(RichText::new(text).color(theme::TEXT_DIM));
                    }

                    let gap = 12.0;
                    let card_w = (ui.available_width() - gap) / 2.0;
                    for row in visible.chunks(2) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;
                            for p in row {
                                match project_card(ui, p, card_w) {
                                    Some(CardEvent::Open) => open = Some(p.id.clone()),
                                    Some(CardEvent::Menu) => menu = Some(p.id.clone()),
                                    None => {}
                                }
                            }
                        });
                    }
                    ui.add_space(24.0);
                });
            });

        if new_project {
            self.setup = Some(Draft {
                name: format!("Project {}", self.projects.len() + 1),
                aspect: Aspect::Vertical,
                resolution: Resolution::P1080,
                fps: 30,
                background: 0,
                blur: false,
            });
        }
        if let Some(id) = open {
            self.screen = Screen::Editor(id);
        }
        if menu.is_some() {
            self.menu_for = menu;
        }

        self.setup_sheet(ctx);
        self.menu_sheet(ctx);
    }

    fn setup_sheet(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.setup.take() else { return };
        let mut create = false;
        let mut cancel = false;

        let dismissed = bottom_sheet(ctx, "setup", |ui| {
            ui.label(RichText::new("New project").size(26.0).strong());
            ui.add(
                egui::TextEdit::singleline(&mut d.name)
                    .hint_text("Project name")
                    .desired_width(f32::INFINITY),
            );

            heading(ui, &format!("Aspect ratio  ·  {}", d.aspect.caption()));
            ui.horizontal_wrapped(|ui| {
                for a in Aspect::ALL {
                    chip(ui, &mut d.aspect, a, a.label());
                }
            });

            heading(ui, "Resolution");
            ui.horizontal_wrapped(|ui| {
                for r in Resolution::ALL {
                    chip(ui, &mut d.resolution, r, r.label());
                }
            });

            heading(ui, "Frame rate");
            ui.horizontal_wrapped(|ui| {
                for f in FRAME_RATES {
                    chip(ui, &mut d.fps, f, &format!("{f} fps"));
                }
            });

            heading(ui, "Canvas background");
            ui.horizontal_wrapped(|ui| {
                for (i, c) in BACKGROUNDS.iter().enumerate() {
                    let (rect, resp) = ui.allocate_exact_size(vec2(42.0, 42.0), Sense::click());
                    let center = rect.center();
                    ui.painter().circle_filled(center, 16.0, Color32::from_rgb(c[0], c[1], c[2]));
                    let ring = if d.background == i {
                        Stroke::new(2.5, theme::ACCENT)
                    } else {
                        Stroke::new(1.0, theme::TEXT_DIM)
                    };
                    ui.painter().circle_stroke(center, 19.0, ring);
                    if resp.clicked() {
                        d.background = i;
                    }
                }
            });
            ui.checkbox(&mut d.blur, "Blurred background");

            ui.add_space(8.0);
            ui.columns(2, |cols| {
                if cols[0]
                    .add_sized([cols[0].available_width(), 52.0], egui::Button::new("Cancel"))
                    .clicked()
                {
                    cancel = true;
                }
                let go = egui::Button::new(RichText::new("Create").strong().color(theme::ON_ACCENT))
                    .fill(theme::ACCENT);
                if cols[1].add_sized([cols[1].available_width(), 52.0], go).clicked() {
                    create = true;
                }
            });
        });

        if create {
            self.create_project(d);
        } else if !(cancel || dismissed) {
            self.setup = Some(d);
        }
    }

    fn menu_sheet(&mut self, ctx: &egui::Context) {
        let Some(id) = self.menu_for.clone() else { return };
        let Some(name) = self.projects.iter().find(|p| p.id == id).map(|p| p.name.clone()) else {
            self.menu_for = None;
            return;
        };

        let mut action = None;
        let dismissed = bottom_sheet(ctx, "menu", |ui| {
            ui.label(RichText::new(name).size(22.0).strong());
            let w = ui.available_width();
            if ui.add_sized([w, 52.0], egui::Button::new("Duplicate")).clicked() {
                action = Some(MenuAction::Duplicate);
            }
            let del = egui::Button::new(RichText::new("Delete project").color(Color32::from_rgb(0xFF, 0x8A, 0x80)));
            if ui.add_sized([w, 52.0], del).clicked() {
                action = Some(MenuAction::Delete);
            }
            if ui.add_sized([w, 52.0], egui::Button::new("Cancel")).clicked() {
                action = Some(MenuAction::Close);
            }
        });

        match action {
            Some(MenuAction::Duplicate) => {
                self.duplicate_project(&id);
                self.menu_for = None;
            }
            Some(MenuAction::Delete) => {
                self.delete_project(&id);
                self.menu_for = None;
            }
            Some(MenuAction::Close) => self.menu_for = None,
            None if dismissed => self.menu_for = None,
            None => {}
        }
    }

    fn editor(&mut self, ctx: &egui::Context, id: &str) {
        let Some(project) = self.projects.iter().find(|p| p.id == id).cloned() else {
            self.screen = Screen::Hub;
            return;
        };

        let mut back = false;
        egui::CentralPanel::default().frame(page_frame()).show(ctx, |ui| {
            if ui.button("< Back").clicked() {
                back = true;
            }
            ui.label(RichText::new(&project.name).size(30.0).strong());
            ui.label(
                RichText::new(format!(
                    "{}  ·  {}  ·  {} fps",
                    project.aspect.label(),
                    project.resolution.label(),
                    project.fps
                ))
                .color(theme::TEXT_DIM),
            );
            ui.add_space(12.0);

            let max_w = ui.available_width();
            let max_h = ui.available_height() * 0.6;
            let ratio = project.aspect.ratio();
            let (w, h) = if max_w / ratio <= max_h { (max_w, max_w / ratio) } else { (max_h * ratio, max_h) };
            let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
            let [r, g, b] = project.background;
            ui.painter().rect_filled(rect, Rounding::same(12.0), Color32::from_rgb(r, g, b));

            ui.add_space(16.0);
            ui.label(RichText::new("Timeline and tools are coming next.").color(theme::TEXT_DIM));
        });

        if back {
            self.screen = Screen::Hub;
        }
    }
}

impl eframe::App for SplyceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.screen.clone() {
            Screen::Hub => self.hub(ctx),
            Screen::Editor(id) => self.editor(ctx, &id),
        }
        self.sync_keyboard(ctx);
    }
}

fn page_frame() -> egui::Frame {
    egui::Frame::none().fill(theme::SURFACE).inner_margin(Margin {
        left: 16.0,
        right: 16.0,
        top: TOP_INSET,
        bottom: 16.0,
    })
}

fn heading(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).color(theme::TEXT_DIM));
}

fn chip<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, value: T, label: &str) {
    if ui.selectable_label(*current == value, label).clicked() {
        *current = value;
    }
}

fn shorten(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_owned();
    }
    let mut out: String = s.chars().take(max - 1).collect();
    out.push('…');
    out
}

fn badge(painter: &Painter, anchor: Pos2, text: &str, align: Align2) {
    let galley = painter.layout_no_wrap(text.to_owned(), FontId::proportional(11.0), Color32::WHITE);
    let rect = align.anchor_size(anchor, galley.size() + vec2(12.0, 6.0));
    painter.rect_filled(rect, Rounding::same(8.0), Color32::from_black_alpha(170));
    painter.galley(rect.center() - galley.size() / 2.0, galley, Color32::WHITE);
}

fn project_card(ui: &mut Ui, p: &Project, w: f32) -> Option<CardEvent> {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, w * 1.2), Sense::click());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, Rounding::same(20.0), theme::SURFACE_HIGH);

    let thumb = Rect::from_min_size(rect.min + vec2(8.0, 8.0), vec2(w - 16.0, w * 0.75));
    painter.rect_filled(thumb, Rounding::same(12.0), Color32::from_rgb(0x0C, 0x0B, 0x0F));

    // Fit the canvas preview inside the thumbnail without distorting the ratio
    let inner = thumb.shrink(10.0);
    let ratio = p.aspect.ratio();
    let size = if ratio > inner.width() / inner.height() {
        vec2(inner.width(), inner.width() / ratio)
    } else {
        vec2(inner.height() * ratio, inner.height())
    };
    let [r, g, b] = p.background;
    painter.rect_filled(
        Rect::from_center_size(thumb.center(), size),
        Rounding::same(4.0),
        Color32::from_rgb(r, g, b),
    );

    badge(&painter, thumb.left_top() + vec2(6.0, 6.0), p.aspect.label(), Align2::LEFT_TOP);
    badge(
        &painter,
        thumb.right_bottom() - vec2(6.0, 6.0),
        &format_duration(p.duration_ms),
        Align2::RIGHT_BOTTOM,
    );

    let text_y = thumb.bottom() + 10.0;
    painter.text(
        pos2(rect.left() + 12.0, text_y),
        Align2::LEFT_TOP,
        shorten(&p.name, 14),
        FontId::proportional(15.0),
        Color32::WHITE,
    );
    painter.text(
        pos2(rect.left() + 12.0, text_y + 20.0),
        Align2::LEFT_TOP,
        format!("{} · {} fps", p.resolution.label(), p.fps),
        FontId::proportional(12.0),
        theme::TEXT_DIM,
    );

    let menu_rect = Rect::from_center_size(pos2(rect.right() - 24.0, text_y + 18.0), vec2(40.0, 40.0));
    let menu_resp = ui.interact(menu_rect, resp.id.with("menu"), Sense::click());
    for i in -1..=1 {
        painter.circle_filled(menu_rect.center() + vec2(0.0, i as f32 * 5.0), 2.0, theme::TEXT_DIM);
    }

    if menu_resp.clicked() {
        Some(CardEvent::Menu)
    } else if resp.clicked() {
        Some(CardEvent::Open)
    } else {
        None
    }
}

/// Returns true when the dimmed backdrop was tapped.
fn bottom_sheet(ctx: &egui::Context, id: &str, add: impl FnOnce(&mut Ui)) -> bool {
    let screen = ctx.screen_rect();

    let scrim = egui::Area::new(Id::new((id, "scrim")))
        .order(Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter().rect_filled(rect, 0.0, Color32::from_black_alpha(140));
            resp
        });

    egui::Area::new(Id::new((id, "sheet")))
        .order(Order::Tooltip)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(theme::SURFACE_HIGH)
                .rounding(Rounding { nw: 28.0, ne: 28.0, sw: 0.0, se: 0.0 })
                .inner_margin(Margin { left: 20.0, right: 20.0, top: 20.0, bottom: 28.0 })
                .show(ui, |ui| {
                    ui.set_width(screen.width() - 40.0);
                    egui::ScrollArea::vertical()
                        .max_height(screen.height() * 0.75)
                        .show(ui, add);
                });
        });

    scrim.inner.clicked()
}
