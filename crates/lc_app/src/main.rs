use eframe::egui;
use lc_shell::{init_shell_logging, ShellApp};

struct LevelCraftApp {
    shell: ShellApp,
}

impl eframe::App for LevelCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.shell.ui(ui);
    }
}

fn main() -> eframe::Result {
    let (console_buffer, _writer) = init_shell_logging();
    tracing::info!("Starting Level Craft editor...");

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Level Craft",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(LevelCraftApp {
                shell: ShellApp::from_console(console_buffer),
            }))
        }),
    )
}
