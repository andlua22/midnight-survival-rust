//! Canonical Bevy coordinates use +X to the right and -Z at the map's top.
//! Unity scene +Z becomes Bevy -Z exactly once in the APK importer.
//!
//! Extents are supplied by the caller: the current overview calibration is not
//! a claim about the APK's original map scale. Projection is deliberately
//! unclamped so clipping a radar crop does not change its scale or center.

/// Project world XZ into overview UV, with (0, 0) at the image's top left.
/// `half_extent` must be positive and is measured in world units.
pub fn world_to_overview_uv(world_xz: [f32; 2], half_extent: f32) -> [f32; 2] {
    let span = half_extent * 2.0;
    [
        (world_xz[0] + half_extent) / span,
        (world_xz[1] + half_extent) / span,
    ]
}

/// Invert overview UV into world XZ, including coordinates outside the image.
/// `half_extent` must match the forward projection and be positive.
pub fn overview_uv_to_world(uv: [f32; 2], half_extent: f32) -> [f32; 2] {
    let span = half_extent * 2.0;
    [uv[0] * span - half_extent, uv[1] * span - half_extent]
}

/// Project a world point relative to the radar center into a screen offset.
/// `world_range` is the positive center-to-edge distance in world units;
/// `screen_radius` is the corresponding screen distance.
pub fn world_to_radar_offset(
    world_xz: [f32; 2],
    center_xz: [f32; 2],
    world_range: f32,
    screen_radius: f32,
) -> [f32; 2] {
    let scale = screen_radius / world_range;
    [
        (world_xz[0] - center_xz[0]) * scale,
        (world_xz[1] - center_xz[1]) * scale,
    ]
}

/// Clockwise bearing from map north (-Z), in radians in [0, TAU).
pub fn world_bearing(world_delta_xz: [f32; 2]) -> f32 {
    world_delta_xz[0]
        .atan2(-world_delta_xz[1])
        .rem_euclid(std::f32::consts::TAU)
}

/// Map bearing of the simulation's camera yaw (yaw zero faces world -Z).
pub fn yaw_to_map_bearing(yaw: f32) -> f32 {
    (-yaw).rem_euclid(std::f32::consts::TAU)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI, TAU};

    fn assert_pair(actual: [f32; 2], expected: [f32; 2], tolerance: f32) {
        for axis in 0..2 {
            assert!(
                (actual[axis] - expected[axis]).abs() <= tolerance,
                "{actual:?} != {expected:?} on axis {axis}"
            );
        }
    }

    #[test]
    fn overview_places_north_at_top_and_east_at_right() {
        assert_pair(world_to_overview_uv([0.0, 0.0], 1000.0), [0.5, 0.5], 1e-6);
        assert_pair(
            world_to_overview_uv([0.0, -1000.0], 1000.0),
            [0.5, 0.0],
            1e-6,
        );
        assert_pair(
            world_to_overview_uv([1000.0, 0.0], 1000.0),
            [1.0, 0.5],
            1e-6,
        );
        assert_pair(
            world_to_overview_uv([-1000.0, 1000.0], 1000.0),
            [0.0, 1.0],
            1e-6,
        );
        assert_pair(
            world_to_overview_uv([100.0, -200.0], 1000.0),
            [0.55, 0.4],
            1e-6,
        );
    }

    #[test]
    fn overview_round_trips_off_center_and_out_of_bounds_coordinates() {
        for half_extent in [1000.0, 5360.0] {
            for point in [
                [0.0, 0.0],
                [2100.0, -700.0],
                [-half_extent, half_extent],
                [8000.0, -6500.0],
            ] {
                let uv = world_to_overview_uv(point, half_extent);
                assert_pair(overview_uv_to_world(uv, half_extent), point, 0.002);
            }
        }
        assert_pair(
            overview_uv_to_world([0.55, 0.6], 1000.0),
            [100.0, 200.0],
            0.001,
        );
    }

    #[test]
    fn radar_places_north_and_east_relative_to_an_off_center_player() {
        let player = [2100.0, -700.0];
        assert_pair(
            world_to_radar_offset(player, player, 150.0, 75.0),
            [0.0, 0.0],
            1e-6,
        );
        assert_pair(
            world_to_radar_offset([2100.0, -850.0], player, 150.0, 75.0),
            [0.0, -75.0],
            1e-6,
        );
        assert_pair(
            world_to_radar_offset([2250.0, -700.0], player, 150.0, 75.0),
            [75.0, 0.0],
            1e-6,
        );
        assert_pair(
            world_to_radar_offset([2100.0, -550.0], player, 150.0, 75.0),
            [0.0, 75.0],
            1e-6,
        );
    }

    #[test]
    fn radar_marker_matches_its_location_in_the_overview_crop() {
        let half_extent = 5360.0;
        let world_range = 150.0;
        let radius = 75.0;
        let player = [2250.0, 1200.0];
        let landmark = [2325.0, 1275.0];
        let center_uv = world_to_overview_uv(player, half_extent);
        let landmark_uv = world_to_overview_uv(landmark, half_extent);
        let crop_half_span = world_range / (half_extent * 2.0);
        let crop_uv = [0, 1].map(|axis| {
            (landmark_uv[axis] - (center_uv[axis] - crop_half_span)) / (crop_half_span * 2.0)
        });
        assert_pair(crop_uv, [0.75, 0.75], 1e-5);
        let background_offset = crop_uv.map(|uv| (uv - 0.5) * radius * 2.0);
        let marker_offset = world_to_radar_offset(landmark, player, world_range, radius);
        assert_pair(marker_offset, [37.5, 37.5], 1e-6);
        assert_pair(background_offset, marker_offset, 0.002);
    }

    #[test]
    fn world_bearings_use_the_same_cardinal_axes_as_the_map() {
        for (delta, expected) in [
            ([0.0, -1.0], 0.0),
            ([1.0, 0.0], FRAC_PI_2),
            ([0.0, 1.0], PI),
            ([-1.0, 0.0], PI + FRAC_PI_2),
        ] {
            assert!((world_bearing(delta) - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn camera_heading_agrees_with_projected_simulation_forward_motion() {
        for (yaw, expected) in [
            (0.0, 0.0),
            (FRAC_PI_2, PI + FRAC_PI_2),
            (PI, PI),
            (PI + FRAC_PI_2, FRAC_PI_2),
        ] {
            let bearing = yaw_to_map_bearing(yaw);
            let angle_error = (bearing - expected + PI).rem_euclid(TAU) - PI;
            assert!(angle_error.abs() < 1e-6);
            let (sin, cos) = yaw.sin_cos();
            let projected_motion = world_to_radar_offset([-sin, -cos], [0.0, 0.0], 1.0, 1.0);
            assert_pair([bearing.sin(), -bearing.cos()], projected_motion, 1e-6);
        }
    }
}
