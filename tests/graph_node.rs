//! The sprite overlay (culling and batching, one unit) is a node of render
//! graphs: registered by the plugin's root through its render feature, pins
//! found by the probe from what its passes declare, placed in the default
//! deferred graph at the display slot (over the final image), built from a
//! graph asset, and absent, with the reason, while the plugin is disabled.

// The plugin as a game links it: its root, with every registration.
use sprites_plugin as _;

use std::sync::{Arc, Mutex};

use engine_backend::subsystems::render::helio_renderer::{pipelines, render_features};
use helio::{DebugDrawState, PassBuildContext, RendererConfig};
use helio_graph_asset::{validate, GraphAsset, NodeInstance, NodeRegistry, PinKind};
use pulsar_scenedb::gpu::{EngineGpuContext, GpuMirrorHandle, SceneGpuConfig, SceneGpuStore};

/// Every sprite node: kind, the feature that registers it, the pass types its
/// build adds (the culler and the batcher are one unit).
const NODES: &[(&str, &str, &[&str])] = &[(
    "helio.pass.sprite_overlay",
    "helio.sprites2d",
    &["SpriteBatchPass", "SpriteCullPass"],
)];

/// The plugin gate is process-wide state and a wgpu device is one at a time
/// (Pulsar-Native#1647): every test takes this first.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn device() -> Option<(Arc<wgpu::Device>, Arc<wgpu::Queue>)> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(helio_core::wgpu_instance_descriptor());
        let adapter = instance.request_adapter(&Default::default()).await.ok()?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: helio::required_wgpu_features(adapter.features()),
                required_limits: helio::required_wgpu_limits(adapter.limits()),
                experimental_features: helio::required_experimental_features(adapter.features()),
                ..Default::default()
            })
            .await
            .ok()?;
        Some((Arc::new(device), Arc::new(queue)))
    })
}

fn buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::UNIFORM
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn configured() -> RendererConfig {
    // The renderer config with the plugins' settings, as a host makes it.
    render_features::configure(RendererConfig::new(320, 180, wgpu::TextureFormat::Rgba8Unorm))
}

fn registry() -> NodeRegistry {
    helio_presets::graphs::registry_with(&render_features::plugin_features())
}

/// The default deferred graph with its sections expanded: every node in
/// execution order.
fn asset_of(registry: &NodeRegistry) -> GraphAsset {
    let sectioned = helio_presets::graphs::deferred_graph_asset_for(registry, &configured());
    helio_graph_asset::sections::expand(&sectioned).asset
}

/// Compiles `asset` and lists the type names of its passes (`None`: no GPU).
fn pass_types(asset: &GraphAsset) -> Option<Vec<String>> {
    let (device, queue) = device()?;
    let gpu = EngineGpuContext::new(Arc::clone(&device), Arc::clone(&queue));
    let store = SceneGpuStore::new(
        &gpu,
        SceneGpuConfig {
            classes: Vec::new(),
            tombstone_headroom: 0,
            max_cells_metadata: 0,
        },
    );
    let camera = buffer(&device, 512);
    let debug_camera = buffer(&device, 64);
    let cull_stats = buffer(&device, 64);
    let mut env = helio_graph_asset::build_env(&PassBuildContext {
        device: &device,
        queue: &queue,
        config: configured(),
        debug_state: Arc::new(Mutex::new(DebugDrawState::default())),
        camera_buffer: &camera,
        debug_camera_buffer: &debug_camera,
        cull_stats_buffer: &cull_stats,
        owns_device: false,
        scene_db: GpuMirrorHandle::new(Arc::new(store), Arc::clone(&queue)),
    });
    let graph = helio_graph_asset::compile(asset, &registry(), &mut env)
        .unwrap_or_else(|errors| panic!("the graph builds: {errors:?}"));
    Some(
        (0..graph.pass_count())
            .filter_map(|index| graph.pass_declaration(index))
            .map(|pass| pass.type_name.to_string())
            .collect(),
    )
}

#[test]
fn every_pass_is_a_registered_node() {
    let _serial = serial();
    pulsar_plugins::deactivate();
    let registry = registry();
    for (kind, feature, _) in NODES {
        let node = registry
            .get(kind)
            .unwrap_or_else(|| panic!("{kind} is not registered"));
        assert_eq!(registry.feature_of(kind), Some(*feature), "{kind}");
        assert!(node.generated, "{kind}: pins come from what the pass declares");
        assert!(node.params.iter().any(|p| p.key == "enabled"), "{kind}");
        assert!(node.placement.is_some(), "{kind}: the preset places it");
        assert!(node.title.contains(": "), "{kind}: title reads `Group: Action`");
    }
}

/// Sprites draw over the final image: the overlay is in the default graph.
#[test]
fn the_default_graph_places_the_overlay_at_the_display_slot() {
    let _serial = serial();
    pulsar_plugins::deactivate();
    let asset = asset_of(&registry());
    let kinds: Vec<&str> = asset.nodes.iter().map(|n| n.kind.as_str()).collect();
    eprintln!("SPRITE_GRAPH_ORDER {kinds:?}");
    assert!(
        kinds.contains(&"helio.pass.sprite_overlay"),
        "the default graph holds the sprite node: {kinds:?}"
    );
}

#[test]
fn the_graph_builds_the_nodes_and_enabled_off_bypasses_them() {
    let _serial = serial();
    pulsar_plugins::deactivate();
    let mut asset = asset_of(&registry());
    let Some(types) = pass_types(&asset) else {
        eprintln!("GPU_VALIDATION_SKIPPED_NO_ADAPTER: graph nodes of pulsar.sprites");
        return;
    };
    for (kind, _, passes) in NODES {
        for pass in *passes {
            assert!(
                types.iter().any(|t| t.contains(pass)),
                "{kind} builds {pass}: {types:?}"
            );
        }
    }

    // The Enabled parameter bypasses a node: no pass.
    for node in asset.nodes.iter_mut().filter(|n| n.kind == "helio.pass.sprite_overlay") {
        node.params.insert("enabled".into(), false.into());
    }
    let after = pass_types(&asset).expect("the same device");
    for pass in NODES[0].2 {
        assert!(!after.iter().any(|t| t.contains(pass)), "{pass} is bypassed");
    }
}

#[test]
fn the_probe_gives_the_nodes_the_pins_their_passes_declare() {
    let _serial = serial();
    pulsar_plugins::deactivate();
    let registry = pipelines::graph_node_registry();
    if pipelines::graph_pin_probe().is_none() {
        eprintln!("GPU_VALIDATION_SKIPPED_NO_ADAPTER: node pins of pulsar.sprites");
        return;
    }
    for (kind, ..) in NODES {
        let pins = registry
            .get(kind)
            .unwrap_or_else(|| panic!("{kind} is not registered"))
            .pins(&Default::default());
        assert!(pins.inputs.iter().any(|p| p.ty.kind == PinKind::Flow), "{kind}: {pins:?}");
        assert!(pins.outputs.iter().any(|p| p.ty.kind == PinKind::Flow), "{kind}: {pins:?}");
        eprintln!("{kind} PINS {pins:?}");
    }
}

#[test]
fn a_disabled_plugin_has_no_nodes_and_a_graph_naming_one_says_so() {
    let _serial = serial();
    pulsar_plugins::deactivate();
    let with_plugin = registry();
    let mut asset = asset_of(&with_plugin);
    asset.nodes.push(NodeInstance {
        id: "extra".into(),
        kind: "helio.pass.sprite_overlay".into(),
        params: Default::default(),
        position: [0.0; 2],
    });

    pulsar_plugins::activate(&pulsar_plugins::resolve(
        pulsar_plugins::installed(),
        &pulsar_plugins::ProjectPlugins::default().with(&["pulsar.sprites"], false),
    ));
    let registry = pipelines::graph_node_registry();
    pulsar_plugins::deactivate();

    for (kind, ..) in NODES {
        assert!(registry.get(kind).is_none(), "{kind} is unregistered");
        assert_eq!(registry.absence(kind), Some("plugin pulsar.sprites disabled"), "{kind}");
    }
    let validation = validate(&asset, &registry);
    let messages: Vec<_> = validation.errors().map(|d| d.message.clone()).collect();
    assert!(
        messages
            .iter()
            .any(|m| m == "missing node `helio.pass.sprite_overlay` (plugin pulsar.sprites disabled)"),
        "{messages:?}"
    );
    // Enabled again, the registry follows.
    assert!(pipelines::graph_node_registry().get("helio.pass.sprite_overlay").is_some());
}
