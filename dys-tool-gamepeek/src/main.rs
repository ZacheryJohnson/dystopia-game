use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use rand::SeedableRng;
use rand_pcg::Pcg64;
use dys_simulation::game::Game;
use dys_simulation::game_objects::combatant::{CombatantId, CombatantState};
use dys_simulation::game_state::GameState;
use dys_simulation::game_tick::{GameTick, GameTickNumber};
use dys_simulation::generator::Generator;
use dys_simulation::simulation::simulate_tick;
use dys_world::arena::Arena;
use dys_world::schedule::calendar::Date;
use dys_world::schedule::calendar::Month::Arguscorp;
use dys_world::schedule::schedule_game::ScheduleGame;

struct GamePeekApp {
    game_state: Arc<Mutex<GameState>>,
    simmed_ticks: Vec<GameTick>,
    combatant_states_by_tick: BTreeMap<GameTickNumber, BTreeMap<CombatantId, CombatantState>>,

    combatant_filter: Option<CombatantId>,
}

impl GamePeekApp {
    fn tick(&mut self) {
        let tick = simulate_tick(self.game_state.clone());
        let tick_number = tick.tick_number;
        self.simmed_ticks.push(tick);

        let game_state = self.game_state.lock().unwrap();
        for (combatant_id, combatant_object) in &game_state.combatants {
            let combatant_state_clone = combatant_object.combatant_state.lock().unwrap().to_owned();
            match self.combatant_states_by_tick.entry(tick_number) {
                Entry::Vacant(entry) => {
                    let mut new_map = BTreeMap::new();
                    new_map.insert(*combatant_id, combatant_state_clone);
                    entry.insert(new_map);
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().insert(*combatant_id, combatant_state_clone);
                }
            }
        }
    }
}

impl eframe::App for GamePeekApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ComboBox::from_label("Combatant Filter")
                .selected_text(if self.combatant_filter.is_some() { format!("{}", self.combatant_filter.unwrap()) } else { "(none)".to_string() })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.combatant_filter, None, "(none)");

                    let game_state = self.game_state.lock().unwrap();
                    for (combatant_id, _) in &game_state.combatants {
                        ui.selectable_value(
                            &mut self.combatant_filter,
                            Some(*combatant_id),
                            format!("{combatant_id}")
                        );
                    }
                });

            if ui.button("Simulate All").clicked() {
                let total_tick_count = {
                    let game_state = self.game_state.lock().unwrap();
                    game_state.simulation_config.ticks_per_game()
                };

                let manually_ticked_count = self.simmed_ticks.len() as u32;

                for _ in manually_ticked_count..total_tick_count {
                    self.tick();
                }
            }
            if ui.button("Tick").clicked() {
                self.tick();
            }

            let make_collapseable = |header: String, _: GameTickNumber| {
                egui::CollapsingHeader::new(header)
                    .default_open(self.combatant_filter.is_some())
            };

            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                for tick in &self.simmed_ticks {
                    // We don't want the tick collapseable to automatically open,
                    // so use ui.collapsing instead.
                    // All other collapseables should use make_collapseable
                    ui.collapsing(format!("Tick {}", tick.tick_number), |ui| {
                        make_collapseable("Simulation Events".to_string(), tick.tick_number).show(ui, |ui| {
                            if self.combatant_filter.is_none() {
                                for evt in &tick.simulation_events {
                                    ui.label(format!("{:?}", evt));
                                }
                            } else {
                                // ZJ-TODO: fix this
                                let filtered_events = tick
                                    .simulation_events
                                    .iter()
                                    .filter(|evt| format!("{evt:?}").contains(&format!("combatant_id: {}", self.combatant_filter.unwrap())))
                                    .collect::<Vec<_>>();

                                for evt in &filtered_events {
                                    ui.label(format!("{:?}", evt));
                                }
                            }
                        });

                        make_collapseable("Combatants".to_string(), tick.tick_number).show(ui, |ui| {
                            let states = if self.combatant_filter.is_some() {
                                let combatant_id = self.combatant_filter.unwrap();
                                let combatant_state = self
                                    .combatant_states_by_tick
                                    .get(&tick.tick_number)
                                    .unwrap()
                                    .get(&combatant_id)
                                    .unwrap();

                                BTreeMap::from([(combatant_id, combatant_state.to_owned())])
                            } else {
                                self.combatant_states_by_tick.get(&tick.tick_number).unwrap().to_owned()
                            };

                            for (combatant_id, combatant_state) in states {
                                make_collapseable(format!("{combatant_id}"), tick.tick_number).show(ui, |ui| {
                                    ui.label(format!("On Plate: {:?}", combatant_state.on_plate));
                                    ui.label(format!("Holding Ball: {:?}", combatant_state.holding_ball));
                                    ui.label(format!("Is Stunned: {:?}", combatant_state.stunned));

                                    make_collapseable("AI".to_string(), tick.tick_number).show(ui, |ui| {
                                        let current_action_name = if combatant_state.current_action.is_some() {
                                            combatant_state.current_action.unwrap().name()
                                        } else {
                                            String::from("(none)")
                                        };

                                        let completed_action_name = if combatant_state.completed_action.is_some() {
                                            combatant_state.completed_action.unwrap().name()
                                        } else {
                                            String::from("(none)")
                                        };

                                        ui.label(format!("Completed Action: {completed_action_name}"));
                                        ui.label(format!("Current Action: {current_action_name}"));
                                        for action in combatant_state.plan.iter().rev() {
                                            ui.label(format!("Planned Action: {}", action.name()));
                                        }
                                        make_collapseable("Beliefs".to_string(), tick.tick_number).show(ui, |ui| {
                                            for (source, beliefs) in &combatant_state.beliefs.sourced_beliefs() {
                                                for belief in beliefs {
                                                    ui.label(format!("({source}) {:?} (t {:?})", belief.belief, belief.expires_on_tick));
                                                }
                                            }
                                        });
                                    });
                                });
                            }
                        });
                    });
                }
            });
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    let seed = [13; 32];
    let generator = Generator::new();
    let world = generator.generate_world(&mut Pcg64::from_seed(seed.to_owned()));

    let app = GamePeekApp {
        game_state: Arc::new(Mutex::new(GameState::from_game_seeded(
            Game {
                schedule_game: ScheduleGame {
                    away_team: world.teams[0].to_owned(),
                    home_team: world.teams[1].to_owned(),
                    arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
                    date: Date(Arguscorp, 1, 1000),
                },
            },
            &seed
        ))),
        simmed_ticks: vec![],
        combatant_states_by_tick: BTreeMap::new(),
        combatant_filter: None,
    };

    eframe::run_native("Dystopia Game Peek", options, Box::new(|_cc| Ok(Box::new(app))))
}