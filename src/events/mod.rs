pub mod console;
pub mod handlers;



use console::*;

use bevy::prelude::*;
#[derive(Event)]
pub enum UiEvent {
    MergeDelete,
    HoleDelete,
    MoveDisplay(bool, bool, f32),
    SelectAll,
    Undo,
    Redo,
    SetPanOrbit(bool),
}
#[derive(Clone, Debug, Event)]
pub enum CommandEvent {
    Translate(Translate),
    Rotate(Rotate),
    Scale(Scale),
    Subdivide(Subdivide),
    // Draw(Draw),
    // Filter(Filter),
    // Map(Map),
}
impl CommandEvent {
    fn inner_mut(&mut self) -> &mut dyn console::Param {
        match self {
            Self::Translate(translate) => translate,
            Self::Rotate(rotate) => rotate,
            Self::Scale(scale) => scale,
            Self::Subdivide(subdivide) => subdivide,
            // Self::Draw(draw) => draw,
            // Self::Filter(filter) => filter,
            // Self::Map(map) => map,
        }
    }
}
#[derive(Event)]
pub enum SystemEvent {
    SaveAs,
    RecalcBounds,
    ForceRefresh,
}
