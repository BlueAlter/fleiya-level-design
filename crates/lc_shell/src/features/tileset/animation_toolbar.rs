//! Toolbar controls for animation playback, modes, duration, and preview visibility.

use egui::{DragValue, RichText, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::{AnimationPlayMode, TilesetDef};
use lc_feature_tileset::SetTileAnimationCommand;

use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};
use crate::ShellApp;

/// Renders the animation toolbar row adjacent to the TILE ATLAS mode selector.
pub fn render_animation_toolbar(app: &mut ShellApp, ui: &mut Ui, t: &Tokens, tileset: &TilesetDef) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        ui.label(
            RichText::new("TOOLS")
                .size(11.0)
                .strong()
                .color(t.accent_ink),
        );

        ui.add_space(2.0);

        // 1. Play / Pause button
        let play_icon = if app.tileset_view_state.is_animation_playing {
            Icon::Pause
        } else {
            Icon::Play
        };
        let play_tip = if app.tileset_view_state.is_animation_playing {
            "Pause animation playback"
        } else {
            "Play animation"
        };
        if icon_button(ui, play_icon, 22.0, IconButtonStyle::bordered())
            .on_hover_text(play_tip)
            .clicked()
        {
            app.tileset_view_state.is_animation_playing =
                !app.tileset_view_state.is_animation_playing;
            if app.tileset_view_state.is_animation_playing {
                app.tileset_view_state.animation_start_time = std::time::Instant::now();
            }
        }

        // Active anchor animation (if any selected or first available)
        let active_anchor_id = app
            .tileset_view_state
            .selected_animation_anchor
            .or_else(|| tileset.animations.keys().next().copied());

        let current_anim = active_anchor_id.and_then(|id| tileset.animations.get(&id).cloned());

        if let Some(mut anim) = current_anim {
            let mut changed = false;

            // 2. Playback mode selector
            let mode_label = match anim.mode {
                AnimationPlayMode::Loop => "Loop",
                AnimationPlayMode::PingPong => "Ping-Pong",
                AnimationPlayMode::Once => "Once",
            };

            egui::ComboBox::from_id_salt("anim_play_mode_combo")
                .selected_text(RichText::new(mode_label).size(11.0).color(t.ink_strong))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut anim.mode, AnimationPlayMode::Loop, "Loop")
                        .clicked()
                    {
                        changed = true;
                    }
                    if ui
                        .selectable_value(&mut anim.mode, AnimationPlayMode::PingPong, "Ping-Pong")
                        .clicked()
                    {
                        changed = true;
                    }
                    if ui
                        .selectable_value(&mut anim.mode, AnimationPlayMode::Once, "Once")
                        .clicked()
                    {
                        changed = true;
                    }
                });

            // 3. Frame duration
            ui.label(RichText::new("Dur:").size(11.0).color(t.muted));
            let mut dur = anim.frame_duration_ms;
            let dur_resp = ui.add(
                DragValue::new(&mut dur)
                    .range(10..=5000)
                    .speed(5.0)
                    .suffix(" ms"),
            );
            if dur_resp.changed() {
                anim.frame_duration_ms = dur;
                changed = true;
            }

            // 4. Random start phase toggle
            let rnd_style = if anim.random_start {
                IconButtonStyle {
                    danger: false,
                    selected: true,
                    bordered: true,
                }
            } else {
                IconButtonStyle::bordered()
            };
            if icon_button(ui, Icon::Sparkle, 22.0, rnd_style)
                .on_hover_text(if anim.random_start {
                    "Random start offset: Enabled"
                } else {
                    "Random start offset: Disabled"
                })
                .clicked()
            {
                anim.random_start = !anim.random_start;
                changed = true;
            }

            if changed {
                app.execute_command(Box::new(SetTileAnimationCommand::new(tileset.id, anim)));
            }
        }

        // 5. Preview visibility toggle
        let preview_icon = if app.tileset_view_state.show_animation_preview {
            Icon::Eye
        } else {
            Icon::EyeOff
        };
        let preview_tip = if app.tileset_view_state.show_animation_preview {
            "Hide animation preview card"
        } else {
            "Show animation preview card"
        };
        let preview_style = if app.tileset_view_state.show_animation_preview {
            IconButtonStyle {
                danger: false,
                selected: true,
                bordered: true,
            }
        } else {
            IconButtonStyle::bordered()
        };
        if icon_button(ui, preview_icon, 22.0, preview_style)
            .on_hover_text(preview_tip)
            .clicked()
        {
            app.tileset_view_state.show_animation_preview =
                !app.tileset_view_state.show_animation_preview;
        }
    });
}
