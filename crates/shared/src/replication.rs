use crate::protocol::{PlayerId, PlayerState};
use std::collections::{BTreeMap, BTreeSet};

pub const CELL_SIZE: f32 = 64.0;
pub const ENTER_RADIUS: f32 = 128.0;
pub const LEAVE_RADIUS: f32 = 160.0;

type CellPlayers = Vec<(PlayerId, [f32; 3])>;

#[derive(Default)]
pub struct SpatialIndex {
    cells: BTreeMap<(i32, i32), CellPlayers>,
}

impl SpatialIndex {
    pub fn build(states: impl IntoIterator<Item = (PlayerId, PlayerState)>) -> Self {
        let mut index = Self::default();
        for (id, state) in states {
            index
                .cells
                .entry(Self::cell(state.position))
                .or_default()
                .push((id, state.position));
        }
        index
    }

    fn cell(p: [f32; 3]) -> (i32, i32) {
        (
            (p[0] / CELL_SIZE).floor() as i32,
            (p[2] / CELL_SIZE).floor() as i32,
        )
    }

    pub fn visible(
        &self,
        owner: PlayerId,
        p: [f32; 3],
        previous: &BTreeSet<PlayerId>,
    ) -> BTreeSet<PlayerId> {
        let (cx, cz) = Self::cell(p);
        let mut result = BTreeSet::new();
        for x in cx - 3..=cx + 3 {
            for z in cz - 3..=cz + 3 {
                if let Some(players) = self.cells.get(&(x, z)) {
                    for &(id, other) in players {
                        let radius = if previous.contains(&id) {
                            LEAVE_RADIUS
                        } else {
                            ENTER_RADIUS
                        };
                        let distance2 = (other[0] - p[0]).powi(2)
                            + (other[1] - p[1]).powi(2)
                            + (other[2] - p[2]).powi(2);
                        if id != owner && distance2 <= radius * radius {
                            result.insert(id);
                        }
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hysteresis_crosses_negative_cells_and_does_not_cap_a_crowd() {
        let states = (0..150).map(|id| {
            (
                PlayerId(id),
                PlayerState::at([-0.1 + id as f32 * 0.01, 1.0, -0.1]),
            )
        });
        let index = SpatialIndex::build(states);
        assert_eq!(
            index
                .visible(PlayerId(0), [-0.1, 1.0, -0.1], &BTreeSet::new())
                .len(),
            149
        );
        let index = SpatialIndex::build([(PlayerId(1), PlayerState::at([140.0, 1.0, 0.0]))]);
        assert!(
            index
                .visible(PlayerId(0), [0.0, 1.0, 0.0], &BTreeSet::new())
                .is_empty()
        );
        assert_eq!(
            index
                .visible(PlayerId(0), [0.0, 1.0, 0.0], &BTreeSet::from([PlayerId(1)]))
                .len(),
            1
        );
        assert!(
            SpatialIndex::default()
                .visible(PlayerId(0), [0.0; 3], &BTreeSet::from([PlayerId(1)]))
                .is_empty()
        );
    }
}
