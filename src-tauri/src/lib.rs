pub mod center;
mod curated;

use center::{CenterSnapshot, SkillContent, TranslationInput};

#[tauri::command]
fn scan_center() -> Result<CenterSnapshot, String> {
    center::scan().map_err(|error| error.to_string())
}

#[tauri::command]
fn connect_skill(skill_id: String, tool_id: String) -> Result<CenterSnapshot, String> {
    center::connect(&skill_id, &tool_id).map_err(|error| error.to_string())?;
    scan_center()
}

#[tauri::command]
fn disconnect_skill(skill_id: String, tool_id: String) -> Result<CenterSnapshot, String> {
    center::disconnect(&skill_id, &tool_id).map_err(|error| error.to_string())?;
    scan_center()
}

#[tauri::command]
fn read_skill(skill_id: String) -> Result<SkillContent, String> {
    center::read_skill(&skill_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn save_translation(input: TranslationInput) -> Result<CenterSnapshot, String> {
    center::save_translation(input).map_err(|error| error.to_string())?;
    scan_center()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_center,
            connect_skill,
            disconnect_skill,
            read_skill,
            save_translation
        ])
        .run(tauri::generate_context!())
        .expect("failed to start BaoCanMou AI Skill Center");
}
