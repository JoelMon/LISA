use eframe::egui;
use egui::Context;

pub fn report_win() {
    let ctx = &egui::Context::default();
    egui::Window::new("My Window").show(ctx, |ui| {
        ui.label("Hello World!");
    });
}
