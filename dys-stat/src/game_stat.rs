use dys_simulation::simulation::simulation_event::SimulationEvent;
use dys_world::combatant::instance::CombatantInstanceId;

type ValueFnT<ValueT> = fn(CombatantInstanceId, &Vec<SimulationEvent>) -> Option<ValueT>;

pub trait GameStat {
    type ValueT;

    fn name(&self) -> impl Into<String>;
    fn value_fn(&self) -> ValueFnT<Self::ValueT>;

    fn calculate(&self, combatant_id: CombatantInstanceId, events: &Vec<SimulationEvent>) -> Option<Self::ValueT> {
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

    #[test]
    fn test_multiple_combatants() {
        let events = vec![
            SimulationEvent::PointsScoredByCombatant {
                plate_id: 1,
                combatant_id: 1,
                points: 1,
            },
            SimulationEvent::PointsScoredByCombatant {
                plate_id: 1,
                combatant_id: 2,
                points: 1,
            },
            SimulationEvent::PointsScoredByCombatant {
                plate_id: 1,
                combatant_id: 1,
                points: 2,
            },
        ];

        assert_eq!(Some(3), GameStatPointsScored.calculate(1, &events));
        assert_eq!(Some(1), GameStatPointsScored.calculate(2, &events));
        assert_eq!(Some(0), GameStatPointsScored.calculate(3, &events));
    }
}