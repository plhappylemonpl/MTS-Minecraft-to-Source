/**
 * @file main.rs
 *
 * Main Rust backend entry point.
 * Defines Tauri commands callable from the frontend and manages the application lifecycle.
 */

use std::path::Path;
use std::time::Duration;
use tauri::Emitter;
// `OpenerExt` provides the `.opener()` method on the AppHandle.
use tauri_plugin_opener::OpenerExt;

/// A Tauri command invoked by the frontend to start the conversion process.
/// This function simulates a sequential process: Reading -> Optimizing (optional) -> Creating.
#[tauri::command]
async fn start_conversion(map_path: String, vmf_path: String, optimize: bool, app: tauri::AppHandle) -> Result<(), String> {
    println!("Starting conversion with params: map_path={}, vmf_path={}, optimize={}", map_path, vmf_path, optimize);

    // --- TASK 1: SIMULATE MAP READING PROCESS ---
    for i in 0..=100 {
        if let Err(e) = app.emit("reading_progress", i) {
            eprintln!("Failed to emit reading_progress event: {}", e);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // --- TASK 2: (OPTIONAL) SIMULATE OPTIMIZATION PROCESS ---
    if optimize {
        for i in 0..=100 {
            // Emit a new event for the optimization progress.
            if let Err(e) = app.emit("optimization_progress", i) {
                eprintln!("Failed to emit optimization_progress event: {}", e);
            }
            // Give it a different timing to feel distinct.
            tokio::time::sleep(Duration::from_millis(75)).await;
        }
    }

    // --- TASK 3: SIMULATE VMF CREATION PROCESS ---
    for i in 0..=100 {
        if let Err(e) = app.emit("creating_progress", i) {
            eprintln!("Failed to emit creating_progress event: {}", e);
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }

    // Signal to the frontend that the entire process has finished, sending the VMF path as a payload.
    app.emit("conversion_complete", &vmf_path)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// A Tauri command that opens the system's file explorer to the directory containing the given file path.
///
/// # Arguments
/// * `path` - The full path to the file whose location should be opened.
/// * `app` - The application handle, used to access the opener plugin.
#[tauri::command]
async fn open_file_location(path: String, app: tauri::AppHandle) -> Result<(), String> {
    let file_path = Path::new(&path);
    // Get the parent directory of the file.
    let parent_dir = file_path.parent().ok_or("Could not get parent directory".to_string())?;

    // Use the `opener` plugin to open the directory path in the default file manager.
    app.opener()
        .open_path(parent_dir.to_str().unwrap(), None::<&str>)
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Note: The `open_url` method in your original file might be a typo or from an older version.
// The modern `tauri-plugin-opener` uses `.open()` for both URLs and file paths.
// If `.open()` does not work, you may need to check the exact version of the plugin you are using.
// The provided code uses `.open()` as it is the current standard.

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        // Register all commands so they can be called from JavaScript.
        .invoke_handler(tauri::generate_handler![
            start_conversion,
            open_file_location
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}