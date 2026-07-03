use crate::world::action::Action;
use crate::world::classes;
use crate::world::observation::{Observation, SelfPosition, VisibleEntity};
use crate::world::rng::SplitMix64;
use crate::world::theta::WorldTheta;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug)]
struct Entity {
    local_id: u32,
    behavior_class: u32,
    shape_id: u32,
    color_id: u32,
    size: u32,
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerminationCause {
    Exit,
    EnergyDeath,
    TickCap,
    ScriptEnd,
}

impl fmt::Display for TerminationCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            TerminationCause::Exit => "Exit",
            TerminationCause::EnergyDeath => "EnergyDeath",
            TerminationCause::TickCap => "TickCap",
            TerminationCause::ScriptEnd => "ScriptEnd",
        };
        f.write_str(text)
    }
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
    pub action_failed: bool,
    pub held_id: Option<u32>,
    pub contact_ids: Vec<u32>,
    pub removed_ids: Vec<u32>,
    pub termination: Option<TerminationCause>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldRenderEntity {
    pub local_id: u32,
    pub behavior_class: u32,
    pub shape_id: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldRenderFrame {
    pub tick: u64,
    pub width: i32,
    pub height: i32,
    pub self_x: i32,
    pub self_y: i32,
    pub energy_bucket: i32,
    pub reward_delta: i32,
    pub blocked: bool,
    pub action_failed: bool,
    pub held_shape_id: Option<u32>,
    pub held_behavior_class: Option<u32>,
    pub termination: Option<TerminationCause>,
    pub entities: Vec<WorldRenderEntity>,
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
    action_failed: bool,
    termination: Option<TerminationCause>,
    entities: Vec<Entity>,
    held: Option<Entity>,
    matching_table: BTreeMap<u32, u32>,
    class_shapes: BTreeMap<u32, u32>,
}

impl GridWorld {
    pub fn new(theta: WorldTheta, map_seed: u64, rule_seed: u64) -> Result<Self> {
        theta.validate()?;
        let theta_hash = theta.hash();
        let self_x = theta.width / 2;
        let self_y = theta.height / 2;
        let class_shapes = classes::class_shape_map(
            rule_seed,
            theta.portable_class_count,
            theta.barrier_class_count,
        );
        let matching_table = classes::matching_table(
            rule_seed,
            theta.portable_class_count,
            theta.barrier_class_count,
        );
        let mut map_rng = SplitMix64::new(map_seed);
        let mut occupied = BTreeSet::new();
        occupied.insert((self_x, self_y));
        let mut entities = Vec::new();
        let mut next_id = 1_u32;
        let mut placement = EntityPlacement {
            theta: &theta,
            rng: &mut map_rng,
            occupied: &mut occupied,
            class_shapes: &class_shapes,
            entities: &mut entities,
            next_id: &mut next_id,
        };

        placement.push(classes::WALL, theta.wall_count);
        for class in classes::portable_classes(theta.portable_class_count) {
            placement.push(class, theta.portables_per_class);
        }
        for class in classes::barrier_classes(theta.barrier_class_count) {
            placement.push(class, theta.barriers_per_class);
        }
        placement.push(classes::CONSUMABLE, theta.consumable_count);
        placement.push(classes::HAZARD, theta.hazard_count);
        placement.push(classes::EXIT, theta.exit_count);
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
            action_failed: false,
            termination: None,
            entities,
            held: None,
            matching_table,
            class_shapes,
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
            held_shape_id: self.held.as_ref().map(|entity| entity.shape_id),
            visible_entities,
        }
    }

    pub fn step(&mut self, action: Action) -> (Observation, WorldTrace) {
        self.tick += 1;
        let energy_before_tick = self.energy_bucket;
        self.reward_delta = 0;
        self.blocked = false;
        self.action_failed = false;
        self.termination = None;
        let mut contact_ids = Vec::new();
        let mut removed_ids = Vec::new();
        let mut suppress_tick_energy = false;

        match action {
            Action::N | Action::S | Action::E | Action::W => {
                let (dx, dy) = action.delta();
                let next_x = self.self_x + dx;
                let next_y = self.self_y + dy;
                if !self.in_bounds(next_x, next_y) || self.blocking_at(next_x, next_y) {
                    self.blocked = true;
                    suppress_tick_energy = true;
                } else {
                    self.self_x = next_x;
                    self.self_y = next_y;
                    self.consume_here(&mut contact_ids, &mut removed_ids);
                }
            }
            Action::PickUp => self.pick_up_here(&mut contact_ids),
            Action::Drop => self.drop_here(&mut contact_ids),
            Action::Open => self.open_adjacent(&mut contact_ids, &mut removed_ids),
            Action::Wait => {}
        }

        if !suppress_tick_energy {
            self.apply_hazard_here(&mut contact_ids);
            self.apply_step_cost();
        }
        self.reward_delta = self.energy_bucket - energy_before_tick;
        self.termination = self.termination_after_effects();

        contact_ids.sort_unstable();
        contact_ids.dedup();
        removed_ids.sort_unstable();
        removed_ids.dedup();

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
            action_failed: self.action_failed,
            held_id: self.held.as_ref().map(|entity| entity.local_id),
            contact_ids,
            removed_ids,
            termination: self.termination,
        };
        (observation, trace)
    }

    pub fn render_frame(&self) -> WorldRenderFrame {
        let entities = self
            .entities
            .iter()
            .map(|entity| WorldRenderEntity {
                local_id: entity.local_id,
                behavior_class: entity.behavior_class,
                shape_id: entity.shape_id,
                x: entity.x,
                y: entity.y,
            })
            .collect();
        WorldRenderFrame {
            tick: self.tick,
            width: self.theta.width,
            height: self.theta.height,
            self_x: self.self_x,
            self_y: self.self_y,
            energy_bucket: self.energy_bucket,
            reward_delta: self.reward_delta,
            blocked: self.blocked,
            action_failed: self.action_failed,
            held_shape_id: self.held.as_ref().map(|entity| entity.shape_id),
            held_behavior_class: self.held.as_ref().map(|entity| entity.behavior_class),
            termination: self.termination,
            entities,
        }
    }

    pub fn class_shape_pairs(&self) -> Vec<(u32, u32)> {
        self.class_shapes
            .iter()
            .map(|(class, shape)| (*class, *shape))
            .collect()
    }

    pub fn dump_line(&self) -> String {
        format!(
            "world tick={} self=({}, {}) energy={} delta={} blocked={} failed={} held={:?} visible={} termination={}",
            self.tick,
            self.self_x,
            self.self_y,
            self.energy_bucket,
            self.reward_delta,
            self.blocked,
            self.action_failed,
            self.held.as_ref().map(|entity| entity.shape_id),
            self.entities.len(),
            self.termination
                .map(|cause| cause.to_string())
                .unwrap_or_else(|| "None".to_string())
        )
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.theta.width && y < self.theta.height
    }

    fn blocking_at(&self, x: i32, y: i32) -> bool {
        self.entities.iter().any(|entity| {
            entity.x == x && entity.y == y && classes::is_blocking(entity.behavior_class)
        })
    }

    fn pick_up_here(&mut self, contact_ids: &mut Vec<u32>) {
        if self.held.is_some() {
            self.action_failed = true;
            return;
        }
        let Some(index) =
            self.entity_index_at_class(self.self_x, self.self_y, classes::is_portable)
        else {
            self.action_failed = true;
            return;
        };
        let entity = self.entities.remove(index);
        contact_ids.push(entity.local_id);
        self.held = Some(entity);
    }

    fn drop_here(&mut self, contact_ids: &mut Vec<u32>) {
        let Some(mut entity) = self.held.take() else {
            self.action_failed = true;
            return;
        };
        if self.entity_index_at(self.self_x, self.self_y).is_some() {
            self.action_failed = true;
            self.held = Some(entity);
            return;
        }
        entity.x = self.self_x;
        entity.y = self.self_y;
        contact_ids.push(entity.local_id);
        self.entities.push(entity);
        self.entities.sort_by_key(|entity| entity.local_id);
    }

    fn open_adjacent(&mut self, contact_ids: &mut Vec<u32>, removed_ids: &mut Vec<u32>) {
        let Some(target_index) = self.adjacent_barrier_index() else {
            self.action_failed = true;
            return;
        };
        let Some(held) = self.held.as_ref() else {
            self.action_failed = true;
            return;
        };
        let target_class = self.entities[target_index].behavior_class;
        if self.matching_table.get(&held.behavior_class) != Some(&target_class) {
            self.action_failed = true;
            return;
        }
        let removed = self.entities.remove(target_index);
        contact_ids.push(removed.local_id);
        removed_ids.push(removed.local_id);
    }

    fn adjacent_barrier_index(&self) -> Option<usize> {
        for action in [Action::N, Action::S, Action::E, Action::W] {
            let (dx, dy) = action.delta();
            let x = self.self_x + dx;
            let y = self.self_y + dy;
            if let Some(index) = self.entity_index_at_class(x, y, classes::is_barrier) {
                return Some(index);
            }
        }
        None
    }

    fn consume_here(&mut self, contact_ids: &mut Vec<u32>, removed_ids: &mut Vec<u32>) {
        let Some(index) = self.entity_index_at_class(self.self_x, self.self_y, |class| {
            class == classes::CONSUMABLE
        }) else {
            return;
        };
        let removed = self.entities.remove(index);
        contact_ids.push(removed.local_id);
        removed_ids.push(removed.local_id);
        self.apply_energy(self.theta.consumable_energy);
    }

    fn apply_hazard_here(&mut self, contact_ids: &mut Vec<u32>) {
        let matches = self
            .entities
            .iter()
            .filter(|entity| {
                entity.x == self.self_x
                    && entity.y == self.self_y
                    && entity.behavior_class == classes::HAZARD
            })
            .map(|entity| entity.local_id)
            .collect::<Vec<_>>();
        if matches.is_empty() {
            return;
        }
        contact_ids.extend(matches);
        self.apply_energy(-self.theta.hazard_energy);
    }

    fn apply_step_cost(&mut self) {
        if self.tick.is_multiple_of(self.theta.step_cost_interval) {
            self.apply_energy(-1);
        }
    }

    fn apply_energy(&mut self, delta: i32) {
        self.energy_bucket = (self.energy_bucket + delta)
            .clamp(self.theta.min_energy_bucket, self.theta.max_energy_bucket);
    }

    fn termination_after_effects(&self) -> Option<TerminationCause> {
        if self.entities.iter().any(|entity| {
            entity.x == self.self_x
                && entity.y == self.self_y
                && entity.behavior_class == classes::EXIT
        }) {
            return Some(TerminationCause::Exit);
        }
        if self.energy_bucket <= self.theta.min_energy_bucket {
            return Some(TerminationCause::EnergyDeath);
        }
        if self.tick as usize >= self.theta.step_limit {
            return Some(TerminationCause::TickCap);
        }
        None
    }

    fn entity_index_at(&self, x: i32, y: i32) -> Option<usize> {
        self.entities
            .iter()
            .position(|entity| entity.x == x && entity.y == y)
    }

    fn entity_index_at_class(
        &self,
        x: i32,
        y: i32,
        predicate: impl Fn(u32) -> bool,
    ) -> Option<usize> {
        self.entities
            .iter()
            .position(|entity| entity.x == x && entity.y == y && predicate(entity.behavior_class))
    }
}

struct EntityPlacement<'a> {
    theta: &'a WorldTheta,
    rng: &'a mut SplitMix64,
    occupied: &'a mut BTreeSet<(i32, i32)>,
    class_shapes: &'a BTreeMap<u32, u32>,
    entities: &'a mut Vec<Entity>,
    next_id: &'a mut u32,
}

impl EntityPlacement<'_> {
    fn push(&mut self, behavior_class: u32, count: usize) {
        for _ in 0..count {
            let (x, y) = next_empty_cell(self.theta, self.rng, self.occupied);
            let color_id = 100 + self.rng.next_range(900) as u32;
            let size = 1 + self.rng.next_range(3) as u32;
            self.entities.push(Entity {
                local_id: *self.next_id,
                behavior_class,
                shape_id: *self
                    .class_shapes
                    .get(&behavior_class)
                    .expect("class shape must exist for every behavior class"),
                color_id,
                size,
                x,
                y,
            });
            *self.next_id += 1;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn blank_theta() -> WorldTheta {
        WorldTheta {
            width: 5,
            height: 5,
            portable_class_count: 2,
            barrier_class_count: 2,
            wall_count: 0,
            barriers_per_class: 0,
            portables_per_class: 0,
            consumable_count: 0,
            hazard_count: 0,
            exit_count: 0,
            start_energy_bucket: 10,
            min_energy_bucket: 0,
            max_energy_bucket: 20,
            consumable_energy: 3,
            hazard_energy: 2,
            step_cost_interval: 20,
            step_limit: 400,
            twins: false,
            motion: false,
            confound: false,
            vision_radius: None,
            rule_resample: false,
        }
    }

    fn fixture() -> GridWorld {
        GridWorld::new(blank_theta(), 1, 2).unwrap()
    }

    fn entity(local_id: u32, behavior_class: u32, x: i32, y: i32) -> Entity {
        Entity {
            local_id,
            behavior_class,
            shape_id: behavior_class + 1,
            color_id: 100 + local_id,
            size: 1,
            x,
            y,
        }
    }

    #[test]
    fn wall_blocks_and_does_not_change_energy() {
        let mut world = fixture();
        world.self_x = 2;
        world.self_y = 2;
        world.entities = vec![entity(1, classes::WALL, 2, 1)];
        let (_, trace) = world.step(Action::N);
        assert!(trace.blocked);
        assert!(!trace.action_failed);
        assert_eq!((trace.x, trace.y), (2, 2));
        assert_eq!(trace.reward_delta, 0);
        assert_eq!(trace.energy_bucket, 10);
    }

    #[test]
    fn barrier_blocks_pre_open_matching_portable_removes_it_and_cell_becomes_passable() {
        let mut world = fixture();
        world.self_x = 2;
        world.self_y = 2;
        let portable = classes::portable(0);
        let barrier = world.matching_table[&portable];
        world.held = Some(entity(2, portable, 2, 2));
        world.entities = vec![entity(1, barrier, 2, 1)];

        let (_, blocked) = world.step(Action::N);
        assert!(blocked.blocked);
        let (_, opened) = world.step(Action::Open);
        assert!(!opened.action_failed);
        assert_eq!(opened.removed_ids, vec![1]);
        assert!(world.entities.is_empty());
        let (_, moved) = world.step(Action::N);
        assert!(!moved.blocked);
        assert_eq!((moved.x, moved.y), (2, 1));
        assert!(moved.held_id.is_some());
    }

    #[test]
    fn wrong_class_and_empty_hands_open_fail() {
        let mut world = fixture();
        world.self_x = 2;
        world.self_y = 2;
        let portable = classes::portable(0);
        let wrong_barrier = classes::barrier_classes(2)
            .into_iter()
            .find(|class| world.matching_table[&portable] != *class)
            .unwrap();
        world.held = Some(entity(2, portable, 2, 2));
        world.entities = vec![entity(1, wrong_barrier, 2, 1)];
        let (_, wrong) = world.step(Action::Open);
        assert!(wrong.action_failed);
        assert_eq!(world.entities.len(), 1);
        world.held = None;
        let (_, empty) = world.step(Action::Open);
        assert!(empty.action_failed);
        assert_eq!(world.entities.len(), 1);
    }

    #[test]
    fn pickup_own_cell_and_holding_slot_failure() {
        let mut world = fixture();
        world.self_x = 2;
        world.self_y = 2;
        world.entities = vec![entity(1, classes::portable(0), 2, 2)];
        let (_, picked) = world.step(Action::PickUp);
        assert!(!picked.action_failed);
        assert_eq!(picked.held_id, Some(1));
        assert!(world.entities.is_empty());
        world.entities = vec![entity(2, classes::portable(1), 2, 2)];
        let (_, failed) = world.step(Action::PickUp);
        assert!(failed.action_failed);
        assert_eq!(failed.held_id, Some(1));
        assert_eq!(world.entities.len(), 1);
    }

    #[test]
    fn drop_success_and_occupied_cell_failure() {
        let mut world = fixture();
        world.self_x = 2;
        world.self_y = 2;
        world.held = Some(entity(1, classes::portable(0), 0, 0));
        let (_, dropped) = world.step(Action::Drop);
        assert!(!dropped.action_failed);
        assert_eq!(dropped.held_id, None);
        assert_eq!(world.entities[0].x, 2);
        assert_eq!(world.entities[0].y, 2);

        world.held = Some(entity(2, classes::portable(1), 0, 0));
        let (_, failed) = world.step(Action::Drop);
        assert!(failed.action_failed);
        assert_eq!(failed.held_id, Some(2));
        assert_eq!(world.entities.len(), 1);
    }

    #[test]
    fn consumable_arithmetic_and_removal() {
        let mut world = fixture();
        world.energy_bucket = 8;
        world.self_x = 2;
        world.self_y = 2;
        world.entities = vec![entity(1, classes::CONSUMABLE, 3, 2)];
        let (_, trace) = world.step(Action::E);
        assert_eq!(trace.reward_delta, 3);
        assert_eq!(trace.energy_bucket, 11);
        assert_eq!(trace.removed_ids, vec![1]);
        assert!(world.entities.is_empty());
    }

    #[test]
    fn hazard_applies_per_occupied_tick_including_wait() {
        let mut world = fixture();
        world.self_x = 2;
        world.self_y = 2;
        world.entities = vec![entity(1, classes::HAZARD, 2, 2)];
        let (_, first) = world.step(Action::Wait);
        assert_eq!(first.reward_delta, -2);
        assert_eq!(first.energy_bucket, 8);
        assert_eq!(world.entities.len(), 1);
        let (_, second) = world.step(Action::Wait);
        assert_eq!(second.reward_delta, -2);
        assert_eq!(second.energy_bucket, 6);
    }

    #[test]
    fn step_cost_fires_exactly_on_interval_ticks() {
        let mut theta = blank_theta();
        theta.step_cost_interval = 20;
        let mut world = GridWorld::new(theta, 1, 2).unwrap();
        for tick in 1..=40 {
            let (_, trace) = world.step(Action::Wait);
            let expected = if tick == 20 || tick == 40 { -1 } else { 0 };
            assert_eq!(trace.reward_delta, expected, "tick {tick}");
        }
    }

    #[test]
    fn energy_death_exit_and_tick_cap_terminate() {
        let mut death = fixture();
        death.energy_bucket = 2;
        death.entities = vec![entity(1, classes::HAZARD, 2, 2)];
        let (_, trace) = death.step(Action::Wait);
        assert_eq!(trace.termination, Some(TerminationCause::EnergyDeath));

        let mut exit = fixture();
        exit.self_x = 2;
        exit.self_y = 2;
        exit.entities = vec![entity(1, classes::EXIT, 3, 2)];
        let (_, trace) = exit.step(Action::E);
        assert_eq!(trace.termination, Some(TerminationCause::Exit));

        let mut theta = blank_theta();
        theta.step_limit = 2;
        let mut capped = GridWorld::new(theta, 1, 2).unwrap();
        assert_eq!(capped.step(Action::Wait).1.termination, None);
        assert_eq!(
            capped.step(Action::Wait).1.termination,
            Some(TerminationCause::TickCap)
        );
    }

    #[test]
    fn rule_seed_drives_table_and_shape_permutation_map_seed_drives_positions() {
        let theta = WorldTheta::default();
        let left = GridWorld::new(theta.clone(), 10, 1).unwrap();
        let same_rules = GridWorld::new(theta.clone(), 11, 1).unwrap();
        let other_rules = GridWorld::new(theta, 10, 2).unwrap();
        assert_eq!(left.matching_table, same_rules.matching_table);
        assert_eq!(left.class_shape_pairs(), same_rules.class_shape_pairs());
        assert_ne!(left.matching_table, other_rules.matching_table);
        assert_ne!(left.class_shape_pairs(), other_rules.class_shape_pairs());
        let left_positions = left
            .entities
            .iter()
            .map(|entity| (entity.local_id, entity.x, entity.y))
            .collect::<Vec<_>>();
        let same_rule_positions = same_rules
            .entities
            .iter()
            .map(|entity| (entity.local_id, entity.x, entity.y))
            .collect::<Vec<_>>();
        assert_ne!(left_positions, same_rule_positions);
    }
}
