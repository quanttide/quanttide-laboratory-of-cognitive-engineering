pub use quanttide_think::{
    thought::Thought,
    intention::{Intention, Agent, Level, Priority, Trigger, Risk},
    situation::{Situation, SituationContent},
    schema::{Schema, SchemaContent, Entity, Causal, KeyValue, Mapping, Bias},
    domain::Domain,
};

use std::collections::HashMap;

pub type WeekData = (Vec<Situation>, Vec<Intention>, HashMap<String, Vec<Intention>>);
