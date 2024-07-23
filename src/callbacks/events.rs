use bevy::prelude::Event;
#[derive(Event)]
pub enum UiEvent {
    ForceRefresh,
    Undo,
    Redo,
    ExportDialogue, // select/deselect??
    MoveDisplay(bool, bool, f32),
}
#[derive(Event)]
pub enum CommandEvent {
    MergeDelete,
    HoleDelete,
    Subdivide,
    RecalcBounds,
    Draw,
}
