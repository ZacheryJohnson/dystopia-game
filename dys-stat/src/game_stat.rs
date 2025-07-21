use dys_simulation::game_objects::combatant::CombatantId;
use dys_simulation::simulation::simulation_event::SimulationEvent;

type ValueFnT<ValueT> = fn(CombatantId, &Vec<SimulationEvent>) -> Option<ValueT>;

pub trait GameStat {
    type ValueT;

    fn name(&self) -> impl Into<String>;
    fn value_fn(&self) -> ValueFnT<Self::ValueT>;

    fn calculate(&self, combatant_id: CombatantId, events: &Vec<SimulationEvent>) -> Option<Self::ValueT> {
        self.value_fn()(combatant_id, events)
    }
}

#[derive(Debug)]
pub struct GameStatPointsScored;
impl GameStat for GameStatPointsScored {
    type ValueT = i32;

    fn name(&self) -> impl Into<String> { "Points" }

    fn value_fn(&self) -> ValueFnT<Self::ValueT> {
        |combatant_id, events| {
            Some(
                events
                    .iter()
                    .filter(|evt| matches!(evt, SimulationEvent::PointsScoredByCombatant { combatant_id: cid, .. } if *cid == combatant_id ))
                    .map(|evt| {
                        let SimulationEvent::PointsScoredByCombatant { points, ..} = evt else {
                            return 0;
                        };

                        points.to_owned() as Self::ValueT
                    })
                    .sum()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_stat_points_scored() {
        let combatant_id = 1;
        let events = vec![
            SimulationEvent::PointsScoredByCombatant {
                plate_id: 1,
                combatant_id,
                points: 2,
            }
        ];

        let value = GameStatPointsScored.calculate(combatant_id, &events);
        assert_eq!(Some(2), value);
    }
}