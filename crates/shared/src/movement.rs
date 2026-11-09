use crate::input::{PlayerInputCommand, normalized_axes};

pub const FIXED_HZ: f64 = 60.0;
pub const FIXED_DT: f32 = 1.0 / FIXED_HZ as f32;
pub const WALK_SPEED: f32 = 4.0;
pub const SPRINT_SPEED: f32 = 7.0;
pub const GRAVITY: f32 = -18.0;
pub const JUMP_SPEED: f32 = 6.0;
pub const MAX_PITCH: f32 = 89.0 * std::f32::consts::PI / 180.0;
pub const PLAYER_RADIUS: f32 = 0.35;
pub const PLAYER_HALF_SEGMENT: f32 = 0.55;
pub const EYE_OFFSET: f32 = 0.7;

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MotionState {
    pub yaw: f32,
    pub pitch: f32,
    pub vertical_velocity: f32,
    pub grounded: bool,
}

impl MotionState {
    /// Produce a requested displacement. Rapier resolves this against the world;
    /// `finish_step` then feeds the result back into the shared controller.
    pub fn plan_step(&mut self, input: PlayerInputCommand) -> [f32; 3] {
        let [yaw, pitch] = preview_look(self.yaw, self.pitch, input.look_delta);
        self.yaw = yaw;
        self.pitch = pitch;

        if self.grounded && self.vertical_velocity < 0.0 {
            self.vertical_velocity = 0.0;
        }
        if input.jump && self.grounded {
            self.vertical_velocity = JUMP_SPEED;
            self.grounded = false;
        }
        self.vertical_velocity += GRAVITY * FIXED_DT;

        let [right, forward] = normalized_axes(input.move_vec);
        let speed = if input.sprint {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };
        let (sin, cos) = self.yaw.sin_cos();
        [
            (cos * right - sin * forward) * speed * FIXED_DT,
            self.vertical_velocity * FIXED_DT,
            (-sin * right - cos * forward) * speed * FIXED_DT,
        ]
    }

    pub fn finish_step(&mut self, grounded: bool, requested_y: f32, actual_y: f32) {
        self.grounded = grounded;
        if (grounded && self.vertical_velocity < 0.0)
            || (requested_y > 0.0 && actual_y < requested_y - 0.001)
        {
            self.vertical_velocity = 0.0;
        }
    }
}

pub fn preview_look(yaw: f32, pitch: f32, delta: [f32; 2]) -> [f32; 2] {
    let finite = delta.map(|value| if value.is_finite() { value } else { 0.0 });
    [
        (yaw + finite[0]).rem_euclid(std::f32::consts::TAU),
        (pitch + finite[1]).clamp(-MAX_PITCH, MAX_PITCH),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::InputAccumulator;

    #[test]
    fn diagonal_distance_matches_straight_and_sprint_has_explicit_speed() {
        let step = |axes, sprint| {
            let d = MotionState::default().plan_step(PlayerInputCommand {
                move_vec: axes,
                sprint,
                ..Default::default()
            });
            d[0].hypot(d[2]) / FIXED_DT
        };
        assert!((step([1.0, 1.0], false) - WALK_SPEED).abs() < 1e-5);
        assert!((step([0.0, 1.0], true) - SPRINT_SPEED).abs() < 1e-5);
    }

    #[test]
    fn jumping_requires_ground_contact_and_ceiling_stops_upward_velocity() {
        let jump = PlayerInputCommand {
            jump: true,
            ..Default::default()
        };
        let mut airborne = MotionState::default();
        assert!(airborne.plan_step(jump)[1] < 0.0);
        let mut grounded = MotionState {
            grounded: true,
            ..Default::default()
        };
        let displacement = grounded.plan_step(jump);
        assert!(displacement[1] > 0.0);
        let first_velocity = grounded.vertical_velocity;
        grounded.plan_step(jump);
        assert!(grounded.vertical_velocity < first_velocity);
        grounded.finish_step(false, displacement[1], 0.0);
        assert_eq!(grounded.vertical_velocity, 0.0);
    }

    #[test]
    fn rendering_frequency_does_not_change_distance_or_total_look() {
        fn run(fps: u32) -> ([f32; 3], MotionState) {
            let mut accumulator = InputAccumulator::default();
            let mut state = MotionState::default();
            let mut position = [0.0; 3];
            let mut tick_budget = 0;
            for _ in 0..fps {
                accumulator.set_movement([0.0, 1.0], false);
                // Look tested separately from movement to avoid sampling curved
                // trajectories at different event timestamps.
                tick_budget += 60;
                while tick_budget >= fps {
                    tick_budget -= fps;
                    let delta = state.plan_step(accumulator.next_command());
                    for (p, d) in position.iter_mut().zip(delta) {
                        *p += d;
                    }
                }
            }
            for _ in 0..fps {
                accumulator.add_look([1.0 / fps as f32, 0.0]);
            }
            state.plan_step(accumulator.next_command());
            (position, state)
        }
        let (reference, _) = run(60);
        for fps in [30, 60, 120, 144, 240] {
            let (position, motion) = run(fps);
            assert!((position[2] - reference[2]).abs() < 1e-5);
            assert!((motion.yaw - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn look_clamps_pitch_and_ignores_non_finite_input() {
        assert_eq!(preview_look(0.0, 0.0, [f32::NAN, 100.0]), [0.0, MAX_PITCH]);
    }
}
