//! Exact linear casts for the upright player capsules. Their Minkowski sum is
//! another capsule. Solving its cylinder and hemisphere intersections directly
//! avoids iterative GJK in crowds. Other shapes use the standard dispatcher.
use bevy::prelude::Vec3;
use bevy_rapier3d::rapier::parry::{
    math::{Pose, Real, Vector},
    query::{
        ClosestPoints, Contact, DefaultQueryDispatcher, NonlinearRigidMotion, QueryDispatcher,
        ShapeCastHit, ShapeCastOptions, ShapeCastStatus, ShapeDistance, ShapeIntersection,
        Unsupported,
    },
    shape::{Capsule, Cuboid, Shape},
};

pub(crate) struct CharacterQueryDispatcher;

/// Roots of a*t² + 2*b*t + c. f64 intermediates and the stable quadratic
/// formula retain the small entry distance when capsules are nearly touching.
fn roots(a: f64, b: f64, c: f64) -> Option<[f64; 2]> {
    if a == 0.0 {
        return None;
    }
    let discriminant = b.mul_add(b, -a * c);
    let tolerance = 4.0 * f64::EPSILON * (b * b + (a * c).abs());
    if discriminant < -tolerance {
        return None;
    }
    let q = -b - discriminant.max(0.0).sqrt().copysign(b);
    if q == 0.0 {
        Some([0.0; 2])
    } else {
        Some([q / a, c / q])
    }
}

fn ray_capsule(
    origin: &Vec3,
    velocity: &Vec3,
    height: f32,
    radius: f32,
    max_time: f32,
) -> Option<f32> {
    let [x, y, z] = [origin.x as f64, origin.y as f64, origin.z as f64];
    let [vx, vy, vz] = [velocity.x as f64, velocity.y as f64, velocity.z as f64];
    let h = height as f64;
    let r2 = (radius as f64).powi(2);
    let dy = y - y.clamp(-h, h);
    if x * x + z * z + dy * dy <= r2 {
        return Some(0.0);
    }

    let mut earliest = f64::INFINITY;
    let mut consider = |time: f64, on_surface: bool| {
        if on_surface && time >= 0.0 && time <= max_time as f64 {
            earliest = earliest.min(time);
        }
    };
    if let Some(times) = roots(vx * vx + vz * vz, x * vx + z * vz, x * x + z * z - r2) {
        for time in times {
            consider(time, (y + time * vy).abs() <= h);
        }
    }
    let speed2 = vx * vx + vy * vy + vz * vz;
    for sign in [-1.0, 1.0] {
        let cap_y = y - sign * h;
        if let Some(times) = roots(
            speed2,
            x * vx + cap_y * vy + z * vz,
            x * x + cap_y * cap_y + z * z - r2,
        ) {
            for time in times {
                consider(time, sign * (y + time * vy) >= h);
            }
        }
    }
    earliest.is_finite().then_some(earliest as f32)
}

fn half_height(shape: &Capsule) -> Option<f32> {
    let a = shape.segment.a;
    let b = shape.segment.b;
    if a.x.abs() + a.z.abs() + b.x.abs() + b.z.abs() > 1e-6 || (a.y + b.y).abs() > 1e-6 {
        return None;
    }
    Some(a.y.abs().max(b.y.abs()))
}

fn cast_upright(
    pos12: &Pose,
    velocity: &Vec3,
    a: &Capsule,
    b: &Capsule,
    options: ShapeCastOptions,
) -> Result<Option<ShapeCastHit>, Unsupported> {
    if pos12.rotation.x.abs() + pos12.rotation.y.abs() + pos12.rotation.z.abs() > 1e-6 {
        return Err(Unsupported);
    }
    let ha = half_height(a).ok_or(Unsupported)?;
    let hb = half_height(b).ok_or(Unsupported)?;
    let height = ha + hb;
    let radius = a.radius + b.radius + options.target_distance;
    if radius <= 0.0 {
        return Err(Unsupported);
    }
    let origin = pos12.translation;
    let Some(toi) = ray_capsule(
        &origin,
        velocity,
        height,
        radius,
        options.max_time_of_impact,
    ) else {
        return Ok(None);
    };
    let center = origin + *velocity * toi;
    let radial = Vec3::new(
        center.x,
        center.y - center.y.clamp(-height, height),
        center.z,
    );
    let normal1 = radial.try_normalize().ok_or(Unsupported)?;
    if !options.stop_at_penetration && toi < 1e-5 && normal1.dot(*velocity) >= 0.0 {
        return Ok(None);
    }
    let normal2 = pos12.inverse_transform_vector(-normal1);
    let (y1, y2) = if center.y > height {
        (ha, -hb)
    } else if center.y < -height {
        (-ha, hb)
    } else {
        let common = ((-ha).max(center.y - hb) + ha.min(center.y + hb)) * 0.5;
        (common, common - center.y)
    };
    let initial_radial = Vec3::new(
        origin.x,
        origin.y - origin.y.clamp(-height, height),
        origin.z,
    );
    Ok(Some(ShapeCastHit {
        time_of_impact: toi,
        witness1: Vec3::new(0.0, y1, 0.0) + normal1 * a.radius,
        witness2: Vec3::new(0.0, y2, 0.0) + normal2 * b.radius,
        normal1,
        normal2,
        status: if initial_radial.length_squared() < radius * radius {
            ShapeCastStatus::PenetratingOrWithinTargetDist
        } else {
            ShapeCastStatus::Converged
        },
        subshape1: 0,
        subshape2: 0,
    }))
}

/// The Minkowski sum of an upright capsule and an axis-aligned box is a
/// rounded box, extended by the capsule's half-segment on Y. If the entire ray
/// stays inside the two transverse slabs, its hit can only be on a flat face.
/// This common floor/wall case is exact; edges and corners still use Parry.
fn cast_box_face(
    pos12: &Pose,
    velocity: &Vec3,
    cuboid: &Cuboid,
    capsule: &Capsule,
    options: ShapeCastOptions,
) -> Result<Option<ShapeCastHit>, Unsupported> {
    if pos12.rotation.x.abs() + pos12.rotation.y.abs() + pos12.rotation.z.abs() > 1e-6 {
        return Err(Unsupported);
    }
    let half = half_height(capsule).ok_or(Unsupported)?;
    let mut extents = cuboid.half_extents;
    extents.y += half;
    let radius = capsule.radius + options.target_distance;
    let origin = pos12.translation;
    let end = origin + *velocity * options.max_time_of_impact;
    for axis in 0..3 {
        if !(0..3).filter(|&other| other != axis).all(|other| {
            origin[other].abs() <= extents[other] && end[other].abs() <= extents[other]
        }) {
            continue;
        }
        // Deep initial penetration has no unique closest face in this case.
        if origin[axis].abs() <= extents[axis] {
            return Err(Unsupported);
        }
        let sign = origin[axis].signum();
        let gap = origin[axis].abs() - extents[axis] - radius;
        let approaching = velocity[axis] * sign < 0.0;
        let toi = if gap <= 0.0 {
            if !options.stop_at_penetration && !approaching {
                return Ok(None);
            }
            0.0
        } else if approaching {
            let time = -(gap as f64) / (velocity[axis] as f64 * sign as f64);
            if time > options.max_time_of_impact as f64 {
                return Ok(None);
            }
            time as f32
        } else {
            return Ok(None);
        };
        let center = origin + *velocity * toi;
        let mut normal1 = Vec3::ZERO;
        normal1[axis] = sign;
        let normal2 = pos12.inverse_transform_vector(-normal1);
        let box_y = cuboid.half_extents.y;
        let (y1, y2) = if center.y > box_y + half {
            (box_y, -half)
        } else if center.y < -box_y - half {
            (-box_y, half)
        } else {
            let common = ((-box_y).max(center.y - half) + box_y.min(center.y + half)) * 0.5;
            (common, common - center.y)
        };
        return Ok(Some(ShapeCastHit {
            time_of_impact: toi,
            witness1: Vec3::new(
                center
                    .x
                    .clamp(-cuboid.half_extents.x, cuboid.half_extents.x),
                y1,
                center
                    .z
                    .clamp(-cuboid.half_extents.z, cuboid.half_extents.z),
            ),
            witness2: Vec3::new(0.0, y2, 0.0) + normal2 * capsule.radius,
            normal1,
            normal2,
            status: if gap < 0.0 {
                ShapeCastStatus::PenetratingOrWithinTargetDist
            } else {
                ShapeCastStatus::Converged
            },
            subshape1: 0,
            subshape2: 0,
        }));
    }
    Err(Unsupported)
}

impl QueryDispatcher for CharacterQueryDispatcher {
    fn intersection_test(
        &self,
        pos12: &Pose,
        g1: &dyn Shape,
        g2: &dyn Shape,
    ) -> Result<ShapeIntersection, Unsupported> {
        DefaultQueryDispatcher.intersection_test(pos12, g1, g2)
    }
    fn distance(
        &self,
        pos12: &Pose,
        g1: &dyn Shape,
        g2: &dyn Shape,
    ) -> Result<ShapeDistance, Unsupported> {
        DefaultQueryDispatcher.distance(pos12, g1, g2)
    }
    fn contact(
        &self,
        pos12: &Pose,
        g1: &dyn Shape,
        g2: &dyn Shape,
        prediction: Real,
    ) -> Result<Option<Contact>, Unsupported> {
        DefaultQueryDispatcher.contact(pos12, g1, g2, prediction)
    }
    fn closest_points(
        &self,
        pos12: &Pose,
        g1: &dyn Shape,
        g2: &dyn Shape,
        distance: Real,
    ) -> Result<ClosestPoints, Unsupported> {
        DefaultQueryDispatcher.closest_points(pos12, g1, g2, distance)
    }
    fn cast_shapes(
        &self,
        pos12: &Pose,
        velocity: Vector,
        a: &dyn Shape,
        b: &dyn Shape,
        options: ShapeCastOptions,
    ) -> Result<Option<ShapeCastHit>, Unsupported> {
        if let (Some(ca), Some(cb)) = (a.as_capsule(), b.as_capsule())
            && let Ok(result) = cast_upright(pos12, &velocity, ca, cb, options)
        {
            return Ok(result);
        }
        if let (Some(cuboid), Some(capsule)) = (a.as_cuboid(), b.as_capsule())
            && let Ok(result) = cast_box_face(pos12, &velocity, cuboid, capsule, options)
        {
            return Ok(result);
        }
        DefaultQueryDispatcher.cast_shapes(pos12, velocity, a, b, options)
    }
    fn cast_shapes_nonlinear(
        &self,
        m1: &NonlinearRigidMotion,
        a: &dyn Shape,
        m2: &NonlinearRigidMotion,
        b: &dyn Shape,
        start: Real,
        end: Real,
        stop: bool,
    ) -> Result<Option<ShapeCastHit>, Unsupported> {
        DefaultQueryDispatcher.cast_shapes_nonlinear(m1, a, m2, b, start, end, stop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_box_face_hits_match_geometry_and_edges_use_the_general_cast() {
        let capsule = Capsule::new_y(0.55, 0.35);
        let options = ShapeCastOptions {
            max_time_of_impact: 0.2,
            target_distance: 0.02,
            stop_at_penetration: false,
            compute_impact_geometry_on_penetration: true,
        };
        for axis in 0..3 {
            let mut extents = Vec3::splat(20.0);
            extents[axis] = 0.5;
            let cuboid = Cuboid::new(extents);
            for sign in [-1.0, 1.0] {
                for gap in [-0.005, 0.00001, 0.01, 0.1] {
                    let mut origin = Vec3::ZERO;
                    origin[axis] = sign * (0.5 + if axis == 1 { 0.55 } else { 0.0 } + 0.37 + gap);
                    for speed in [-1.0, 0.0, 1.0] {
                        let mut velocity = Vec3::splat(0.1);
                        velocity[axis] = speed;
                        let p = Pose::translation(origin.x, origin.y, origin.z);
                        let fast =
                            cast_box_face(&p, &velocity, &cuboid, &capsule, options).unwrap();
                        assert_eq!(
                            fast.is_some(),
                            speed * sign < 0.0,
                            "axis={axis} p={p:?} v={velocity:?}"
                        );
                        if let Some(fast) = fast {
                            // At unit normal speed the entry time equals the
                            // known initial gap. Check the witness geometry as
                            // well, including the capsule's half-segment on Y.
                            assert!((fast.time_of_impact - gap.max(0.0)).abs() < 1e-6);
                            assert_eq!(fast.normal1[axis], sign);
                            assert!(fast.normal1.dot(velocity) < 0.0);
                            if fast.time_of_impact > 0.0 {
                                let at_hit = origin + velocity * fast.time_of_impact;
                                let gap = (at_hit + fast.witness2 - fast.witness1)
                                    .dot(fast.normal1);
                                assert!((gap - 0.02).abs() < 1e-5);
                            }
                        }
                    }
                }
            }
        }
        assert!(
            cast_box_face(
                &Pose::translation(1.0, 1.45, 0.0),
                &Vec3::new(1.0, -1.0, 0.0),
                &Cuboid::new(Vec3::new(1.0, 0.5, 1.0)),
                &capsule,
                options,
            )
            .is_err(),
            "sweeps near an edge must use the full rounded geometry"
        );
    }

    #[test]
    fn grazing_near_contact_and_vertical_rays_hit_the_capsule_boundary() {
        let height = 1.1;
        let radius = 0.72;
        let tangent = ray_capsule(
            &Vec3::new(radius, 0.0, -2.0),
            &Vec3::Z,
            height,
            radius,
            3.0,
        )
        .unwrap();
        assert!((tangent - 2.0).abs() < 1e-5);
        assert!(
            ray_capsule(
                &Vec3::new(radius + 0.0001, 0.0, -2.0),
                &Vec3::Z,
                height,
                radius,
                3.0
            )
            .is_none()
        );
        assert!(
            ray_capsule(
                &Vec3::new(2.0, 0.0, 0.0),
                &Vec3::ZERO,
                height,
                radius,
                3.0
            )
            .is_none()
        );
        let down = ray_capsule(
            &Vec3::new(0.0, 3.0, 0.0),
            &-Vec3::Y,
            height,
            radius,
            3.0,
        )
        .unwrap();
        assert!((down - (3.0 - height - radius)).abs() < 1e-6);

        let capsule = Capsule::new_y(0.55, 0.35);
        let options = ShapeCastOptions {
            max_time_of_impact: 0.2,
            target_distance: 0.02,
            stop_at_penetration: false,
            compute_impact_geometry_on_penetration: true,
        };
        for index in 0..128 {
            let angle = index as f32 * std::f32::consts::TAU / 128.0;
            let normal = Vec3::new(angle.cos(), 0.0, angle.sin());
            let tangent = Vec3::new(-normal.z, 0.0, normal.x);
            for gap in [0.00001, 0.001, 0.02] {
                let origin = normal * (radius + gap);
                let velocity = -normal * 0.8 + tangent * 0.6;
                let hit = cast_upright(
                    &Pose::translation(origin.x, 0.0, origin.z),
                    &velocity,
                    &capsule,
                    &capsule,
                    options,
                )
                .unwrap()
                .expect("nearby approaching capsules must collide");
                let at_hit = origin + velocity * hit.time_of_impact;
                assert!((at_hit.length() - radius).abs() < 1e-6);
                assert!(hit.normal1.dot(velocity) < 0.0);
                assert!(
                    cast_upright(
                        &Pose::translation(origin.x, 0.0, origin.z),
                        &-velocity,
                        &capsule,
                        &capsule,
                        options
                    )
                    .unwrap()
                    .is_none()
                );
            }
        }
    }

    #[test]
    fn upright_cast_matches_reference_for_sides_caps_and_separating_motion() {
        let a = Capsule::new_y(0.55, 0.35);
        let b = Capsule::new_y(0.55, 0.35);
        for x in [-2.0, -0.9, 0.0, 0.8, 2.0] {
            for y in [-2.5, -1.2, 0.0, 1.3, 2.5] {
                for z in [-1.0, 0.1, 1.0] {
                    let p = Pose::translation(x, y, z);
                    for sign in [-1.0, 1.0] {
                        let velocity = p.translation.normalize() * sign;
                        let options = ShapeCastOptions {
                            max_time_of_impact: 5.0,
                            target_distance: 0.02,
                            stop_at_penetration: false,
                            compute_impact_geometry_on_penetration: true,
                        };
                        let fast = CharacterQueryDispatcher
                            .cast_shapes(&p, velocity, &a, &b, options)
                            .unwrap();
                        let reference = DefaultQueryDispatcher
                            .cast_shapes(&p, velocity, &a, &b, options)
                            .unwrap();
                        assert_eq!(
                            fast.is_some(),
                            reference.is_some(),
                            "p={p:?} v={velocity:?}"
                        );
                        if let (Some(fast), Some(reference)) = (fast, reference) {
                            assert!(
                                (fast.time_of_impact - reference.time_of_impact).abs() < 0.005,
                                "fast={fast:?} reference={reference:?}"
                            );
                            assert!(fast.normal1.dot(reference.normal1) > 0.99);
                            let at_impact = p.translation + velocity * fast.time_of_impact;
                            let gap = (at_impact + fast.witness2 - fast.witness1)
                                .dot(fast.normal1);
                            if fast.time_of_impact > 0.0 {
                                assert!((gap - 0.02).abs() < 0.001);
                            }
                        }
                    }
                }
            }
        }
    }
}
