use eframe::egui;

struct LevelCraftApp;

impl eframe::App for LevelCraftApp {
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Level Craft",
        native_options,
        Box::new(|_cc| Ok(Box::new(LevelCraftApp))),
    )
}
