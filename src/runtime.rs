//! What the sprites plugin registers for a game (and the editor): every
//! runtime contribution, grouped by the engine seam it plugs into.

use engine_backend::scene::helio_bridge::SceneRowsRegistration;
use engine_backend::subsystems::render::RenderFeatureRegistration;
use helio_component::texture_assets::TextureBindingRegistration;
use sprites_component::SpriteComponent;

// ── Component classes ───────────────────────────────────────────────────────

pulsar_world_registry::submit_world_component!(SpriteComponent);

// ── Rendering ───────────────────────────────────────────────────────────────

// Sprite culling, batching and the 2D overlay.
inventory::submit! {
    RenderFeatureRegistration {
        origin: pulsar_plugins::origin!(),
        features: sprites_component::render_features,
        configure: |config| config,
    }
}

// The sprites' source rows and the join that places them by their owners as
// the overlay's rows.
inventory::submit! {
    SceneRowsRegistration {
        origin: pulsar_plugins::origin!(),
        name: "sprites.rows",
        register_columns: sprites_component::register_columns,
        derivation: sprites_component::derivation,
    }
}

// ── Assets ──────────────────────────────────────────────────────────────────

// A sprite takes in its loaded image.
inventory::submit! {
    TextureBindingRegistration {
        origin: pulsar_plugins::origin!(),
        name: "sprites.textures",
        register: sprites_component::register_texture_binding,
    }
}

#[cfg(test)]
mod tests {
    use engine_backend::subsystems::render::helio_renderer::render_features;

    #[test]
    fn registers_its_feature() {
        let features = render_features::plugin_features();
        assert!(
            features.ids().contains(&"helio.sprites2d"),
            "features: {:?}",
            features.ids()
        );
    }

    #[test]
    fn registers_under_its_class_name() {
        assert_eq!(
            pulsar_world_registry::component_id_for_class(sprites_component::SPRITE_CLASS_NAME),
            Some(pulsar_scenedb::component_id::<
                sprites_component::SpriteComponent,
            >())
        );
    }

    /// Every registration of the plugin names this root, so it resolves to
    /// the plugin's id.
    #[test]
    fn every_registration_resolves_to_the_plugin() {
        let ours: Vec<_> = pulsar_plugins::contributions()
            .into_iter()
            .filter(|summary| summary.origin == pulsar_plugins::origin!())
            .collect();
        assert!(
            ours.iter()
                .any(|summary| summary.category == "component.class")
        );
        for summary in &ours {
            assert_eq!(
                pulsar_plugins::plugin_of(summary.origin).map(|plugin| plugin.id.as_str()),
                Some("pulsar.sprites"),
                "{summary:?}"
            );
        }
    }
}
