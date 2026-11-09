//! Unity RectTransform layout in an explicit caller-provided parent canvas.
//! A supplied viewport is preview/runtime input, never evidence of an APK reference resolution.

use std::collections::BTreeSet;

use glam::{Mat4, Quat, Vec3};
use midnight_map_data::SourceObject;
use serde::Serialize;

use crate::{HierarchyNode, UiError, UiPanel, invalid};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct UiRect {
    pub min: [f32; 2],
    pub size: [f32; 2],
}

impl UiRect {
    pub fn max(self) -> [f32; 2] {
        [self.min[0] + self.size[0], self.min[1] + self.size[1]]
    }
    /// Unity local coordinates: bottom-left, bottom-right, top-right, top-left.
    pub fn corners(self) -> [[f32; 2]; 4] {
        let max = self.max();
        [self.min, [max[0], self.min[1]], max, [self.min[0], max[1]]]
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct CanvasViewport {
    pub pixels: [u32; 2],
    pub scale_factor: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResolvedNode {
    pub source: SourceObject,
    pub path: String,
    pub parent: Option<usize>,
    pub active: bool,
    pub local_rect: Option<UiRect>,
    /// Column-major matrix. Converts source local XY/Y-up to physical screen pixels/Y-down.
    pub screen_from_local: [f32; 16],
    pub screen_corners: Option<[[f32; 2]; 4]>,
}

impl ResolvedNode {
    pub fn to_screen(&self, point: [f32; 2]) -> [f32; 2] {
        Mat4::from_cols_array(&self.screen_from_local)
            .transform_point3(Vec3::new(point[0], point[1], 0.0))
            .truncate()
            .to_array()
    }
}

pub fn resolve_panel(
    panel: &UiPanel,
    viewport: CanvasViewport,
) -> Result<Vec<ResolvedNode>, UiError> {
    if viewport.pixels.contains(&0)
        || !viewport.scale_factor.is_finite()
        || viewport.scale_factor <= 0.0
    {
        return Err(invalid(
            "UI viewport dimensions/scale must be positive and finite",
        ));
    }
    let width = viewport.pixels[0] as f32;
    let height = viewport.pixels[1] as f32;
    let logical = [
        width / viewport.scale_factor,
        height / viewport.scale_factor,
    ];
    let parent_rect = UiRect {
        min: [-logical[0] / 2.0, -logical[1] / 2.0],
        size: logical,
    };
    let screen = Mat4::from_translation(Vec3::new(width / 2.0, height / 2.0, 0.0))
        * Mat4::from_scale(Vec3::new(
            viewport.scale_factor,
            -viewport.scale_factor,
            1.0,
        ));
    let mut result = Vec::new();
    resolve_node(
        &panel.root,
        None,
        true,
        Some(parent_rect),
        screen,
        "",
        &mut BTreeSet::new(),
        &mut result,
    )?;
    if result.len() != panel.stats.nodes {
        return Err(invalid("resolved UI node count differs from source panel"));
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn resolve_node(
    node: &HierarchyNode,
    parent: Option<usize>,
    parent_active: bool,
    parent_rect: Option<UiRect>,
    parent_matrix: Mat4,
    prefix: &str,
    visited: &mut BTreeSet<SourceObject>,
    result: &mut Vec<ResolvedNode>,
) -> Result<(), UiError> {
    if !visited.insert(node.source.clone()) || prefix.split('/').count() > 128 {
        return Err(invalid("duplicate/cyclic/deep UI hierarchy"));
    }
    let transform = &node.transform;
    if transform
        .local_position
        .iter()
        .chain(&transform.local_rotation)
        .chain(&transform.local_scale)
        .any(|value| !value.is_finite())
    {
        return Err(invalid("non-finite source UI transform"));
    }
    let quaternion = Quat::from_array(transform.local_rotation);
    if quaternion.length_squared() < 1e-8 {
        return Err(invalid("source UI transform has a zero quaternion"));
    }
    let mut position = transform.local_position;
    let rect = if let Some(rect) = &transform.rect {
        if rect
            .anchor_min
            .iter()
            .chain(&rect.anchor_max)
            .chain(&rect.anchored_position)
            .chain(&rect.size_delta)
            .chain(&rect.pivot)
            .any(|value| !value.is_finite())
        {
            return Err(invalid("non-finite RectTransform field"));
        }
        let parent = parent_rect.unwrap_or(UiRect {
            min: [0.0; 2],
            size: [0.0; 2],
        });
        let mut size = [0.0; 2];
        let mut min = [0.0; 2];
        for axis in 0..2 {
            size[axis] = parent.size[axis] * (rect.anchor_max[axis] - rect.anchor_min[axis])
                + rect.size_delta[axis];
            position[axis] = parent.min[axis]
                + parent.size[axis]
                    * (rect.anchor_min[axis]
                        + (rect.anchor_max[axis] - rect.anchor_min[axis]) * rect.pivot[axis])
                + rect.anchored_position[axis];
            min[axis] = -rect.pivot[axis] * size[axis];
        }
        Some(UiRect { min, size })
    } else {
        None
    };
    let matrix = parent_matrix
        * Mat4::from_scale_rotation_translation(
            Vec3::from_array(transform.local_scale),
            quaternion.normalize(),
            Vec3::from_array(position),
        );
    if !matrix.is_finite() {
        return Err(invalid("resolved UI transform overflow"));
    }
    let index = result.len();
    let path = if prefix.is_empty() {
        node.name.clone()
    } else {
        format!("{prefix}/{}", node.name)
    };
    let active = parent_active && node.active_self;
    let screen_corners = rect.map(|rect| {
        rect.corners().map(|point| {
            matrix
                .transform_point3(Vec3::new(point[0], point[1], 0.0))
                .truncate()
                .to_array()
        })
    });
    result.push(ResolvedNode {
        source: node.source.clone(),
        path: path.clone(),
        parent,
        active,
        local_rect: rect,
        screen_from_local: matrix.to_cols_array(),
        screen_corners,
    });
    if transform.ordered_children.len() != node.children.len() {
        return Err(invalid("UI source child references and hierarchy differ"));
    }
    for (reference, child) in transform.ordered_children.iter().zip(&node.children) {
        if reference.target() != Some(&child.transform.source)
            || child.transform.parent.target() != Some(&transform.source)
        {
            return Err(invalid("UI source hierarchy links differ from transforms"));
        }
        resolve_node(
            child,
            Some(index),
            active,
            rect,
            matrix,
            &path,
            visited,
            result,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        PanelStats, RectTransformData, ReferenceState, SourceReference, TransformData,
        UI_SCHEMA_VERSION,
    };

    fn source(id: u32) -> SourceObject {
        SourceObject {
            bundle: "fixture".into(),
            cab: "CAB-layout".into(),
            path_id: id.to_string(),
        }
    }

    fn reference(owner: &SourceObject, target: Option<&SourceObject>) -> SourceReference {
        SourceReference {
            owner: owner.clone(),
            file_id: 0,
            path_id: target.map_or_else(|| "0".into(), |source| source.path_id.clone()),
            resolution: target.map_or(ReferenceState::Null, |target| ReferenceState::Resolved {
                target: target.clone(),
                class_id: 224,
            }),
        }
    }

    fn node(id: u32, rect: Option<RectTransformData>) -> HierarchyNode {
        let transform_source = source(id + 100);
        HierarchyNode {
            source: source(id),
            name: format!("node{id}"),
            active_self: true,
            active_in_prefab_hierarchy: true,
            layer: 5,
            tag: None,
            components: Vec::new(),
            children: Vec::new(),
            transform: TransformData {
                source: transform_source.clone(),
                local_position: [0.0; 3],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0; 3],
                parent: reference(&transform_source, None),
                ordered_children: Vec::new(),
                root_order: Some(0),
                local_euler_angles_hint: None,
                rect,
            },
        }
    }

    fn fixed(size: [f32; 2]) -> RectTransformData {
        RectTransformData {
            anchor_min: [0.5; 2],
            anchor_max: [0.5; 2],
            anchored_position: [0.0; 2],
            size_delta: size,
            pivot: [0.5; 2],
        }
    }

    fn attach(parent: &mut HierarchyNode, mut child: HierarchyNode) {
        child.transform.parent = reference(&child.transform.source, Some(&parent.transform.source));
        parent.transform.ordered_children.push(reference(
            &parent.transform.source,
            Some(&child.transform.source),
        ));
        parent.children.push(child);
    }

    fn panel(root: HierarchyNode) -> UiPanel {
        fn count(node: &HierarchyNode) -> usize {
            1 + node.children.iter().map(count).sum::<usize>()
        }
        UiPanel {
            schema_version: UI_SCHEMA_VERSION,
            source_apk_sha256: "a".repeat(64),
            name: root.name.clone(),
            bundle: "fixture".into(),
            prefab_asset_path: "fixture.prefab".into(),
            prefab_reference: reference(&root.source, Some(&root.source)),
            stats: PanelStats {
                nodes: count(&root),
                ..Default::default()
            },
            root,
        }
    }

    fn corners(node: &ResolvedNode, expected: [[f32; 2]; 4]) {
        for (actual, expected) in node
            .screen_corners
            .expect("rectangle")
            .into_iter()
            .zip(expected)
        {
            for axis in 0..2 {
                assert!(
                    (actual[axis] - expected[axis]).abs() < 0.001,
                    "{actual:?} != {expected:?}"
                );
            }
        }
    }

    #[test]
    fn source_top_right_anchors_override_serialized_local_xy() {
        let mut root = node(
            1,
            Some(RectTransformData {
                anchor_min: [1.0; 2],
                anchor_max: [1.0; 2],
                pivot: [1.0; 2],
                size_delta: [330.0; 2],
                anchored_position: [10.0, -32.4],
            }),
        );
        root.transform.local_position = [777.0, 888.0, 9.0];
        let resolved = resolve_panel(
            &panel(root),
            CanvasViewport {
                pixels: [1280, 720],
                scale_factor: 1.0,
            },
        )
        .unwrap();
        corners(
            &resolved[0],
            [
                [960.0, 362.4],
                [1290.0, 362.4],
                [1290.0, 32.4],
                [960.0, 32.4],
            ],
        );
        assert_eq!(resolved[0].screen_from_local[14], 9.0);
    }

    #[test]
    fn stretch_anchors_use_parent_size_and_asymmetric_pivot() {
        let mut root = node(
            1,
            Some(RectTransformData {
                anchor_min: [0.0; 2],
                anchor_max: [1.0; 2],
                size_delta: [0.0; 2],
                ..fixed([0.0; 2])
            }),
        );
        attach(
            &mut root,
            node(
                2,
                Some(RectTransformData {
                    anchor_min: [0.25, 0.0],
                    anchor_max: [0.75, 1.0],
                    pivot: [0.2, 0.8],
                    size_delta: [12.0, -20.0],
                    anchored_position: [5.0, -3.0],
                }),
            ),
        );
        let resolved = resolve_panel(
            &panel(root),
            CanvasViewport {
                pixels: [200, 100],
                scale_factor: 1.0,
            },
        )
        .unwrap();
        corners(
            &resolved[0],
            [[0.0, 100.0], [200.0, 100.0], [200.0, 0.0], [0.0, 0.0]],
        );
        corners(
            &resolved[1],
            [[52.6, 87.0], [164.6, 87.0], [164.6, 7.0], [52.6, 7.0]],
        );
    }

    #[test]
    fn rotated_nonuniform_parent_transforms_child_translation_and_corners() {
        let mut root = node(1, Some(fixed([100.0; 2])));
        root.transform.local_rotation =
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array();
        root.transform.local_scale = [2.0, 3.0, 1.0];
        attach(
            &mut root,
            node(
                2,
                Some(RectTransformData {
                    anchored_position: [10.0, 0.0],
                    pivot: [0.0; 2],
                    ..fixed([10.0, 20.0])
                }),
            ),
        );
        let resolved = resolve_panel(
            &panel(root),
            CanvasViewport {
                pixels: [100, 100],
                scale_factor: 1.0,
            },
        )
        .unwrap();
        corners(
            &resolved[1],
            [[50.0, 30.0], [50.0, 10.0], [-10.0, 10.0], [-10.0, 30.0]],
        );
    }

    #[test]
    fn canvas_scale_applies_once_to_sizes_and_anchor_offsets() {
        let root = node(
            1,
            Some(RectTransformData {
                anchor_min: [1.0; 2],
                anchor_max: [1.0; 2],
                pivot: [1.0; 2],
                size_delta: [50.0, 30.0],
                anchored_position: [-5.0, -7.0],
            }),
        );
        let resolved = resolve_panel(
            &panel(root),
            CanvasViewport {
                pixels: [800, 600],
                scale_factor: 2.0,
            },
        )
        .unwrap();
        corners(
            &resolved[0],
            [[690.0, 74.0], [790.0, 74.0], [790.0, 14.0], [690.0, 14.0]],
        );
    }

    #[test]
    fn ordinary_parent_has_no_anchor_size_and_propagates_inactivity() {
        let mut root = node(1, Some(fixed([100.0; 2])));
        let mut ordinary = node(2, None);
        ordinary.active_self = false;
        ordinary.transform.local_position = [20.0, -10.0, 0.0];
        attach(
            &mut ordinary,
            node(
                3,
                Some(RectTransformData {
                    anchor_min: [1.0; 2],
                    anchor_max: [1.0; 2],
                    anchored_position: [3.0, 4.0],
                    ..fixed([20.0; 2])
                }),
            ),
        );
        attach(&mut root, ordinary);
        let resolved = resolve_panel(
            &panel(root),
            CanvasViewport {
                pixels: [100, 100],
                scale_factor: 1.0,
            },
        )
        .unwrap();
        assert!(resolved[0].active);
        assert!(!resolved[1].active && !resolved[2].active);
        assert_eq!(resolved[2].parent, Some(1));
        corners(
            &resolved[2],
            [[63.0, 66.0], [83.0, 66.0], [83.0, 46.0], [63.0, 46.0]],
        );
    }

    #[test]
    fn rejects_invalid_viewport_transforms_and_hierarchy_links() {
        let viewport = CanvasViewport {
            pixels: [100, 100],
            scale_factor: 1.0,
        };
        let mut root = node(1, Some(fixed([10.0; 2])));
        assert!(
            resolve_panel(
                &panel(root.clone()),
                CanvasViewport {
                    scale_factor: 0.0,
                    ..viewport
                }
            )
            .is_err()
        );
        root.transform.local_rotation = [0.0; 4];
        assert!(resolve_panel(&panel(root.clone()), viewport).is_err());
        root.transform.local_rotation = [0.0, 0.0, 0.0, 1.0];
        root.transform.rect.as_mut().unwrap().pivot[0] = f32::NAN;
        assert!(resolve_panel(&panel(root.clone()), viewport).is_err());
        root.transform.rect.as_mut().unwrap().pivot[0] = 0.5;
        attach(&mut root, node(2, None));
        root.children[0].transform.parent.resolution = ReferenceState::Null;
        assert!(resolve_panel(&panel(root), viewport).is_err());
    }
}
