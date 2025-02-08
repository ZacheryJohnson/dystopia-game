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

                for i in 0..total_tick_count {
                    self.tick();
                }
            }
            if ui.button("Tick").clicked() {
                self.tick();
            }

            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                for tick in &self.simmed_ticks {
                    ui.collapsing(format!("Tick {}", tick.tick_number), |ui| {
                        ui.collapsing("Simulation Events", |ui| {
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

                        ui.collapsing("Combatants", |ui| {
                            let states = self.combatant_states_by_tick.get(&tick.tick_number).unwrap();
                            if self.combatant_filter.is_none() {
                                for (combatant_id, combatant_state) in states {
                                    ui.label(format!("{}: {:?}", combatant_id, combatant_state));
                                }
                            } else {
                                let state = states.get(&self.combatant_filter.unwrap()).unwrap();
                                ui.label(format!("{}: {:?}", self.combatant_filter.unwrap(), state));
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

    let seed = [0; 32];
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