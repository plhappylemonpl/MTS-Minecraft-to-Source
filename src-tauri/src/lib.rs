/**
 * @file lib.rs
 *
 * Main Rust backend entry point.
 * Defines Tauri commands callable from the frontend and manages the application lifecycle.
 */

mod anvil;
mod vmf;

use anvil::{ConversionCoords, get_data_from_map, get_total_chunk_count};
use vmf::{convert_and_write_vmf, VmfBlock};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Emitter;
use tauri_plugin_opener::OpenerExt;
use serde::Serialize;

#[derive(Serialize, Clone)]
struct ProgressUpdate {
    progress: u16,
    label: String,
}

/// A Tauri command invoked by the frontend to start the conversion process.
#[tauri::command]
async fn start_conversion(
    map_path: String,
    vmf_path: String,
    optimize: bool,
    dynamic: bool,
    coords: Option<ConversionCoords>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    println!(
        "Starting conversion with params: map_path={}, vmf_path={}, optimize={}, dynamic={}, coords={:?}",
        map_path, vmf_path, optimize, dynamic, coords
    );

    // --- TASK 1: REAL MAP READING PROCESS ---
    
    // Set overall progress to reading stage (0-25%)
    app.emit("overall_progress", ProgressUpdate {
        progress: 5,
        label: "Reading Minecraft world data...".to_string(),
    }).map_err(|e| e.to_string())?;

    app.emit("stage_progress", ProgressUpdate {
        progress: 0,
        label: "Scanning world files...".to_string(),
    }).map_err(|e| e.to_string())?;

    let total_chunks = get_total_chunk_count(&map_path, dynamic, coords.clone())?;
    println!("Total Chunks to process: {}", total_chunks);

    if total_chunks == 0 {
        app.emit("detail_progress", ProgressUpdate {
            progress: 100,
            label: "No chunks found".to_string(),
        }).map_err(|e| e.to_string())?;
        
        app.emit("stage_progress", ProgressUpdate {
            progress: 100,
            label: "World scanning complete".to_string(),
        }).map_err(|e| e.to_string())?;
    }

    let chunks_processed = Arc::new(Mutex::new(0_usize));
    let progress_reporter = {
        let app_handle = app.clone();
        let chunks_processed = chunks_processed.clone();
        
        move || {
            let mut num = chunks_processed.lock().unwrap();
            *num += 1;

            if total_chunks > 0 {
                let detail_percent = (*num as f32 / total_chunks as f32 * 100.0).round() as u16;
                let stage_percent = (detail_percent as f32 * 0.8).round() as u16; // Stage is 80% when detail is 100%
                
                app_handle.emit("detail_progress", ProgressUpdate {
                    progress: detail_percent,
                    label: format!("Processing chunk {}/{}", *num, total_chunks),
                }).unwrap();
                
                app_handle.emit("stage_progress", ProgressUpdate {
                    progress: stage_percent,
                    label: "Reading world chunks...".to_string(),
                }).unwrap();
            }
        }
    };
    
    let minecraft_blocks = get_data_from_map(&map_path, progress_reporter, dynamic, coords)?;
    println!("[DEBUG] Map reading completed. Found {} blocks", minecraft_blocks.len());

    // Complete reading stage
    app.emit("stage_progress", ProgressUpdate {
        progress: 100,
        label: "World reading complete".to_string(),
    }).map_err(|e| e.to_string())?;

    app.emit("overall_progress", ProgressUpdate {
        progress: 25,
        label: "Converting to VMF format...".to_string(),
    }).map_err(|e| e.to_string())?;

    // --- CONVERT MINECRAFT BLOCKS TO VMF BLOCKS ---
    println!("[DEBUG] Converting Minecraft blocks to VMF blocks...");
    
    app.emit("stage_progress", ProgressUpdate {
        progress: 0,
        label: "Converting block data...".to_string(),
    }).map_err(|e| e.to_string())?;

    let total_blocks = minecraft_blocks.len();
    let vmf_blocks: Vec<VmfBlock> = minecraft_blocks
        .into_iter()
        .enumerate()
        .map(|(index, mb)| {
            // Update progress every 1000 blocks
            if index % 1000 == 0 {
                let detail_percent = ((index as f32 / total_blocks as f32) * 100.0).round() as u16;
                let stage_percent = ((index as f32 / total_blocks as f32) * 50.0).round() as u16; // First half of conversion stage
                
                app.emit("detail_progress", ProgressUpdate {
                    progress: detail_percent,
                    label: format!("Converting block {}/{}", index + 1, total_blocks),
                }).unwrap();
                
                app.emit("stage_progress", ProgressUpdate {
                    progress: stage_percent,
                    label: "Converting Minecraft blocks...".to_string(),
                }).unwrap();
            }
            
            VmfBlock {
                x: mb.x,
                y: mb.y,
                z: mb.z,
                block_type: mb.block_type,
            }
        })
        .collect();
        
    println!("[DEBUG] Block conversion completed. {} VMF blocks ready", vmf_blocks.len());

    app.emit("detail_progress", ProgressUpdate {
        progress: 100,
        label: format!("Converted {} blocks", total_blocks),
    }).map_err(|e| e.to_string())?;

    app.emit("stage_progress", ProgressUpdate {
        progress: 50,
        label: if optimize { 
            "Optimizing VMF structures..." 
        } else { 
            "Creating VMF structures..." 
        }.to_string(),
    }).map_err(|e| e.to_string())?;

    // --- CONVERT MAP DATA TO VMF WITH OPTIMIZED PROCESSING ---
    println!("[DEBUG] Starting VMF generation with optimization mode: {}", optimize);
    
    app.emit("stage_progress", ProgressUpdate {
        progress: 0,
        label: "Preparing VMF generation...".to_string(),
    }).map_err(|e| e.to_string())?;

    app.emit("overall_progress", ProgressUpdate {
        progress: 75,
        label: "Generating VMF file...".to_string(),
    }).map_err(|e| e.to_string())?;

    // Bezpośrednie konwertowanie i zapisywanie z optymalizacją
    convert_and_write_vmf(vmf_blocks, &vmf_path, app.clone(), optimize)?;

    app.emit("stage_progress", ProgressUpdate {
        progress: 100,
        label: "VMF generation complete".to_string(),
    }).map_err(|e| e.to_string())?;

    app.emit("detail_progress", ProgressUpdate {
        progress: 100,
        label: "File saved successfully".to_string(),
    }).map_err(|e| e.to_string())?;

    // --- TASK 2: (OPTIONAL) SIMULATE WORLD OPTIMIZATION PROCESS ---
    // This is now integrated into the main VMF generation process when optimize=true
    if optimize {
        app.emit("overall_progress", ProgressUpdate {
            progress: 95,
            label: "Finalizing optimizations...".to_string(),
        }).map_err(|e| e.to_string())?;

        // Short delay to show the optimization completion
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    app.emit("overall_progress", ProgressUpdate {
        progress: 100,
        label: if optimize {
            "Optimized conversion completed successfully!".to_string()
        } else {
            "Conversion completed successfully!".to_string()
        },
    }).map_err(|e| e.to_string())?;

    app.emit("conversion_complete", &vmf_path)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// A Tauri command that opens the system's file explorer to the directory containing the given file path.
#[tauri::command]
async fn open_file_location(path: String, app: tauri::AppHandle) -> Result<(), String> {
    let file_path = Path::new(&path);
    let parent_dir = file_path.parent().ok_or("Could not get parent directory".to_string())?;

    app.opener()
        .open_path(parent_dir.to_str().unwrap(), None::<&str>)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_conversion,
            open_file_location
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}