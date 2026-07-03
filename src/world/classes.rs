use crate::world::rng::SplitMix64;
use std::collections::BTreeMap;

pub const WALL: u32 = 0;
pub const PORTABLE_BASE: u32 = 10;
pub const BARRIER_BASE: u32 = 20;
pub const CONSUMABLE: u32 = 30;
pub const HAZARD: u32 = 31;
pub const EXIT: u32 = 32;

pub fn portable(index: u32) -> u32 {
    PORTABLE_BASE + index
}

pub fn barrier(index: u32) -> u32 {
    BARRIER_BASE + index
}

pub fn portable_classes(count: u32) -> Vec<u32> {
    (0..count).map(portable).collect()
}

pub fn barrier_classes(count: u32) -> Vec<u32> {
    (0..count).map(barrier).collect()
}

pub fn all_classes(portable_count: u32, barrier_count: u32) -> Vec<u32> {
    let mut out = vec![WALL];
    out.extend(portable_classes(portable_count));
    out.extend(barrier_classes(barrier_count));
    out.extend([CONSUMABLE, HAZARD, EXIT]);
    out.sort_unstable();
    out.dedup();
    out
}

pub fn class_shape_map(
    rule_seed: u64,
    portable_count: u32,
    barrier_count: u32,
) -> BTreeMap<u32, u32> {
    let classes = all_classes(portable_count, barrier_count);
    let mut shapes = (1..=classes.len() as u32).collect::<Vec<_>>();
    let mut rng = SplitMix64::new(rule_seed ^ 0xA51C_0A55_51A9_EC0D);
    shuffle(&mut shapes, &mut rng);
    if !shapes.is_empty() {
        let offset = rule_seed as usize % shapes.len();
        shapes.rotate_left(offset);
    }
    classes.into_iter().zip(shapes).collect()
}

pub fn matching_table(
    rule_seed: u64,
    portable_count: u32,
    barrier_count: u32,
) -> BTreeMap<u32, u32> {
    let portables = portable_classes(portable_count);
    let mut barriers = barrier_classes(barrier_count);
    let mut rng = SplitMix64::new(rule_seed ^ 0xB422_1E2A_7A61_0F13);
    shuffle(&mut barriers, &mut rng);
    if !barriers.is_empty() {
        let offset = rule_seed as usize % barriers.len();
        barriers.rotate_left(offset);
    }
    portables.into_iter().zip(barriers).collect()
}

pub fn is_portable(class: u32) -> bool {
    (PORTABLE_BASE..BARRIER_BASE).contains(&class)
}

pub fn is_barrier(class: u32) -> bool {
    (BARRIER_BASE..CONSUMABLE).contains(&class)
}

pub fn is_blocking(class: u32) -> bool {
    class == WALL || is_barrier(class)
}

fn shuffle<T>(items: &mut [T], rng: &mut SplitMix64) {
    for index in (1..items.len()).rev() {
        let swap = rng.next_range((index + 1) as u64) as usize;
        items.swap(index, swap);
    }
}
