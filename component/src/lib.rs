//! 2D sprites (`pulsar.sprites`): the component crate of the plugin that
//! owns sprites.
//!
//! - The component ([`component`]): `SpriteComponent`, an image or a tint
//!   drawn over the 3D scene in a 2D screen space, placed by its owner.
//! - Its row ([`SpriteSourceRow`]): the sprite passes' row SceneDB derives
//!   from the authored value, with the image's texture store slot; the
//!   plugin's join places it at the owner (`helio_pass_sprite_batch::scene_rows`).
//! - What the engine runs for them: the plugin's Helio feature
//!   ([`render_features`]: `helio_pass_sprite_batch::SpriteFeature`), its
//!   scene rows ([`register_columns`], [`derivation`]) and how a sprite
//!   takes in a loaded image ([`register_texture_binding`]).
//!
//! This crate registers nothing itself but its type's SceneDB attachments
//! (GPU mirror and clear, handle ledger): the plugin's root crate
//! (`sprites_plugin`) submits the component class and every registration
//! above, in one place.

pub mod component;

pub use component::*;

use pulsar_scenedb::gpu::{GpuMirrorHandle, SceneGpuStore};
use pulsar_scenedb_derive::SceneStore;

/// Where the sprites' source rows live.
pub const SPRITE_SOURCES_BUFFER: &str = "sprite_sources";

/// A 2D sprite: the sprite passes' row
/// (`helio_pass_sprite_batch::SpriteRow`, bit for bit) at the 2D
/// origin, which the join moves to its owner's X and Y, turns by the
/// owner's roll and scales by its X and Y scale. Its image is registered in
/// the scene's texture store as the row is derived (its slot in
/// `atlas_layer`). A disabled sprite's row is zero, which the join skips.
#[derive(SceneStore, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug, PartialEq)]
#[repr(C)]
#[gpu(layout = packed, buffer = "sprite_sources")]
pub struct SpriteSourceRow {
    /// Position, size, rotation, depth, padding, UV rectangle and tint.
    #[gpu]
    pub sprite: [f32; 16],
    /// The image's texture store slot and padding.
    #[gpu]
    pub texture: [u32; 4],
}

impl SpriteSourceRow {
    pub fn of(sprite: &SpriteComponent, texture_slot: u32) -> Self {
        bytemuck::pod_read_unaligned(bytemuck::bytes_of(&sprite.to_row(texture_slot)))
    }
}

// A sprite's image is registered in the scene's texture store as its row is
// derived, so the row carries its bindless slot.
fn sprite_dispatch(mirror: &GpuMirrorHandle, row: u32, data: *const (), is_new_insert: bool) {
    // SAFETY: SceneDB reaches this only through `SpriteComponent`'s own
    // `ComponentId`, with a pointer to a live value.
    let sprite = unsafe { &*(data as *const SpriteComponent) };
    let slot = if sprite.enabled {
        helio_component::texture_assets::texture_slot(
            &sprite.image,
            &sprite.texture,
            mirror,
            "sprite",
        )
    } else {
        u32::MAX
    };
    pulsar_scenedb::gpu::write_derived_row(
        mirror,
        row,
        &SpriteSourceRow::of(sprite, slot),
        is_new_insert,
    );
}

fn sprite_clear(mirror: &GpuMirrorHandle, row: u32) {
    pulsar_scenedb::gpu::clear_derived_row::<SpriteSourceRow>(mirror, row);
}

pulsar_scenedb::pulsar_reflection::inventory::submit! {
    pulsar_scenedb::gpu::GpuMirrorRegistration {
        component_id: pulsar_scenedb::component_id::<SpriteComponent>,
        dispatch: sprite_dispatch,
    }
}

pulsar_scenedb::pulsar_reflection::inventory::submit! {
    pulsar_scenedb::gpu::GpuClearRegistration {
        component_id: pulsar_scenedb::component_id::<SpriteComponent>,
        clear: sprite_clear,
    }
}

/// Registers how a sprite takes in a loaded image.
pub fn register_texture_binding() {
    pulsar_scenedb::register_binding::<SpriteComponent>(
        helio_component::texture_assets::TEXTURE_ASSET_KIND,
        bind_sprite_texture,
    );
}

/// Registers the sprites' source-row columns.
pub fn register_columns(store: &mut SceneGpuStore, device: &std::sync::Arc<wgpu::Device>) {
    // The join places them as the sprite overlay's rows.
    SpriteSourceRow::register_gpu_columns_growable(store, 64, device);
}

/// The sprites' scene rows: the plugin's join from source rows to the
/// sprite overlay's rows, placed by their owners.
pub fn derivation(
    device: &wgpu::Device,
    keys: helio_scene_join::RowJoinKeys,
) -> Box<dyn helio_scene_join::SceneDerivation> {
    Box::new(helio_pass_sprite_batch::scene_rows::scene_rows(
        device,
        keys,
        helio_scene_join::BufferKey::of(SPRITE_SOURCES_BUFFER),
    ))
}

/// The sprites' feature: cull, batch and the 2D overlay.
pub fn render_features() -> helio_feature_api::FeatureSet {
    helio_feature_api::FeatureSet::new().with(helio_pass_sprite_batch::SpriteFeature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_source_row_is_the_pass_row() {
        assert_eq!(std::mem::size_of::<SpriteSourceRow>(), 20 * 4);
        assert_eq!(
            std::mem::size_of::<SpriteSourceRow>() as u64,
            helio_pass_sprite_batch::scene_rows::SPRITE_SOURCE_ROW_BYTES
        );
        assert_eq!(
            std::mem::size_of::<helio_pass_sprite_batch::SpriteRow>(),
            20 * 4
        );
    }
}
