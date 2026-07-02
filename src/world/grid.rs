use crate::world::action::Action;
use crate::world::observation::{Observation, SelfPosition, VisibleEntity};
use crate::world::rng::SplitMix64;
use crate::world::theta::WorldTheta;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
struct Entity {
    local_id: u32,
    shape_id: u32,
    color_id: u32,
    size: u32,
    x: i32,
    y: i32,
    rule_code: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorldTrace {
    pub tick: u64,
    pub map_seed: u64,
    pub rule_seed: u64,
    pub theta_hash: String,
    pub action: Action,
    pub x: i32,
    pub y: i32,
    pub energy_bucket: i32,
    pub reward_delta: i32,
    pub blocked: bool,
    pub contact_ids: Vec<u32>,
    pub rule_codes: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct GridWorld {
    theta: WorldTheta,
    map_seed: u64,
    rule_seed: u64,
    theta_hash: String,
    tick: u64,
    self_x: i32,
    self_y: i32,
    energy_bucket: i32,
    reward_delta: i32,
    blocked: bool,
    entities: Vec<Entity>,
}

impl GridWorld {
    pub fn new(theta: WorldTheta, map_seed: u64, rule_seed: u64) -> Result<Self> {
        theta.validate()?;
        let theta_hash = theta.hash();
        let self_x = theta.width / 2;
        let self_y = theta.height / 2;
        let mut map_rng = SplitMix64::new(map_seed);
        let mut rule_rng = SplitMix64::new(rule_seed);
        let mut occupied = BTreeSet::new();
        occupied.insert((self_x, self_y));
        let mut entities = Vec::new();

        for index in 0..theta.entity_count {
            let (x, y) = next_empty_cell(&theta, &mut map_rng, &mut occupied);
            let local_id = (index + 1) as u32;
            entities.push(Entity {
                local_id,
                shape_id: local_id,
                color_id: 100 + local_id,
                size: (index % 3 + 1) as u32,
                x,
                y,
                rule_code: rule_rng.next_range(4) as u32,
            });
        }
        entities.sort_by_key(|entity| entity.local_id);

        Ok(Self {
            energy_bucket: theta.start_energy_bucket,
            theta,
            map_seed,
            rule_seed,
            theta_hash,
            tick: 0,
            self_x,
            self_y,
            reward_delta: 0,
            blocked: false,
            entities,
        })
    }

    pub fn observe(&self) -> Observation {
        let mut visible_entities: Vec<VisibleEntity> = self
            .entities
            .iter()
            .map(|entity| {
                let dx = entity.x - self.self_x;
                let dy = entity.y - self.self_y;
                let distance_to_self = dx.unsigned_abs() + dy.unsigned_abs();
                VisibleEntity {
                    local_id: entity.local_id,
                    shape_id: entity.shape_id,
                    color_id: entity.color_id,
                    size: entity.size,
                    x: entity.x,
                    y: entity.y,
                    dx,
                    dy,
                    adjacent: distance_to_self == 1,
                    distance_to_self,
                }
            })
            .collect();
        visible_entities.sort_by_key(|entity| entity.local_id);
        Observation {
            tick: self.tick,
            map_seed: self.map_seed,
            rule_seed: self.rule_seed,
            self_position: SelfPosition {
                x: self.self_x,
                y: self.self_y,
            },
            energy_bucket: self.energy_bucket,
            reward_delta: self.reward_delta,
            blocked: self.blocked,
            visible_entities,
        }
    }

    pub fn step(&mut self, action: Action) -> (Observation, WorldTrace) {
        self.tick += 1;
        self.reward_delta = 0;
        self.blocked = false;
        let mut contact_ids = Vec::new();
        let mut rule_codes = Vec::new();

        match action {
            Action::N | Action::S | Action::E | Action::W => {
                let (dx, dy) = action.delta();
                let next_x = self.self_x + dx;
                let next_y = self.self_y + dy;
                if !self.in_bounds(next_x, next_y) || self.blocking_at(next_x, next_y) {
                    self.blocked = true;
                    self.apply_reward(self.theta.block_reward);
                } else {
                    self.self_x = next_x;
                    self.self_y = next_y;
                    self.apply_contacts_here(&mut contact_ids, &mut rule_codes);
                }
            }
            Action::PickUp | Action::Drop | Action::Open => {
                self.apply_adjacent_signal(action, &mut contact_ids, &mut rule_codes);
            }
            Action::Wait => {}
        }

        let observation = self.observe();
        let trace = WorldTrace {
            tick: self.tick,
            map_seed: self.map_seed,
            rule_seed: self.rule_seed,
            theta_hash: self.theta_hash.clone(),
            action,
            x: self.self_x,
            y: self.self_y,
            energy_bucket: self.energy_bucket,
            reward_delta: self.reward_delta,
            blocked: self.blocked,
            contact_ids,
            rule_codes,
        };
        (observation, trace)
    }

    pub fn dump_line(&self) -> String {
        format!(
            "world tick={} self=({}, {}) energy={} reward={} blocked={} visible={}",
            self.tick,
            self.self_x,
            self.self_y,
            self.energy_bucket,
            self.reward_delta,
            self.blocked,
            self.entities.len()
        )
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.theta.width && y < self.theta.height
    }

    fn blocking_at(&self, x: i32, y: i32) -> bool {
        self.entities
            .iter()
            .any(|entity| entity.x == x && entity.y == y && entity.rule_code == 0)
    }

    fn apply_contacts_here(&mut self, contact_ids: &mut Vec<u32>, rule_codes: &mut Vec<u32>) {
        let matches: Vec<(u32, u32)> = self
            .entities
            .iter()
            .filter(|entity| entity.x == self.self_x && entity.y == self.self_y)
            .map(|entity| (entity.local_id, entity.rule_code))
            .collect();
        for (local_id, rule_code) in matches {
            contact_ids.push(local_id);
            rule_codes.push(rule_code);
            self.apply_rule_code(rule_code);
        }
    }

    fn apply_adjacent_signal(
        &mut self,
        action: Action,
        contact_ids: &mut Vec<u32>,
        rule_codes: &mut Vec<u32>,
    ) {
        let action_code = match action {
            Action::PickUp => 1,
            Action::Drop => 2,
            Action::Open => 3,
            Action::N | Action::S | Action::E | Action::W | Action::Wait => 0,
        };
        let matches: Vec<(u32, u32)> = self
            .entities
            .iter()
            .filter(|entity| {
                let dx = (entity.x - self.self_x).abs();
                let dy = (entity.y - self.self_y).abs();
                dx + dy == 1
            })
            .map(|entity| (entity.local_id, entity.rule_code))
            .collect();
        for (local_id, rule_code) in matches {
            if rule_code == action_code {
                contact_ids.push(local_id);
                rule_codes.push(rule_code);
                self.apply_reward(self.theta.touch_reward);
            }
        }
        contact_ids.sort_unstable();
        rule_codes.sort_unstable();
    }

    fn apply_rule_code(&mut self, rule_code: u32) {
        match rule_code {
            1 => self.apply_reward(self.theta.touch_reward),
            2 => self.apply_reward(-self.theta.touch_reward),
            3 => self.apply_reward(0),
            _ => {}
        }
    }

    fn apply_reward(&mut self, reward: i32) {
        self.reward_delta += reward;
        self.energy_bucket = (self.energy_bucket + reward)
            .clamp(self.theta.min_energy_bucket, self.theta.max_energy_bucket);
    }
}

fn next_empty_cell(
    theta: &WorldTheta,
    rng: &mut SplitMix64,
    occupied: &mut BTreeSet<(i32, i32)>,
) -> (i32, i32) {
    let cells = (theta.width * theta.height) as u64;
    loop {
        let idx = rng.next_range(cells) as i32;
        let x = idx % theta.width;
        let y = idx / theta.width;
        if occupied.insert((x, y)) {
            return (x, y);
        }
    }
}
