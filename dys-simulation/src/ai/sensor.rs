use crate::game_objects::game_object::GameObject;

pub struct Sensor<'sim, GameObjectT: GameObject> {
    current_triggers: Vec<&'sim GameObjectT>,
    on_trigger_start_callback: Box<dyn Fn(&'sim GameObjectT)>,
    on_trigger_stop_callback: Box<dyn Fn(&'sim GameObjectT)>,
}

impl<'sim, GameObjectT: GameObject> Sensor<'sim, GameObjectT> {
    pub fn new(
        on_trigger_start: impl Fn(&'sim GameObjectT) + 'static,
        on_trigger_stop: impl Fn(&'sim GameObjectT) + 'static,
    ) -> Sensor<'sim, GameObjectT> {
        Sensor { 
            current_triggers: vec![], 
            on_trigger_start_callback: Box::new(on_trigger_start), 
            on_trigger_stop_callback: Box::new(on_trigger_stop),
        }
    }

    pub fn start_trigger(&mut self, trigger: &'sim GameObjectT) {
        (self.on_trigger_start_callback)(trigger);
        self.current_triggers.push(trigger);
    }

    pub fn stop_trigger(&mut self, trigger: &'sim GameObjectT) {
        (self.on_trigger_stop_callback)(trigger);
        self.current_triggers.retain(|current_trigger| trigger.id() != current_trigger.id());
    }

    pub fn current_triggers(&self) -> &Vec<&'sim GameObjectT> {
        &self.current_triggers
    }
}