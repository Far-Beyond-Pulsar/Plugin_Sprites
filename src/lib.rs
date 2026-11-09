//! Sprites (`pulsar.sprites`): the plugin's root crate, the one place the
//! plugin registers everything it contributes.
//!
//! The plugin's other crates hold the code: `helio-pass-sprite-batch` (the
//! feature: culling, batching, the 2D overlay and the join that places
//! sprites by their owners), `sprites_component` (the SceneDB component, its
//! row and its image binding) and `sprites_editor` (placing a sprite in the
//! level editor). None of them registers a contribution; this crate imports
//! them and submits their registrations, so this file and the two modules
//! below are the whole list of what the plugin brings:
//!
//! - [`runtime`]: what a game links. The component class, the Helio feature,
//!   the scene rows and the texture binding.
//! - `editor` (with the `editor` feature): what the editor adds. The
//!   hierarchy's sprite quick-add.
//!
//! Every registration names this crate as its origin
//! (`pulsar_plugins::origin!()`), so the plugin gate skips all of them while
//! a project disables the plugin, and the plugin catalog lists them under
//! `pulsar.sprites`. Other plugins never depend on this crate; hosts reach
//! it through the installed set or a generated game, a game without the
//! `editor` feature.

// The plugin's descriptor (`pulsar-plugin.toml`): linking this crate installs
// the plugin; a project enables it or not (`pulsar_plugins`).
pulsar_plugins::installed!("pulsar-plugin.toml");

pub use sprites_component;
#[cfg(feature = "editor")]
pub use sprites_editor;

pub mod runtime;

#[cfg(feature = "editor")]
pub mod editor;
