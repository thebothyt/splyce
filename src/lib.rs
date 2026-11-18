use eframe::egui;

mod theme;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    if let Err(err) = eframe::run_native(
        "Splyce",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(SplyceApp::default()))
        }),
    ) {
        log::error!("failed to start: {err}");
    }
}

#[derive(Default)]
struct SplyceApp {
    taps: u32,
}

impl eframe::App for SplyceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(64.0);
            ui.vertical_centered(|ui| {
                ui.heading("Splyce");
                ui.label("Build pipeline check");
                ui.add_space(24.0);
                if ui.button("Tap me").clicked() {
                    self.taps += 1;
                }
                ui.label(format!("Taps: {}", self.taps));
            });
        });
    }
}
