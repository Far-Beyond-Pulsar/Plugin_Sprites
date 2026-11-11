//! What the sprites plugin registers for the editor (the `editor` feature):
//! its level editor entries.

use sprites_editor::level_editor;
use ui_level_editor::extend::CreateEntry;

/// Everything the sprites plugin registers for the editor.
pub(crate) fn build(p: &mut pulsar_plugins::PluginBuilder) {
    // ── Level editor ────────────────────────────────────────────────────────

    // The hierarchy's quick-add: a default sprite at the centre of the view.
    p.add(CreateEntry {
        origin: p.origin(),
        id: "sprite",
        menu_label: None,
        environment_label: None,
        quick_add: Some(level_editor::QUICK_ADD),
        offered: None,
        order: 500,
        create: level_editor::create_default_sprite,
    });
}
