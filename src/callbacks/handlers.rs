use bevy::prelude::{Event, EventReader};
use super::events::*;
pub enum Command<T: Default + bevy::prelude::Resource + Copy + Sized> {
    // t is the actual callback
    MergeDelete(T),
    HoleDelete(T),
    Subdivide(T),
    RecalcBounds(T),
    Undo(T),
    Redo(T),

}

fn handle<E>(mut event: EventReader<E>) where E: Event {
    for event in event.read() {
        match *event {
            UiEvent::ForceRefresh => {
                // do something
            }
            UiEvent::Undo => {
                // do something
            }
            UiEvent::Redo => {
                // do something
            }
            CommandEvent::MergeDelete => {
                // do something
            }
            CommandEvent::HoleDelete => {
                // do something
            }
            CommandEvent::Subdivide => {
                // do something
            }
            CommandEvent::RecalcBounds => {
                // do something
            }
            CommandEvent::Draw => {
                // do something
            }
        }
    }
}