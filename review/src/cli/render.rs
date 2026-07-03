use crate::world::classes;
use crate::world::grid::{WorldRenderEntity, WorldRenderFrame};
use std::collections::BTreeMap;

pub fn format_frame(frame: &WorldRenderFrame) -> String {
    let mut glyphs = EpisodeGlyphs::from_first_frame(frame);
    format_frame_with_glyphs(frame, &mut glyphs)
}

pub fn format_frame_with_glyphs(frame: &WorldRenderFrame, glyphs: &mut EpisodeGlyphs) -> String {
    let mut cells = vec![vec!['.'; frame.width as usize]; frame.height as usize];
    for entity in &frame.entities {
        if in_frame(frame, entity.x, entity.y) {
            cells[entity.y as usize][entity.x as usize] = glyphs.entity_glyph(entity);
        }
    }
    cells[frame.self_y as usize][frame.self_x as usize] = '@';

    let mut out = String::new();
    out.push_str(&format!(
        "tick={} energy={} delta={} held_shape={} blocked={} failed={} termination={}\n",
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

#[derive(Clone, Debug, Default)]
pub struct EpisodeGlyphs {
    portable_by_shape: BTreeMap<u32, char>,
    barrier_by_shape: BTreeMap<u32, char>,
    next_portable: usize,
    next_barrier: usize,
}

impl EpisodeGlyphs {
    pub fn from_first_frame(frame: &WorldRenderFrame) -> Self {
        let mut glyphs = Self::default();
        let mut portable_shapes = frame
            .entities
            .iter()
            .filter(|entity| classes::is_portable(entity.behavior_class))
            .map(|entity| entity.shape_id)
            .collect::<Vec<_>>();
        if frame.held_behavior_class.is_some_and(classes::is_portable) {
            if let Some(shape) = frame.held_shape_id {
                portable_shapes.push(shape);
            }
        }
        let mut barrier_shapes = frame
            .entities
            .iter()
            .filter(|entity| classes::is_barrier(entity.behavior_class))
            .map(|entity| entity.shape_id)
            .collect::<Vec<_>>();
        portable_shapes.sort_unstable();
        portable_shapes.dedup();
        barrier_shapes.sort_unstable();
        barrier_shapes.dedup();
        for shape in portable_shapes {
            glyphs.portable_glyph(shape);
        }
        for shape in barrier_shapes {
            glyphs.barrier_glyph(shape);
        }
        glyphs
    }

    fn entity_glyph(&mut self, entity: &WorldRenderEntity) -> char {
        match entity.behavior_class {
            0 => '#',
            class if classes::is_portable(class) => self.portable_glyph(entity.shape_id),
            class if classes::is_barrier(class) => self.barrier_glyph(entity.shape_id),
            30 => 'o',
            31 => '~',
            32 => 'X',
            _ => '?',
        }
    }

    fn portable_glyph(&mut self, shape: u32) -> char {
        if let Some(glyph) = self.portable_by_shape.get(&shape) {
            return *glyph;
        }
        let glyph = indexed_char(b'a', self.next_portable);
        self.next_portable += 1;
        self.portable_by_shape.insert(shape, glyph);
        glyph
    }

    fn barrier_glyph(&mut self, shape: u32) -> char {
        if let Some(glyph) = self.barrier_by_shape.get(&shape) {
            return *glyph;
        }
        let glyph = indexed_char(b'A', self.next_barrier);
        self.next_barrier += 1;
        self.barrier_by_shape.insert(shape, glyph);
        glyph
    }
}

fn indexed_char(base: u8, idx: usize) -> char {
    (base + (idx.min(25) as u8)) as char
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::grid::TerminationCause;

    fn frame(entities: Vec<WorldRenderEntity>) -> WorldRenderFrame {
        WorldRenderFrame {
            tick: 1,
            width: 4,
            height: 2,
            self_x: 0,
            self_y: 0,
            energy_bucket: 10,
            reward_delta: 0,
            blocked: false,
            action_failed: false,
            held_shape_id: None,
            held_behavior_class: None,
            termination: Some(TerminationCause::ScriptEnd),
            entities,
        }
    }

    #[test]
    fn barrier_letter_stays_stable_after_removal() {
        let first = frame(vec![
            WorldRenderEntity {
                local_id: 1,
                behavior_class: classes::barrier(0),
                shape_id: 4,
                x: 1,
                y: 0,
            },
            WorldRenderEntity {
                local_id: 2,
                behavior_class: classes::barrier(1),
                shape_id: 9,
                x: 2,
                y: 0,
            },
        ]);
        let second = frame(vec![WorldRenderEntity {
            local_id: 2,
            behavior_class: classes::barrier(1),
            shape_id: 9,
            x: 2,
            y: 0,
        }]);
        let mut glyphs = EpisodeGlyphs::from_first_frame(&first);
        let first_text = format_frame_with_glyphs(&first, &mut glyphs);
        let second_text = format_frame_with_glyphs(&second, &mut glyphs);
        assert!(first_text.lines().nth(1).unwrap().contains("@AB"));
        assert!(second_text.lines().nth(1).unwrap().contains("@.B"));
        assert!(!second_text.lines().nth(1).unwrap().contains("@.A"));
    }
}
