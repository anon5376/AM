use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SelfPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VisibleEntity {
    pub local_id: u32,
    pub shape_id: u32,
    pub color_id: u32,
    pub size: u32,
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub adjacent: bool,
    pub distance_to_self: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Observation {
    pub tick: u64,
    pub map_seed: u64,
    pub rule_seed: u64,
    #[serde(rename = "self")]
    pub self_position: SelfPosition,
    pub energy_bucket: i32,
    pub reward_delta: i32,
    pub blocked: bool,
    pub visible_entities: Vec<VisibleEntity>,
}
