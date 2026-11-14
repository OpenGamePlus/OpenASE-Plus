#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gui;

fn main() -> eframe::Result {
    // An `openase://` link (from the browser) goes to the running instance
    // if there is one; otherwise this instance opens it.
    let link = std::env::args()
        .skip(1)
        .find(|arg| openase::links::parse(arg).is_some());
    if let Some(url) = &link
        && openase::instance::forward(&openase::instance::port_file(), url)
    {
        return Ok(());
    }
    let mut viewport = eframe::egui::ViewportBuilder::default();
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")) {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport: viewport
            .with_title("OpenASE-Plus")
            .with_inner_size([1280.0, 760.0])
            .with_min_inner_size([960.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "OpenASE-Plus",
        options,
        Box::new(|cc| Ok(Box::new(gui::OpenAseApp::new(cc, link)))),
    )
}
