use crate::core::event::Event;

pub fn render_event(event: &Event) -> String {
    format!("event#{}", event.id)
}
