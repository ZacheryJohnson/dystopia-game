use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::{Arc, Mutex};
use serde::de::{DeserializeSeed, Error, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use crate::combatant::instance::{CombatantInstance, CombatantInstanceId};
use crate::team::instance::TeamInstance;
use crate::world::World;

impl<'de> Deserialize<'de> for World {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        const FIELDS: &'static [&'static str] = &["combatants", "teams"];
        enum Field {
            Combatants,
            Teams,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                        formatter.write_str(format!("one of [{}]", FIELDS.join(", ")).as_str())
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "combatants" => Ok(Field::Combatants),
                            "teams" => Ok(Field::Teams),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        self.visit_str(std::str::from_utf8(v).unwrap())
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct WorldVisitor;
        impl<'de> Visitor<'de> for WorldVisitor {
            type Value = World;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct World")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut world_instance = World {
                    combatants: vec![],
                    teams: vec![],
                };

                let mut combatants: Vec<CombatantInstance> = vec![];
                let mut partial_teams: Vec<PartialTeamInstance> = vec![];

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Combatants => combatants = map.next_value()?,
                        Field::Teams => partial_teams = map.next_value()?,
                    }
                }

                let combatant_arcs: Vec<Arc<Mutex<CombatantInstance>>> = combatants
                    .into_iter()
                    .map(|combatant| Arc::new(Mutex::new(combatant)))
                    .collect();

                let mut combatant_id_to_arcs: HashMap<CombatantInstanceId, Arc<Mutex<CombatantInstance>>> = HashMap::new();
                for combatant_arc in &combatant_arcs {
                    let combatant_id = combatant_arc.lock().unwrap().id;
                    combatant_id_to_arcs.insert(combatant_id, combatant_arc.clone());
                }

                for partial_team in &mut partial_teams {
                    for combatant_id in &partial_team.team_member_ids {
                        partial_team.team_instance.combatants.push(
                            combatant_id_to_arcs.get(combatant_id).unwrap().clone(),
                        );
                    }
                }

                let teams = partial_teams
                    .into_iter()
                    .map(|partial_team| partial_team.team_instance)
                    .map(|team_instance| Arc::new(Mutex::new(team_instance)))
                    .collect();

                world_instance.combatants = combatant_arcs;
                world_instance.teams = teams;
                Ok(world_instance)
            }
        }

        deserializer.deserialize_struct(
            "World",
            FIELDS,
            WorldVisitor
        )
    }
}

/// PartialTeamInstances have fully populated team_instance fields,
/// except for combatants. Combatant IDs are returned separately,
/// with the caller expected to reconcile that once combatants are parsed.
struct PartialTeamInstance {
    team_instance: TeamInstance,
    team_member_ids: Vec<CombatantInstanceId>,
}

impl<'de> Deserialize<'de> for PartialTeamInstance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        const FIELDS: &'static [&'static str] = &["id", "name", "combatants"];
        enum Field {
            Id,
            Name,
            Combatants,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                        formatter.write_str(format!("one of [{}]", FIELDS.join(", ")).as_str())
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "id" => Ok(Field::Id),
                            "name" => Ok(Field::Name),
                            "combatants" => Ok(Field::Combatants),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        self.visit_str(std::str::from_utf8(v).unwrap())
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct PartialTeamInstanceVisitor;
        impl<'de> Visitor<'de> for PartialTeamInstanceVisitor {
            type Value = PartialTeamInstance;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct PartialTeamInstance")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut partial_team_instance = PartialTeamInstance {
                    team_instance: TeamInstance {
                        id: 0,
                        name: String::new(),
                        combatants: vec![],
                    },
                    team_member_ids: vec![],
                };

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => partial_team_instance.team_instance.id = map.next_value()?,
                        Field::Name => partial_team_instance.team_instance.name = map.next_value()?,
                        Field::Combatants => partial_team_instance.team_member_ids = map.next_value()?,
                    }
                }

                Ok(partial_team_instance)
            }
        }

        deserializer.deserialize_struct(
            "PartialTeamInstance",
            FIELDS,
            PartialTeamInstanceVisitor
        )
    }
}