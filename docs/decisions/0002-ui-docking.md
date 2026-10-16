# ADR-0002 — UI docking (status: PROPOSED, finalized by spike in T-0008)
Preferred: egui_dock. Fallback if the spike shows single-DockState cannot restrict drops or collapse the bottom dock:
documents in an egui_dock instance (central area); left/right/bottom docks as a small custom tabbed-panel container with drag-to-move between slots
(egui `SidePanel`/`TopBottomPanel` + own tab strip). `PanelView` / `DocumentView` traits stay identical either way, so features are unaffected.
