//! The real `.qgcp` supplied by the project owner (its `MANIFEST.json` and
//! `HEADER.json`) loads, validates, resolves and round-trips unchanged.

use stencil_core::bsp::{Rect, SplitAxis};
use stencil_core::manifest::{BlendMode, FactionCrest, LifecycleState, RendererSource};
use stencil_core::{BspNode, QgcpLayoutManifest};

const MANIFEST: &str = include_str!("fixtures/layout-blueprint.manifest.json");
const HEADER: &str = include_str!("fixtures/layout-blueprint.header.json");

fn load() -> QgcpLayoutManifest {
    serde_json::from_str(MANIFEST).expect("the real manifest parses")
}

#[test]
fn the_real_manifest_loads_with_the_expected_content() {
    let m = load();
    assert_eq!(m.genesis_id, "genesis_phantori_system_01");
    assert_eq!(m.lifecycle, LifecycleState::Active);
    assert_eq!((m.capacity.current_octave, m.capacity.child_count), (4, 4));
    assert_eq!(m.instrument_library.len(), 2);
    let canvas = &m.instrument_library["meso_voxel_canvas_template"];
    assert_eq!(canvas.layers.len(), 2);
    assert_eq!(canvas.layers[0].blend_mode, BlendMode::Screen);
    assert!(matches!(
        &canvas.layers[0].renderer,
        RendererSource::Particles {
            particle_count: 64,
            ..
        }
    ));
    assert!(matches!(
        canvas.layers[1].renderer,
        RendererSource::InfiniteCanvas
    ));
    assert_eq!(canvas.layers[1].faction_crest, FactionCrest::Sylvanid);
    assert_eq!(m.meso_voxels[0].bound_nodes[0].name, "Damage Vector Node");
}

#[test]
fn the_real_manifest_satisfies_every_law() {
    let m = load();
    assert_eq!(m.validate(), Vec::<String>::new());
}

#[test]
fn it_round_trips_to_identical_json() {
    let m = load();
    // Through a string: `to_value` would widen f32 (1.2 → 1.2000000476837158).
    let again: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    let original: serde_json::Value = serde_json::from_str(MANIFEST).unwrap();
    assert_eq!(again, original);
}

#[test]
fn the_layout_resolves_and_leaves_point_at_real_templates() {
    let m = load();
    let panels = m.layout_tree.resolved(Rect::new(0.0, 0.0, 1000.0, 600.0));
    assert_eq!(panels.len(), 2);
    assert!(
        (panels[0].rect.width - 220.0).abs() < 1e-3,
        "{}",
        panels[0].rect.width
    );
    assert!((panels[1].rect.x - 220.0).abs() < 1e-3);
    for p in &panels {
        let t = p.active_template_id.as_deref().expect("template");
        assert!(m.instrument_library.contains_key(t), "{t}");
    }
    assert!(matches!(
        m.layout_tree,
        BspNode::Branch {
            axis: SplitAxis::Vertical,
            ..
        }
    ));
}

#[test]
fn a_swapped_template_changes_validation_only_when_it_is_missing() {
    let mut m = load();
    assert!(m
        .layout_tree
        .set_template("primary_viewport", Some("sidebar_navigation_template")));
    assert!(m.validate().is_empty());
    m.layout_tree.set_template("primary_viewport", Some("nope"));
    assert_eq!(
        m.validate(),
        vec!["leaf 'primary_viewport' points at missing template 'nope'".to_string()]
    );
}

#[test]
fn a_wrong_border_weight_is_caught() {
    let mut m = load();
    m.instrument_library
        .get_mut("sidebar_navigation_template")
        .unwrap()
        .layers[0]
        .border_weight = 1.5;
    let p = m.validate();
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(p[0].contains("Lore Violation"));
}

#[test]
fn the_package_header_matches_the_manifest() {
    let h: serde_json::Value = serde_json::from_str(HEADER).unwrap();
    assert_eq!(h["capsuleCount"], 4);
    assert_eq!(h["lifecycle"], "active");
    let m = load();
    assert_eq!(
        h["capsuleCount"].as_u64().unwrap() as u32,
        m.capacity.child_count
    );
}
