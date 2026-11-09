use serde::{Deserialize, Serialize};

/// One simulation tick. Movement is right/forward; look deltas are radians.
/// Device adapters convert pixels, touches or gyro samples into these units.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerInputCommand {
    pub tick_id: u64,
    pub move_vec: [f32; 2],
    pub look_delta: [f32; 2],
    pub jump: bool,
    pub sprint: bool,
}

/// Held actions survive ticks. Look and jump survive frames without a tick and
/// are consumed only by the next tick, including during multi-tick catch-up.
#[derive(Debug, Default)]
pub struct InputAccumulator {
    next_tick: u64,
    movement: [f32; 2],
    look: [f32; 2],
    jump: bool,
    sprint: bool,
}

impl InputAccumulator {
    pub fn set_movement(&mut self, movement: [f32; 2], sprint: bool) {
        self.movement = normalized_axes(movement);
        self.sprint = sprint;
    }

    pub fn add_look(&mut self, delta: [f32; 2]) {
        for (pending, value) in self.look.iter_mut().zip(delta) {
            if value.is_finite() {
                // A complete revolution per pending tick is already more than
                // useful input; keep accidental/untrusted values bounded.
                *pending = (*pending + value).clamp(-std::f32::consts::TAU, std::f32::consts::TAU);
            }
        }
    }

    pub fn queue_jump(&mut self) {
        self.jump = true;
    }

    pub fn pending_look(&self) -> [f32; 2] {
        self.look
    }

    /// Drop all device state on focus loss or UI entry without rewinding time.
    pub fn clear(&mut self) {
        self.movement = [0.0; 2];
        self.look = [0.0; 2];
        self.jump = false;
        self.sprint = false;
    }

    pub fn next_command(&mut self) -> PlayerInputCommand {
        let command = PlayerInputCommand {
            tick_id: self.next_tick,
            move_vec: self.movement,
            look_delta: std::mem::take(&mut self.look),
            jump: std::mem::take(&mut self.jump),
            sprint: self.sprint,
        };
        self.next_tick = self.next_tick.wrapping_add(1);
        command
    }
}

pub fn normalized_axes(axes: [f32; 2]) -> [f32; 2] {
    let [x, y] = axes.map(|axis| {
        if axis.is_finite() {
            axis.clamp(-1.0, 1.0)
        } else {
            0.0
        }
    });
    let length = x.hypot(y).max(1.0);
    [x / length, y / length]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shot_input_waits_for_a_tick_and_is_not_replayed() {
        let mut input = InputAccumulator::default();
        input.set_movement([0.0, 1.0], true);
        input.add_look([0.1, 0.2]);
        input.add_look([0.2, -0.1]);
        input.queue_jump();
        let first = input.next_command();
        assert!(first.jump);
        assert!((first.look_delta[0] - 0.3).abs() < 1e-6);
        for expected_tick in 1..4 {
            let command = input.next_command();
            assert_eq!(command.tick_id, expected_tick);
            assert_eq!(command.move_vec, [0.0, 1.0]);
            assert!(command.sprint);
            assert!(!command.jump);
            assert_eq!(command.look_delta, [0.0; 2]);
        }
    }

    #[test]
    fn focus_loss_discards_pending_actions_but_preserves_tick_sequence() {
        let mut input = InputAccumulator::default();
        input.next_command();
        input.set_movement([1.0, 1.0], true);
        input.add_look([0.2, 0.1]);
        input.queue_jump();
        input.clear();
        assert_eq!(
            input.next_command(),
            PlayerInputCommand {
                tick_id: 1,
                ..Default::default()
            }
        );
    }

    #[test]
    fn diagonal_and_untrusted_axes_are_bounded() {
        let [x, y] = normalized_axes([1.0, 1.0]);
        assert!((x.hypot(y) - 1.0).abs() < 1e-6);
        assert_eq!(normalized_axes([f32::NAN, f32::INFINITY]), [0.0; 2]);
        assert_eq!(normalized_axes([0.25, 0.0]), [0.25, 0.0]);
    }
}
