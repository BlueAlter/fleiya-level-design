//! Transient UI state for the Layers Workspace.

use std::collections::{BTreeMap, HashSet};

use egui::Pos2;
use lc_core::auto_rules::ANY_NON_ZERO_PATTERN;
use lc_core::grid::Grid;
use lc_core::id::{AutoRuleGroupId, AutoRuleId, LayerDefId};
use lc_core::layer::LayerKind;
use lc_feature_layers::AutoRuleWizardPreset;

use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::markdown_document::{new_document_cache, DocumentCache};
use crate::shared::ui::tile_picker::TilePicker;

/// Active workbench view inside the Center Area of the `AUTO TILE` tab.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CenterAutoMode {
    /// Edit the selected rule's pattern matrix & output tiles (LDtk Rule Editor).
    #[default]
    RuleEditor,
    /// Open the full `AUTO-LAYER RULES ASSISTANT` (mapping IntGrid & autotile slots to Tileset).
    RulesAssistant,
}

/// Persistent transient state for the Center `AUTO-LAYER RULES ASSISTANT`.
#[derive(Clone, Debug)]
pub struct RulesAssistantState {
    /// Target layer definition receiving the generated group.
    pub layer_id: LayerDefId,
    /// Group name (`2. GROUP OF RULES NAME`).
    pub group_name: String,
    /// Main IntGrid value (`1. INTGRID VALUES -> Main IntGrid value`).
    pub int_grid_value: i32,
    /// Optional outer IntGrid value (`None` = Any other / Empty).
    pub outer_int_grid_value: Option<i32>,
    /// Selected preset / slot mode.
    pub preset: AutoRuleWizardPreset,
    /// Currently selected slot index inside `AUTOTILE_ASSISTANT_SLOTS` (`3. AUTO-RENDERED TILES`).
    pub selected_slot_idx: Option<usize>,
    /// Mapped tile IDs per `AUTOTILE_ASSISTANT_SLOTS` index.
    pub slot_tiles: BTreeMap<usize, u32>,
    /// Fallback sample tile IDs for quick non-slot presets.
    pub sample_tiles: Vec<u32>,
    /// `4. OPTIONS -> Allow rules to stack` (`!break_on_match`).
    pub allow_stack: bool,
}

impl RulesAssistantState {
    /// Creates a fresh assistant state seeded for `layer_id` and `default_val`.
    pub fn for_layer(layer_id: LayerDefId, default_val: i32, default_name: &str) -> Self {
        Self {
            layer_id,
            group_name: default_name.to_string(),
            int_grid_value: default_val,
            outer_int_grid_value: None,
            preset: AutoRuleWizardPreset::AssistantSlots,
            selected_slot_idx: Some(5), // Center Fill selected by default
            slot_tiles: BTreeMap::new(),
            sample_tiles: vec![0, 1, 2, 3],
            allow_stack: false,
        }
    }
}

/// Transient view state for the Layers Workspace (never persisted to `Project`).
#[derive(Clone)]
pub struct LayerViewState {
    /// Currently selected layer definition in the left list.
    pub selected_layer_id: Option<LayerDefId>,
    /// Left panel search filter (matches layer name, kind, or `#tag`).
    pub search_query: String,
    /// Active detail sub-tab (`0 = LAYERS DEF`, `1 = AUTO TILE`, `2 = DOCUMENT`).
    pub active_sub_tab: usize,
    /// Open context menu `(layer_id, popup_pos)`.
    pub context_menu: Option<(LayerDefId, Pos2)>,
    /// Pending new-layer kind picker modal open flag.
    pub show_new_layer_modal: bool,
    /// Pending delete confirmation target `(id, name, non_empty_levels, placed_items)`.
    pub pending_delete: Option<(LayerDefId, String, usize, usize)>,
    /// Pending kind conversion confirmation `(layer_id, target_kind, non_empty_levels, placed_items)`.
    pub pending_kind_change: Option<(LayerDefId, LayerKind, usize, usize)>,
    /// Pending `MOVE EXISTING ENTITIES TO ANOTHER LAYER` modal `(from_layer_id, target_layer_id)`.
    pub pending_move_entities: Option<(LayerDefId, Option<LayerDefId>)>,
    /// Pending `BAKE AUTO-LAYER` modal target layer ID.
    pub pending_bake_auto: Option<LayerDefId>,
    /// Error banner message, if any.
    pub error_message: Option<String>,
    /// Cached markdown state for the `DOCUMENT` sub-tab.
    pub doc_cache: DocumentCache,
    /// Vertical scroll offset for the `DOCUMENT` sub-tab.
    pub doc_scroll: f32,
    /// Collapse state of `GENERAL LAYER SETTINGS`.
    pub collapse_settings: bool,
    /// Collapse state of `DISPLAY OPTIONS`.
    pub collapse_visual: bool,
    /// Collapse state of `<TYPE> SETTINGS`.
    pub collapse_kind_config: bool,
    /// Drag reorder state for IntGrid value rows.
    pub intgrid_drag: DragListState,
    /// Active tile picker for an IntGrid value icon `(layer_id, value_index, TilePicker)`.
    pub intgrid_tile_picker: Option<(LayerDefId, usize, TilePicker)>,

    // --- Auto-Tile Tab State ---
    /// Active center area mode (`RuleEditor` vs `RulesAssistant`).
    pub center_auto_mode: CenterAutoMode,
    /// Assistant configuration when `center_auto_mode == CenterAutoMode::RulesAssistant`.
    pub assistant: RulesAssistantState,
    /// Currently selected rule group in the `AUTO TILE` tab.
    pub selected_group_id: Option<AutoRuleGroupId>,
    /// Currently selected rule in the `AUTO TILE` tab.
    pub selected_rule_id: Option<AutoRuleId>,
    /// Rule currently hovered or selected for purple sandbox highlight.
    pub hovered_rule_id: Option<AutoRuleId>,
    /// Collapsed auto-rule group IDs.
    pub collapsed_rule_groups: HashSet<AutoRuleGroupId>,
    /// Active IntGrid condition painted into the pattern matrix (`1..` or `ANY_NON_ZERO_PATTERN` or `GROUP_PATTERN_BASE + uid`).
    pub pattern_brush_value: i32,
    /// Drag reorder state for rule groups.
    pub rule_group_drag: DragListState,
    /// Drag reorder state for rules inside the selected group.
    pub rule_drag: DragListState,
    /// Active tile picker modal for picking rule output tiles `(layer_id, group_id, rule_id, TilePicker)`.
    pub auto_tile_picker: Option<(LayerDefId, AutoRuleGroupId, AutoRuleId, TilePicker)>,
    /// Whether the `[x] Render` toggle renders auto-tiled output in Live Sandbox.
    pub show_render_preview: bool,
    /// Whether the green `Enable help` strip is expanded in the Rule Pattern Editor.
    pub show_rule_help: bool,
    /// Live 7x7 mini sandbox IntGrid used to preview Auto-Tile evaluation in real time.
    pub sandbox_grid: Grid<i32>,
    /// Seed used for live sandbox Auto-Tile evaluation.
    pub sandbox_seed: u64,
    /// Active biome filter tags simulated in the live sandbox preview.
    pub sandbox_biomes_text: String,
    /// IntGrid value painted on left-click in the live sandbox preview.
    pub sandbox_paint_value: i32,
}

impl Default for LayerViewState {
    fn default() -> Self {
        let mut sandbox_grid = Grid::new(7, 7, 0);
        for y in 2..=4 {
            for x in 2..=4 {
                let _ = sandbox_grid.set(x, y, 1);
            }
        }
        Self {
            selected_layer_id: None,
            search_query: String::new(),
            active_sub_tab: 0,
            context_menu: None,
            show_new_layer_modal: false,
            pending_delete: None,
            pending_kind_change: None,
            pending_move_entities: None,
            pending_bake_auto: None,
            error_message: None,
            doc_cache: new_document_cache(),
            doc_scroll: 0.0,
            collapse_settings: false,
            collapse_visual: true,
            collapse_kind_config: false,
            intgrid_drag: DragListState::new(),
            intgrid_tile_picker: None,
            center_auto_mode: CenterAutoMode::RuleEditor,
            assistant: RulesAssistantState::for_layer(LayerDefId(0), 1, "AutoTerrain"),
            selected_group_id: None,
            selected_rule_id: None,
            hovered_rule_id: None,
            collapsed_rule_groups: HashSet::new(),
            pattern_brush_value: ANY_NON_ZERO_PATTERN,
            rule_group_drag: DragListState::new(),
            rule_drag: DragListState::new(),
            auto_tile_picker: None,
            show_render_preview: true,
            show_rule_help: false,
            sandbox_grid,
            sandbox_seed: 1,
            sandbox_biomes_text: String::new(),
            sandbox_paint_value: 1,
        }
    }
}

impl LayerViewState {
    /// Selects a layer definition and clears per-layer transient selections/popups.
    pub fn select_layer(&mut self, id: LayerDefId) {
        if self.selected_layer_id != Some(id) {
            self.selected_layer_id = Some(id);
            self.context_menu = None;
            self.pending_delete = None;
            self.pending_kind_change = None;
            self.pending_move_entities = None;
            self.pending_bake_auto = None;
            self.intgrid_tile_picker = None;
            self.selected_group_id = None;
            self.selected_rule_id = None;
            self.hovered_rule_id = None;
            self.auto_tile_picker = None;
            self.center_auto_mode = CenterAutoMode::RuleEditor;
        }
    }
}
