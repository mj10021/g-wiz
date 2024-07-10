use bevy::prelude::{Color, KeyCode, MouseButton, Resource};
use serde_json::{from_str, Value};
use std::fs::{read_to_string, File};
use std::io::Write;

#[derive(Resource)]
pub struct Settings {
    pub hole_delete_button: KeyCode,
    pub merge_delete_button: KeyCode,
    pub orbit_mouse_button: MouseButton,
    pub pan_mouse_button: MouseButton,
    pub extrusion_color: Color,
    pub retraction_color: Color,
    pub deretraction_color: Color,
    pub travel_color: Color,
}

fn read_key(settings: &Value, key: &str) -> KeyCode {
    let value = settings.get("keys").unwrap();
    let key = value.get(key).unwrap().as_str();
    match key {
        Some("del") | Some("delete") => KeyCode::Delete,
        Some("backspace") => KeyCode::Backspace,
        _ => panic!("invalid key option"),
    }
}

fn read_mouse_button(settings: &Value, key: &str) -> MouseButton {
    let value = settings.get("buttons").unwrap();
    let button = value.get(key).unwrap().as_str();
    match button {
        Some("right") => MouseButton::Right,
        Some("left") => MouseButton::Left,
        Some("middle") => MouseButton::Middle,
        _ => panic!("invalid mouse button option"),
    }
}

fn read_color(settings: &Value, key: &str) -> Color {
    let value = settings.get("colors").unwrap();
    let color = value.get(key).unwrap().as_str().unwrap();
    Color::hex(color).unwrap()
}

pub fn read_settings() -> Settings {
    let path = std::env::current_exe()
        .expect("could not find excecutable directory")
        .parent()
        .unwrap()
        .join(std::path::PathBuf::from("settings.json"));
    let settings = {
        if path.exists() {
            &read_to_string(&path).unwrap()
        } else {
            let mut default = File::create(&path).expect("msg");
            default.write_all(DEFAULT_SETTINGS.as_bytes()).expect("msg");
            DEFAULT_SETTINGS
        }
    };
    let settings = from_str(settings).expect("failed to parse json");
    Settings {
        hole_delete_button: read_key(&settings, "hole delete"),
        merge_delete_button: read_key(&settings, "merge delete"),
        orbit_mouse_button: read_mouse_button(&settings, "mouse orbit"),
        pan_mouse_button: read_mouse_button(&settings, "mouse pan"),
        extrusion_color: read_color(&settings, "extrusion color"),
        retraction_color: read_color(&settings, "retraction color"),
        deretraction_color: read_color(&settings, "deretraction color"),
        travel_color: read_color(&settings, "travel move color"),
    }
}

const DEFAULT_SETTINGS: &str = r#"{
    "colors" : {
        "extrusion color": "ff0000",
        "retraction color" : "00ff00",
        "deretraction color": "000000",
        "travel move color": "0000ff"
    },
    "keys" : {
        "hole delete": "del",
        "merge delete": "backspace"
   },
    "buttons" : {
        "mouse orbit": "right",
        "mouse pan": "left" 
    }
}"#;

pub const DEFAULT_GCODE: &str = r#"G28
F800
G1 X1 Y1 Z1
G1 X100 Y100 Z0.4
E0.5
G1 X110 E2.0
G1 Y110 E2.0
G1 X100 E2.0
G1 Y100 E2.0
G1 E-1.0"#;

pub const SHAPE_THRESHOLD: f32 = 0.1;
