//! Sprites (`pulsar.sprites`): the editor crate -- placing a sprite (the
//! hierarchy's quick-add, [`level_editor`]). It registers nothing itself:
//! the plugin's root crate (`sprites_plugin`, with its `editor` feature)
//! submits it through the level editor's extension points. The inspector
//! edits `SpriteComponent` through its reflected properties.

pub mod level_editor;
