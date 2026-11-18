mod app;
mod project;
mod theme;

#[cfg(target_os = "android")]
use winit::platform::android::{activity::AndroidApp, EventLoopBuilderExtAndroid};

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let data_dir = app
        .internal_data_path()
        .unwrap_or_else(std::env::temp_dir)
        .join("projects");

    // NativeActivity never raises the keyboard on its own
    let ime = app.clone();
    let keyboard: Box<dyn Fn(bool)> = Box::new(move |show| {
        if show {
            ime.show_soft_input(true);
        } else {
            ime.hide_soft_input(true);
        }
    });

    // eframe 0.29 takes the activity through the event loop builder hook
    let options = eframe::NativeOptions {
        event_loop_builder: Some(Box::new(move |builder| {
            builder.with_android_app(app);
        })),
        ..Default::default()
    };

    if let Err(err) = eframe::run_native(
        "Splyce",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(app::SplyceApp::new(data_dir, keyboard)))
        }),
    ) {
        log::error!("failed to start: {err}");
    }
}
