use crate::world::classes;
use crate::world::grid::{WorldRenderEntity, WorldRenderFrame};
use std::collections::BTreeMap;

pub fn format_frame(frame: &WorldRenderFrame) -> String {
    let glyphs = Glyphs::new(
        &frame.entities,
        frame.held_behavior_class,
        frame.held_shape_id,
    );
    let mut cells = vec![vec!['.'; frame.width as usize]; frame.height as usize];
    for entity in &frame.entities {
        if in_frame(frame, entity.x, entity.y) {
            cells[entity.y as usize][entity.x as usize] = glyphs.entity_glyph(entity);
        }
    }
    cells[frame.self_y as usize][frame.self_x as usize] = '@';

    let mut out = String::new();
    out.push_str(&format!(
        "tick={} energy={} delta={} held={} blocked={} failed={} termination={}\n",
        frame.tick,
        frame.energy_bucket,
        frame.reward_delta,
        frame
            .held_shape_id
            .map(|shape| shape.to_string())
            .unwrap_or_else(|| "None".to_string()),
        frame.blocked,
        frame.action_failed,
        frame
            .termination
            .map(|cause| cause.to_string())
            .unwrap_or_else(|| "None".to_string())
    ));
    for row in cells {
        for cell in row {
            out.push(cell);
        }
        out.push('\n');
    }
    out
}

fn in_frame(frame: &WorldRenderFrame, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < frame.width && y < frame.height
}

struct Glyphs {
    portable_by_shape: BTreeMap<u32, char>,
    barrier_by_shape: BTreeMap<u32, char>,
}

impl Glyphs {
    fn new(
        entities: &[WorldRenderEntity],
        held_behavior_class: Option<u32>,
        held_shape_id: Option<u32>,
    ) -> Self {
        let mut portable_shapes = entities
            .iter()
            .filter(|entity| classes::is_portable(entity.behavior_class))
            .map(|entity| entity.shape_id)
            .collect::<Vec<_>>();
        if held_behavior_class.is_some_and(classes::is_portable) {
            if let Some(shape) = held_shape_id {
                portable_shapes.push(shape);
            }
        }
        let mut barrier_shapes = entities
            .iter()
            .filter(|entity| classes::is_barrier(entity.behavior_class))
            .map(|entity| entity.shape_id)
            .collect::<Vec<_>>();
        portable_shapes.sort_unstable();
        portable_shapes.dedup();
        barrier_shapes.sort_unstable();
        barrier_shapes.dedup();
        Self {
            portable_by_shape: portable_shapes
                .into_iter()
                .enumerate()
                .map(|(idx, shape)| (shape, indexed_char(b'a', idx)))
                .collect(),
            barrier_by_shape: barrier_shapes
                .into_iter()
                .enumerate()
                .map(|(idx, shape)| (shape, indexed_char(b'A', idx)))
                .collect(),
        }
    }

    fn entity_glyph(&self, entity: &WorldRenderEntity) -> char {
        match entity.behavior_class {
            0 => '#',
            class if classes::is_portable(class) => self
                .portable_by_shape
                .get(&entity.shape_id)
                .copied()
                .unwrap_or('a'),
            class if classes::is_barrier(class) => self
                .barrier_by_shape
                .get(&entity.shape_id)
                .copied()
                .unwrap_or('A'),
            30 => 'o',
            31 => '~',
            32 => 'X',
            _ => '?',
        }
    }
}

fn indexed_char(base: u8, idx: usize) -> char {
    (base + (idx.min(25) as u8)) as char
}
