use std::io::Write;
use std::collections::HashMap;
use tauri::Emitter;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Clone)]
struct ProgressUpdate {
    progress: u16,
    label: String,
}

#[derive(Debug, Clone)]
pub struct VmfBlock {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_type: String,
}

#[derive(Debug, Clone)]
pub struct VmfSide {
    pub id: u32,
    pub plane: String,
    pub material: String,
    pub uaxis: String,
    pub vaxis: String,
    pub rotation: String,
    pub lightmapscale: String,
    pub smoothing_groups: String,
}

#[derive(Debug, Clone)]
pub struct VmfSolid {
    pub id: u32,
    pub sides: Vec<VmfSide>,
}

// Struktura dla konfiguracji materiału z obsługą properties
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum MaterialConfig {
    Simple(String),
    Complex {
        top: Option<String>,
        side: Option<String>,
        bottom: Option<String>,
        properties: Option<HashMap<String, HashMap<String, MaterialOverride>>>,
    },
}

// Struktura dla nadpisywania materiałów na podstawie properties
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaterialOverride {
    top: Option<String>,
    side: Option<String>, 
    bottom: Option<String>,
}

// Enum dla stron bloku
#[derive(Debug, Clone, Copy)]
enum BlockSide {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

static mut CURRENT_ID: u32 = 1;

fn get_next_id() -> u32 {
    unsafe {
        let id = CURRENT_ID;
        CURRENT_ID += 1;
        id
    }
}

fn get_available_memory() -> usize {
    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use winapi::um::sysinfoapi::{MEMORYSTATUSEX, GlobalMemoryStatusEx};
        
        unsafe {
            let mut mem_status = MEMORYSTATUSEX {
                dwLength: mem::size_of::<MEMORYSTATUSEX>() as u32,
                ..mem::zeroed()
            };
            
            if GlobalMemoryStatusEx(&mut mem_status) != 0 {
                ((mem_status.ullAvailPhys as f64) * 0.3) as usize
            } else {
                1_000_000_000
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        1_000_000_000
    }
}

fn calculate_batch_size() -> usize {
    let available_memory = get_available_memory();
    const ESTIMATED_SOLID_SIZE: usize = 2048;
    let max_solids = available_memory / ESTIMATED_SOLID_SIZE;
    max_solids.max(1000).min(20000)
}

// Funkcja parsująca properties z nazwy bloku (Minecraft 1.13+ format)
fn parse_block_properties(block_name: &str) -> (String, HashMap<String, String>) {
    if let Some(bracket_pos) = block_name.find('[') {
        let base_name = &block_name[..bracket_pos];
        let properties_str = &block_name[bracket_pos+1..];
        
        if let Some(end_bracket) = properties_str.rfind(']') {
            let properties_content = &properties_str[..end_bracket];
            let mut properties = HashMap::new();
            
            for property in properties_content.split(',') {
                if let Some(eq_pos) = property.find('=') {
                    let key = property[..eq_pos].trim().to_string();
                    let value = property[eq_pos+1..].trim().to_string();
                    properties.insert(key, value);
                }
            }
            
            return (base_name.to_string(), properties);
        }
    }
    
    // Brak properties - zwróć tylko nazwę bloku
    (block_name.to_string(), HashMap::new())
}

// Ładowanie konfiguracji materiałów
fn load_material_config() -> HashMap<String, MaterialConfig> {
    let config_content = include_str!("../material_config.json");
    
    match serde_json::from_str::<HashMap<String, serde_json::Value>>(config_content) {
        Ok(raw_config) => {
            let mut config = HashMap::new();
            
            for (key, value) in raw_config {
                // Pomijamy komentarze (klucze zaczynające się od "//")
                if key.starts_with("//") {
                    continue;
                }
                
                match value {
                    serde_json::Value::String(simple) => {
                        config.insert(key, MaterialConfig::Simple(simple));
                    }
                    serde_json::Value::Object(obj) => {
                        let top = obj.get("top").and_then(|v| v.as_str()).map(String::from);
                        let side = obj.get("side").and_then(|v| v.as_str()).map(String::from);
                        let bottom = obj.get("bottom").and_then(|v| v.as_str()).map(String::from);
                        
                        // Parsowanie properties
                        let properties = obj.get("properties")
                            .and_then(|v| v.as_object())
                            .map(|props_obj| {
                                let mut properties_map = HashMap::new();
                                
                                for (prop_name, prop_values) in props_obj {
                                    if let Some(values_obj) = prop_values.as_object() {
                                        let mut values_map = HashMap::new();
                                        
                                        for (value_name, material_override) in values_obj {
                                            if let Some(override_obj) = material_override.as_object() {
                                                let override_top = override_obj.get("top")
                                                    .and_then(|v| v.as_str()).map(String::from);
                                                let override_side = override_obj.get("side")
                                                    .and_then(|v| v.as_str()).map(String::from);
                                                let override_bottom = override_obj.get("bottom")
                                                    .and_then(|v| v.as_str()).map(String::from);
                                                
                                                values_map.insert(value_name.clone(), MaterialOverride {
                                                    top: override_top,
                                                    side: override_side,
                                                    bottom: override_bottom,
                                                });
                                            }
                                        }
                                        
                                        properties_map.insert(prop_name.clone(), values_map);
                                    }
                                }
                                
                                properties_map
                            });
                        
                        config.insert(key, MaterialConfig::Complex { top, side, bottom, properties });
                    }
                    _ => {} // Ignorujemy inne typy
                }
            }
            
            config
        }
        Err(e) => {
            println!("[WARNING] Failed to parse material_config.json: {}. Using default materials.", e);
            HashMap::new()
        }
    }
}

fn get_material_for_side(
    block_name: &str, 
    side: BlockSide, 
    config: &HashMap<String, MaterialConfig>
) -> String {
    const MATERIAL_PREFIX: &str = "mc_1.21.4/";
    
    // Parsuj block name i properties
    let (base_name, properties) = parse_block_properties(block_name);
    
    // Sprawdź czy blok ma specjalną konfigurację
    if let Some(material_config) = config.get(&base_name) {
        match material_config {
            MaterialConfig::Simple(texture) => {
                return format!("{}{}", MATERIAL_PREFIX, texture);
            }
            MaterialConfig::Complex { top, side: side_texture, bottom, properties: config_properties } => {
                // Domyślne tekstury dla tego bloku
                let mut final_top = top.as_ref();
                let mut final_side = side_texture.as_ref();
                let mut final_bottom = bottom.as_ref();
                
                // Sprawdź properties i nadpisz tekstury jeśli potrzeba
                if let Some(config_props) = config_properties {
                    for (prop_name, prop_value) in &properties {
                        if let Some(prop_config) = config_props.get(prop_name) {
                            if let Some(override_config) = prop_config.get(prop_value) {
                                // Nadpisz tekstury jeśli są zdefiniowane w override
                                if override_config.top.is_some() {
                                    final_top = override_config.top.as_ref();
                                }
                                if override_config.side.is_some() {
                                    final_side = override_config.side.as_ref();
                                }
                                if override_config.bottom.is_some() {
                                    final_bottom = override_config.bottom.as_ref();
                                }
                            }
                        }
                    }
                }
                
                // Wybierz odpowiednią teksturę dla strony
                let texture = match side {
                    BlockSide::Top => {
                        final_top.or(final_side)
                    }
                    BlockSide::Bottom => {
                        final_bottom.or(final_side)
                    }
                    BlockSide::North | BlockSide::South | BlockSide::East | BlockSide::West => {
                        final_side
                    }
                };
                
                if let Some(tex) = texture {
                    return format!("{}{}", MATERIAL_PREFIX, tex);
                }
            }
        }
    }
    
    // Domyślna logika - usuń "minecraft:" i użyj nazwy bloku jako tekstury na wszystkich stronach
    let clean_name = if let Some(stripped) = base_name.strip_prefix("minecraft:") {
        stripped
    } else {
        &base_name
    };
    
    format!("{}{}", MATERIAL_PREFIX, clean_name)
}

// POPRAWIONA funkcja tworzenia kostki z obsługą różnych materiałów i automatycznym dopasowaniem tekstur
fn create_cube_solid(block: &VmfBlock, block_size: i32, config: &HashMap<String, MaterialConfig>) -> VmfSolid {
    let solid_id = get_next_id();
    
    // Pobierz materiały dla każdej strony
    let top_material = get_material_for_side(&block.block_type, BlockSide::Top, config);
    let bottom_material = get_material_for_side(&block.block_type, BlockSide::Bottom, config);
    let north_material = get_material_for_side(&block.block_type, BlockSide::North, config);
    let south_material = get_material_for_side(&block.block_type, BlockSide::South, config);
    let east_material = get_material_for_side(&block.block_type, BlockSide::East, config);
    let west_material = get_material_for_side(&block.block_type, BlockSide::West, config);
    
    // Konwersja współrzędnych: Minecraft (Y-up) na Source (Z-up)
    let x1 = block.x * block_size;
    let x2 = x1 + block_size;
    let y1 = block.z * block_size; // Oś Z z Minecrafta staje się osią Y w VMF
    let y2 = y1 + block_size;
    let z1 = block.y * block_size; // Oś Y z Minecrafta (wysokość) staje się osią Z w VMF
    let z2 = z1 + block_size;

    let sides = vec![
        // Górna płaszczyzna (+Z) - Fit na całej powierzchni
        VmfSide {
            id: get_next_id(),
            plane: format!("({} {} {}) ({} {} {}) ({} {} {})", x1, y2, z2, x2, y2, z2, x2, y1, z2),
            material: top_material,
            uaxis: format!("[1 0 0 0] {}", block_size as f32 / 256.0),
            vaxis: format!("[0 -1 0 0] {}", block_size as f32 / 256.0),
            rotation: "0".to_string(),
            lightmapscale: "16".to_string(),
            smoothing_groups: "0".to_string(),
        },
        
        // Dolna płaszczyzna (-Z) - Fit na całej powierzchni
        VmfSide {
            id: get_next_id(),
            plane: format!("({} {} {}) ({} {} {}) ({} {} {})", x1, y1, z1, x2, y1, z1, x2, y2, z1),
            material: bottom_material,
            uaxis: format!("[1 0 0 0] {}", block_size as f32 / 256.0),
            vaxis: format!("[0 -1 0 0] {}", block_size as f32 / 256.0),
            rotation: "0".to_string(),
            lightmapscale: "16".to_string(),
            smoothing_groups: "0".to_string(),
        },

        // Północna płaszczyzna (+Y) - Fit na całej powierzchni
        VmfSide {
            id: get_next_id(),
            plane: format!("({} {} {}) ({} {} {}) ({} {} {})", x1, y2, z2, x1, y2, z1, x2, y2, z1),
            material: north_material,
            uaxis: format!("[1 0 0 0] {}", block_size as f32 / 256.0),
            vaxis: format!("[0 0 -1 0] {}", block_size as f32 / 256.0),
            rotation: "0".to_string(),
            lightmapscale: "16".to_string(),
            smoothing_groups: "0".to_string(),
        },

        // Południowa płaszczyzna (-Y) - Fit na całej powierzchni
        VmfSide {
            id: get_next_id(),
            plane: format!("({} {} {}) ({} {} {}) ({} {} {})", x1, y1, z1, x1, y1, z2, x2, y1, z2),
            material: south_material,
            uaxis: format!("[-1 0 0 0] {}", block_size as f32 / 256.0),
            vaxis: format!("[0 0 -1 0] {}", block_size as f32 / 256.0),
            rotation: "0".to_string(),
            lightmapscale: "16".to_string(),
            smoothing_groups: "0".to_string(),
        },

        // Wschodnia płaszczyzna (+X) - Fit na całej powierzchni
        VmfSide {
            id: get_next_id(),
            plane: format!("({} {} {}) ({} {} {}) ({} {} {})", x2, y2, z2, x2, y2, z1, x2, y1, z1),
            material: east_material,
            uaxis: format!("[0 1 0 0] {}", block_size as f32 / 256.0),
            vaxis: format!("[0 0 -1 0] {}", block_size as f32 / 256.0),
            rotation: "0".to_string(),
            lightmapscale: "16".to_string(),
            smoothing_groups: "0".to_string(),
        },

        // Zachodnia płaszczyzna (-X) - Fit na całej powierzchni
        VmfSide {
            id: get_next_id(),
            plane: format!("({} {} {}) ({} {} {}) ({} {} {})", x1, y1, z1, x1, y2, z1, x1, y2, z2),
            material: west_material,
            uaxis: format!("[0 -1 0 0] {}", block_size as f32 / 256.0),
            vaxis: format!("[0 0 -1 0] {}", block_size as f32 / 256.0),
            rotation: "0".to_string(),
            lightmapscale: "16".to_string(),
            smoothing_groups: "0".to_string(),
        },
    ];

    VmfSolid {
        id: solid_id,
        sides,
    }
}

fn write_vmf_header(file: &mut std::fs::File) -> Result<(), String> {
    writeln!(file, "// VMF File Generated by MTS").map_err(|e| e.to_string())?;
    writeln!(file, "versioninfo").map_err(|e| e.to_string())?;
    writeln!(file, "{{").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"editorversion\" \"400\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"editorbuild\" \"8867\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"mapversion\" \"1\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"formatversion\" \"100\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"prefab\" \"0\"").map_err(|e| e.to_string())?;
    writeln!(file, "}}").map_err(|e| e.to_string())?;
    writeln!(file, "visgroups").map_err(|e| e.to_string())?;
    writeln!(file, "{{").map_err(|e| e.to_string())?;
    writeln!(file, "}}").map_err(|e| e.to_string())?;
    writeln!(file, "viewsettings").map_err(|e| e.to_string())?;
    writeln!(file, "{{").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"bSnapToGrid\" \"1\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"bShowGrid\" \"1\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"bShowLogicalGrid\" \"0\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"nGridSpacing\" \"64\"").map_err(|e| e.to_string())?;
    writeln!(file, "}}").map_err(|e| e.to_string())?;
    writeln!(file, "world").map_err(|e| e.to_string())?;
    writeln!(file, "{{").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"id\" \"1\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"mapversion\" \"1\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"classname\" \"worldspawn\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"skyname\" \"sky_day01_01\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"maxpropscreenwidth\" \"-1\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"detailvbsp\" \"detail.vbsp\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"detailmaterial\" \"detail/detailsprites\"").map_err(|e| e.to_string())?;
    Ok(())
}

fn write_vmf_footer(file: &mut std::fs::File) -> Result<(), String> {
    writeln!(file, "}}").map_err(|e| e.to_string())?;
    writeln!(file, "entity").map_err(|e| e.to_string())?;
    writeln!(file, "{{").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"id\" \"2\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"classname\" \"info_player_start\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"angles\" \"0 0 0\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\"origin\" \"0 0 100\"").map_err(|e| e.to_string())?;
    writeln!(file, "\teditor").map_err(|e| e.to_string())?;
    writeln!(file, "\t{{").map_err(|e| e.to_string())?;
    writeln!(file, "\t\t\"color\" \"0 255 0\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\t\"visgroupshown\" \"1\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\t\"visgroupautoshown\" \"1\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\t\"logicalpos\" \"[0 0]\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t}}").map_err(|e| e.to_string())?;
    writeln!(file, "}}").map_err(|e| e.to_string())?;
    Ok(())
}

fn write_solid_to_file(file: &mut std::fs::File, solid: &VmfSolid) -> Result<(), String> {
    writeln!(file, "\tsolid").map_err(|e| e.to_string())?;
    writeln!(file, "\t{{").map_err(|e| e.to_string())?;
    writeln!(file, "\t\t\"id\" \"{}\"", solid.id).map_err(|e| e.to_string())?;

    for side in &solid.sides {
        writeln!(file, "\t\tside").map_err(|e| e.to_string())?;
        writeln!(file, "\t\t{{").map_err(|e| e.to_string())?;
        writeln!(file, "\t\t\t\"id\" \"{}\"", side.id).map_err(|e| e.to_string())?;
        writeln!(file, "\t\t\t\"plane\" \"{}\"", side.plane).map_err(|e| e.to_string())?;
        writeln!(file, "\t\t\t\"material\" \"{}\"", side.material).map_err(|e| e.to_string())?;
        writeln!(file, "\t\t\t\"uaxis\" \"{}\"", side.uaxis).map_err(|e| e.to_string())?;
        writeln!(file, "\t\t\t\"vaxis\" \"{}\"", side.vaxis).map_err(|e| e.to_string())?;
        writeln!(file, "\t\t\t\"rotation\" \"{}\"", side.rotation).map_err(|e| e.to_string())?;
        writeln!(file, "\t\t\t\"lightmapscale\" \"{}\"", side.lightmapscale).map_err(|e| e.to_string())?;
        writeln!(file, "\t\t\t\"smoothing_groups\" \"{}\"", side.smoothing_groups).map_err(|e| e.to_string())?;
        writeln!(file, "\t\t}}").map_err(|e| e.to_string())?;
    }

    writeln!(file, "\t\teditor").map_err(|e| e.to_string())?;
    writeln!(file, "\t\t{{").map_err(|e| e.to_string())?;
    writeln!(file, "\t\t\t\"color\" \"0 180 0\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\t\t\"visgroupshown\" \"1\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\t\t\"visgroupautoshown\" \"1\"").map_err(|e| e.to_string())?;
    writeln!(file, "\t\t}}").map_err(|e| e.to_string())?;
    writeln!(file, "\t}}").map_err(|e| e.to_string())?;
    Ok(())
}

// Główna funkcja konwersji z obsługą konfiguracji materiałów i properties
pub fn convert_and_write_vmf(
    blocks: Vec<VmfBlock>, 
    vmf_path: &str,
    app: tauri::AppHandle,
    optimize: bool,
) -> Result<(), String> {
    let total_blocks = blocks.len();
    let block_size = 40;
    
    println!("[DEBUG] Starting VMF conversion with {} blocks, optimization: {}", total_blocks, optimize);
    
    // Załaduj konfigurację materiałów
    let material_config = load_material_config();
    println!("[DEBUG] Loaded material config for {} blocks", material_config.len());
    
    // Debug - sprawdź czy oak_log jest w konfiguracji
    if material_config.contains_key("minecraft:oak_log") {
        println!("[DEBUG] oak_log found in config");
    } else {
        println!("[DEBUG] oak_log NOT found in config");
    }
    
    // Debug - wypisz kilka pierwszych kluczy
    let mut count = 0;
    for key in material_config.keys() {
        if count < 5 {
            println!("[DEBUG] Config key: {}", key);
            count += 1;
        }
    }
    
    // Utwórz plik i napisz header
    let mut file = std::fs::File::create(vmf_path).map_err(|e| e.to_string())?;
    write_vmf_header(&mut file)?;
    
    let _ = app.emit("detail_progress", ProgressUpdate {
        progress: 0,
        label: "Starting VMF generation...".to_string(),
    });
    
    if optimize {
        // PROSTSZA OPTYMALIZACJA - tylko usuwanie duplikatów
        println!("[DEBUG] Using optimized mode - removing duplicates");
        
        let _ = app.emit("detail_progress", ProgressUpdate {
            progress: 10,
            label: "Removing duplicate blocks...".to_string(),
        });
        
        // Usuń duplikaty bloków (te same pozycje)
        let mut unique_blocks: std::collections::HashMap<(i32, i32, i32), VmfBlock> = std::collections::HashMap::new();
        for block in blocks {
            let key = (block.x, block.y, block.z);
            unique_blocks.insert(key, block);
        }
        
        let deduplicated_blocks: Vec<VmfBlock> = unique_blocks.into_values().collect();
        let unique_count = deduplicated_blocks.len();
        
        println!("[DEBUG] Removed {} duplicate blocks, {} unique blocks remain", 
            total_blocks - unique_count, unique_count);
        
        // Przetwórz bloki w batch-ach
        let batch_size = calculate_batch_size();
        let mut processed_blocks = 0;
        
        for (batch_idx, chunk) in deduplicated_blocks.chunks(batch_size).enumerate() {
            let batch_start = processed_blocks;
            
            let _ = app.emit("detail_progress", ProgressUpdate {
                progress: (10 + ((batch_start * 80) / unique_count)) as u16,
                label: format!("Processing optimized batch {}: blocks {}-{}", 
                    batch_idx + 1, batch_start, batch_start + chunk.len()),
            });
            
            // Twórz solidy dla tego batch-a
            let mut solids = Vec::with_capacity(chunk.len());
            for block in chunk {
                solids.push(create_cube_solid(block, block_size, &material_config));
            }
            
            // Zapisz solidy bezpośrednio do pliku
            for solid in &solids {
                write_solid_to_file(&mut file, solid)?;
            }
            
            processed_blocks += chunk.len();
            
            // Wyczyść pamięć
            drop(solids);
            
            // Flush co kilka batch-ów
            if batch_idx % 10 == 0 {
                file.flush().map_err(|e| e.to_string())?;
            }
            
            println!("[DEBUG] Optimized batch {}, processed {}/{} unique blocks", 
                batch_idx + 1, processed_blocks, unique_count);
        }
        
        let _ = app.emit("detail_progress", ProgressUpdate {
            progress: 90,
            label: format!("Optimization complete: {} unique solids created", unique_count),
        });
        
        println!("[DEBUG] Optimization complete: {} unique solids from {} total blocks (duplicates removed: {})", 
            unique_count, total_blocks, total_blocks - unique_count);
            
    } else {
        // STANDARDOWY TRYB - każdy blok = jeden solid
        println!("[DEBUG] Using standard mode - individual blocks");
        
        let batch_size = calculate_batch_size();
        let mut processed_blocks = 0;
        
        for (batch_idx, chunk) in blocks.chunks(batch_size).enumerate() {
            let batch_start = batch_idx * batch_size;
            
            let _ = app.emit("detail_progress", ProgressUpdate {
                progress: ((batch_start as f32 / total_blocks as f32) * 90.0) as u16,
                label: format!("Processing standard batch {}: blocks {}-{}", 
                    batch_idx + 1, batch_start, batch_start + chunk.len()),
            });
            
            let mut solids = Vec::with_capacity(chunk.len());
            for block in chunk {
                solids.push(create_cube_solid(block, block_size, &material_config));
            }
            
            // Zapisz solidy bezpośrednio do pliku
            for solid in &solids {
                write_solid_to_file(&mut file, solid)?;
            }
            
            processed_blocks += chunk.len();
            
            // Wyczyść pamięć
            drop(solids);
            
            // Flush co kilka batch-ów
            if batch_idx % 10 == 0 {
                file.flush().map_err(|e| e.to_string())?;
            }
            
            println!("[DEBUG] Processed standard batch {}/{}, total blocks: {}/{}", 
                batch_idx + 1, 
                (total_blocks + batch_size - 1) / batch_size,
                processed_blocks, 
                total_blocks
            );
        }
    }
    
    // Napisz footer i zamknij plik
    write_vmf_footer(&mut file)?;
    file.flush().map_err(|e| e.to_string())?;
    
    let _ = app.emit("detail_progress", ProgressUpdate {
        progress: 100,
        label: if optimize {
            "Optimized VMF generation complete".to_string()
        } else {
            format!("Standard VMF generation complete: {} solids written", total_blocks)
        },
    });
    
    println!("[DEBUG] VMF file generation completed successfully");
    Ok(())
}