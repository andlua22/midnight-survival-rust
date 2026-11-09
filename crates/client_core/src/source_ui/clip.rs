//! Binary alpha-mask coverage for the egui preview, evaluated at physical pixel
//! centers. This deliberately does not approximate a texture mask by its bounds.

use std::collections::BTreeMap;

use bevy_egui::egui::{self, Color32, Mesh, Pos2, Rect, epaint::Vertex, pos2};

use super::TexturePixels;

pub(super) struct MaskCoverage {
    size: [u32; 2],
    pixels: Vec<bool>,
}

fn cross(a: egui::Vec2, b: egui::Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

impl MaskCoverage {
    pub(super) fn from_mesh(
        mesh: &Mesh,
        texture: &TexturePixels,
        size: [u32; 2],
        alpha: f32,
    ) -> Self {
        let mut result = Self {
            size,
            pixels: vec![false; size[0] as usize * size[1] as usize],
        };
        for triangle in mesh.triangles() {
            let [a, b, c] = triangle.map(|index| mesh.vertices[index as usize]);
            let area = cross(b.pos - a.pos, c.pos - a.pos);
            if area.abs() < 1e-8 {
                continue;
            }
            let bounds = Rect::from_points(&[a.pos, b.pos, c.pos]);
            let [x0, y0, x1, y1] = pixel_bounds(bounds, size);
            for y in y0..y1 {
                for x in x0..x1 {
                    let point = pos2(x as f32 + 0.5, y as f32 + 0.5);
                    let weights = [
                        cross(b.pos - point, c.pos - point) / area,
                        cross(c.pos - point, a.pos - point) / area,
                        cross(a.pos - point, b.pos - point) / area,
                    ];
                    if weights.iter().any(|weight| *weight < -1e-6) {
                        continue;
                    }
                    let uv = a.uv.to_vec2() * weights[0]
                        + b.uv.to_vec2() * weights[1]
                        + c.uv.to_vec2() * weights[2];
                    if texture.alpha([uv.x, uv.y]) * alpha >= 0.001 {
                        result.pixels[y as usize * size[0] as usize + x as usize] = true;
                    }
                }
            }
        }
        result
    }

    fn contains(&self, x: u32, y: u32) -> bool {
        x < self.size[0]
            && y < self.size[1]
            && self.pixels[y as usize * self.size[0] as usize + x as usize]
    }
}

fn pixel_bounds(rect: Rect, size: [u32; 2]) -> [u32; 4] {
    [
        rect.min.x.floor().max(0.0).min(size[0] as f32) as u32,
        rect.min.y.floor().max(0.0).min(size[1] as f32) as u32,
        rect.max.x.ceil().max(0.0).min(size[0] as f32) as u32,
        rect.max.y.ceil().max(0.0).min(size[1] as f32) as u32,
    ]
}

/// Coalesce identical adjacent scanline runs to keep a rectangular mask cheap.
pub(super) fn clip_masks(mesh: Mesh, masks: &[&MaskCoverage], size: [u32; 2]) -> Mesh {
    if masks.is_empty() || mesh.indices.is_empty() {
        return mesh;
    }
    let [x0, y0, x1, y1] = pixel_bounds(mesh.calc_bounds(), size);
    let mut open: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    let mut rectangles = Vec::new();
    for y in y0..y1 {
        let mut next = BTreeMap::new();
        let mut x = x0;
        while x < x1 {
            while x < x1 && !masks.iter().all(|mask| mask.contains(x, y)) {
                x += 1;
            }
            let start = x;
            while x < x1 && masks.iter().all(|mask| mask.contains(x, y)) {
                x += 1;
            }
            if start < x {
                next.insert((start, x), open.remove(&(start, x)).unwrap_or(y));
            }
        }
        for ((left, right), top) in open {
            rectangles.push(Rect::from_min_max(
                pos2(left as f32, top as f32),
                pos2(right as f32, y as f32),
            ));
        }
        open = next;
    }
    for ((left, right), top) in open {
        rectangles.push(Rect::from_min_max(
            pos2(left as f32, top as f32),
            pos2(right as f32, y1 as f32),
        ));
    }
    let mut result = Mesh::with_texture(mesh.texture_id);
    for rectangle in rectangles {
        result.append(clip_polygon(
            &mesh,
            &[
                rectangle.left_top(),
                rectangle.right_top(),
                rectangle.right_bottom(),
                rectangle.left_bottom(),
            ],
        ));
    }
    result
}

fn interpolate(a: Vertex, b: Vertex, t: f32) -> Vertex {
    let a_color = a.color.to_array();
    let b_color = b.color.to_array();
    let color: [u8; 4] = std::array::from_fn(|index| {
        (f32::from(a_color[index]) + (f32::from(b_color[index]) - f32::from(a_color[index])) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    });
    Vertex {
        pos: a.pos + (b.pos - a.pos) * t,
        uv: a.uv + (b.uv - a.uv) * t,
        color: Color32::from_rgba_premultiplied(color[0], color[1], color[2], color[3]),
    }
}

/// Clip triangles and their UVs against a convex, either-winding polygon.
pub(super) fn clip_polygon(mesh: &Mesh, polygon: &[Pos2]) -> Mesh {
    let mut result = Mesh::with_texture(mesh.texture_id);
    if polygon.len() < 3 {
        return result;
    }
    let area: f32 = polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
        .map(|(a, b)| a.x * b.y - a.y * b.x)
        .sum();
    if area.abs() < 1e-8 {
        return result;
    }
    let orientation = area.signum();
    for triangle in mesh.triangles() {
        let mut vertices: Vec<_> = triangle.map(|index| mesh.vertices[index as usize]).into();
        for (edge_a, edge_b) in polygon
            .iter()
            .zip(polygon.iter().cycle().skip(1))
            .take(polygon.len())
        {
            if vertices.is_empty() {
                break;
            }
            let mut next = Vec::new();
            let signed =
                |vertex: Vertex| cross(*edge_b - *edge_a, vertex.pos - *edge_a) * orientation;
            for (a, b) in vertices
                .iter()
                .zip(vertices.iter().cycle().skip(1))
                .take(vertices.len())
            {
                let da = signed(*a);
                let db = signed(*b);
                if da >= 0.0 {
                    next.push(*a);
                }
                if (da >= 0.0) != (db >= 0.0) {
                    next.push(interpolate(*a, *b, da / (da - db)));
                }
            }
            vertices = next;
        }
        if vertices.len() < 3 {
            continue;
        }
        let start = result.vertices.len() as u32;
        for index in 1..vertices.len() - 1 {
            result.indices.extend_from_slice(&[
                start,
                start + index as u32,
                start + index as u32 + 1,
            ]);
        }
        result.vertices.extend(vertices);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use midnight_map_data::TextureSampler;

    fn quad(rect: Rect) -> Mesh {
        let mut mesh = Mesh::default();
        mesh.add_rect_with_uv(
            rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        mesh
    }

    #[test]
    fn stencil_alpha_holes_and_nested_intersection_survive_clipping() {
        let texture = TexturePixels {
            size: [2, 2],
            rgba: vec![
                255, 255, 255, 255, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 255,
            ],
            sampler: TextureSampler {
                filter_mode: 0,
                wrap: [1; 3],
                aniso: 1,
                mip_bias: 0.0,
            },
        };
        let mesh = quad(Rect::from_min_max(pos2(0.0, 0.0), pos2(4.0, 4.0)));
        let mask = MaskCoverage::from_mesh(&mesh, &texture, [4, 4], 1.0);
        assert!(mask.contains(0, 0) && mask.contains(3, 3));
        assert!(!mask.contains(3, 0) && !mask.contains(0, 3));
        let clipped = clip_masks(mesh.clone(), &[&mask], [4, 4]);
        assert!(!clipped.indices.is_empty());
        let second = MaskCoverage {
            size: [4, 4],
            pixels: (0..16).map(|index| index % 4 < 2).collect(),
        };
        let nested = clip_masks(mesh, &[&mask, &second], [4, 4]);
        assert_eq!(
            nested.calc_bounds(),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(2.0, 2.0))
        );
        assert!(
            clip_masks(
                quad(Rect::from_min_max(pos2(0.0, 0.0), pos2(4.0, 4.0))),
                &[&MaskCoverage::from_mesh(&clipped, &texture, [4, 4], 0.0)],
                [4, 4]
            )
            .indices
            .is_empty()
        );
    }

    #[test]
    fn rotated_clip_keeps_affine_uvs_for_both_windings() {
        let mesh = quad(Rect::from_min_max(pos2(0.0, 0.0), pos2(10.0, 10.0)));
        let mut diamond = [
            pos2(5.0, 0.0),
            pos2(10.0, 5.0),
            pos2(5.0, 10.0),
            pos2(0.0, 5.0),
        ];
        for _ in 0..2 {
            let result = clip_polygon(&mesh, &diamond);
            assert!(!result.indices.is_empty());
            for vertex in &result.vertices {
                assert!(((vertex.pos.x - 5.0).abs() + (vertex.pos.y - 5.0).abs()) <= 5.001);
                assert!((vertex.uv.x - vertex.pos.x / 10.0).abs() < 1e-5);
                assert!((vertex.uv.y - vertex.pos.y / 10.0).abs() < 1e-5);
            }
            diamond.reverse();
        }
    }
}
