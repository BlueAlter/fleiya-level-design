use eframe::egui;
use lc_shell::ShellApp;

struct LevelCraftApp {
    shell: ShellApp,
}

impl eframe::App for LevelCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.shell.ui(ui);
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Level Craft",
        native_options,
        Box::new(|_cc| {
            Ok(Box::new(LevelCraftApp {
                shell: ShellApp::default(),
            }))
        }),
    )
}
