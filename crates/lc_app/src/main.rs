use eframe::egui;
use lc_shell::{init_shell_logging, theme, ShellApp};

struct LevelCraftApp {
    shell: ShellApp,
    /// `Context::set_fonts` only takes effect on the next frame.
    fonts_ready: bool,
}

impl eframe::App for LevelCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.fonts_ready {
            self.fonts_ready = true;
            // Fonts installed at startup become active on the next frame.
            ui.ctx().request_repaint();
            return;
        }
        self.shell.ui(ui);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }
}

fn main() -> eframe::Result {
    let (console_buffer, _writer) = init_shell_logging();
    tracing::info!("Starting Level Craft editor...");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Level Craft",
        native_options,
        Box::new(move |cc| {
            theme::install_fonts(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx, theme::ThemeChoice::Dark);

            let mut shell = ShellApp::from_console(console_buffer);

            let project_path_arg = std::env::args().nth(1);
            let candidate_path = project_path_arg.map(std::path::PathBuf::from).or_else(|| {
                let sample = std::path::PathBuf::from("examples/sample_project/project.lcproj");
                if sample.exists() {
                    Some(sample)
                } else {
                    None
                }
            });

            if let Some(ref path) = candidate_path {
                if let Err(e) = shell.open_project_from_path(path) {
                    tracing::warn!("Failed to auto-load project {}: {e}", path.display());
                } else {
                    shell.open_first_level();
                }
            }

            Ok(Box::new(LevelCraftApp {
                shell,
                fonts_ready: false,
            }))
        }),
    )
}
