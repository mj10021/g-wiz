use bevy::prelude::*;

#[derive(Event)]
pub enum UiEvent {
    ForceRefresh,
    MoveDisplay(bool, bool, f32),
    SetSelections,
}
#[derive(Event)]
pub enum CommandEvent {
    MergeDelete,
    HoleDelete,
    Subdivide,
    RecalcBounds,
    InsertPoint,
    Translate(Vec3),
    Rotate(Vec3),
    Scale(Vec3),
    Undo,
    Redo
}

#[derive(Event)]
pub enum FileEvent {
    Open,
    Save,
    SaveAs,
}