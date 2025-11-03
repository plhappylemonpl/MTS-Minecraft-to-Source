use std::io::Write;
use std::collections::{HashMap, HashSet};
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

// Struktura dla zoptymalizowanego bloku (może być większy niż 1x1x1)
#[derive(Debug, Clone)]
struct OptimizedBlock {
    x: i32,
    y: i32,
    z: i32,
    width: i32,   // rozmiar w osi X
    height: i32,  // rozmiar w osi Z (Y w Minecrafcie)
    depth: i32,   // rozmiar w osi Y (Z w Minecrafcie)
    block_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum MaterialConfig {
    Simple(String),
    Complex {
        top: Option<String>,
        side: Option<String>,
        bottom: Option<String>,
        front: Option<String>,
        end: Option<String>,
        properties: Option<HashMap<String, HashMap<String, MaterialOverride>>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaterialOverride {
    top: Option<String>,
    side: Option<String>,
    bottom: Option<String>,
}

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
    
    (block_name.to_string(), HashMap::new())
}

fn load_material_config() -> HashMap<String, MaterialConfig> {
    let config_content = include_str!("../material_config.json");
    
    match serde_json::from_str::<HashMap<String, serde_json::Value>>(config_content) {
        Ok(raw_config) => {
            let mut config = HashMap::new();
            
            for (key, value) in raw_config {
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
                        let front = obj.get("front").and_then(|v| v.as_str()).map(String::from);
                        let end = obj.get("end").and_then(|v| v.as_str()).map(String::from);
                        
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
                        
                        config.insert(key, MaterialConfig::Complex { 
                            top, side, bottom, front, end, properties 
                        });
                    }
                    _ => {}
                }
            }
            
            config
        }
        Err(e) => {
            println!("[WARNING] Failed to parse material_config.json: {}. Using auto-detection.", e);
            HashMap::new()
        }
    }
}

fn get_material_for_side(
    block_name: &str, 
    side: BlockSide, 
    config: &HashMap<String, MaterialConfig>
) -> String {
    const MATERIAL_PREFIX: &str = "mc_1.21.8/";
    
    let (base_name, properties) = parse_block_properties(block_name);
    
    if let Some(material_config) = config.get(&base_name) {
        match material_config {
            MaterialConfig::Simple(texture) => {
                return format!("{}{}", MATERIAL_PREFIX, texture);
            }
            MaterialConfig::Complex { top, side: side_texture, bottom, front, end, properties: config_properties } => {
                let mut final_top = top.clone();
                let mut final_side = side_texture.clone();
                let mut final_bottom = bottom.clone();
                
                if let Some(config_props) = config_properties {
                    for (prop_name, prop_value) in &properties {
                        if let Some(prop_config) = config_props.get(prop_name) {
                            if let Some(override_config) = prop_config.get(prop_value) {
                                if override_config.top.is_some() {
                                    final_top = override_config.top.clone();
                                }
                                if override_config.side.is_some() {
                                    final_side = override_config.side.clone();
                                }
                                if override_config.bottom.is_some() {
                                    final_bottom = override_config.bottom.clone();
                                }
                            }
                        }
                    }
                }
                
                let texture = match side {
                    BlockSide::Top => {
                        final_top.or(final_side)
                    }
                    BlockSide::Bottom => {
                        final_bottom.or(final_side)
                    }
                    BlockSide::North | BlockSide::South => {
                        front.clone().or(final_side)
                    }
                    BlockSide::East | BlockSide::West => {
                        end.clone().or(final_side)
                    }
                };
                
                if let Some(tex) = texture {
                    return format!("{}{}", MATERIAL_PREFIX, tex);
                }
            }
        }
    }
    
    let clean_name = if let Some(stripped) = base_name.strip_prefix("minecraft:") {
        stripped
    } else {
        &base_name
    };
    
    format!("{}{}", MATERIAL_PREFIX, clean_name)
}

// NOWA FUNKCJA: Optymalizacja bloków przez łączenie sąsiadujących
fn optimize_blocks(blocks: Vec<VmfBlock>) -> Vec<OptimizedBlock> {
    println!("[OPTIMIZE] Starting greedy mesh optimization with {} blocks", blocks.len());
    
    // Grupuj bloki po typie
    let mut blocks_by_type: HashMap<String, Vec<VmfBlock>> = HashMap::new();
    for block in blocks {
        blocks_by_type.entry(block.block_type.clone())
            .or_insert_with(Vec::new)
            .push(block);
    }
    
    let mut optimized = Vec::new();
    
    for (block_type, mut type_blocks) in blocks_by_type {
        println!("[OPTIMIZE] Processing {} blocks of type {}", type_blocks.len(), block_type);
        
        // Sortuj dla lepszej optymalizacji
        type_blocks.sort_by(|a, b| {
            a.y.cmp(&b.y)
                .then(a.z.cmp(&b.z))
                .then(a.x.cmp(&b.x))
        });
        
        let mut used: HashSet<(i32, i32, i32)> = HashSet::new();
        
        for block in &type_blocks {
            let key = (block.x, block.y, block.z);
            if used.contains(&key) {
                continue;
            }
            
            // Greedy meshing: rozszerz blok w osiach X, Y, Z
            let mut width = 1;
            let mut height = 1;
            let mut depth = 1;
            
            // Rozszerz w osi X
            'expand_x: loop {
                let test_x = block.x + width;
                for dy in 0..height {
                    for dz in 0..depth {
                        let test_key = (test_x, block.y + dy, block.z + dz);
                        if used.contains(&test_key) || 
                           !type_blocks.iter().any(|b| b.x == test_x && b.y == block.y + dy && b.z == block.z + dz) {
                            break 'expand_x;
                        }
                    }
                }
                width += 1;
            }
            
            // Rozszerz w osi Y (wysokość w Minecraft)
            'expand_y: loop {
                let test_y = block.y + height;
                for dx in 0..width {
                    for dz in 0..depth {
                        let test_key = (block.x + dx, test_y, block.z + dz);
                        if used.contains(&test_key) ||
                           !type_blocks.iter().any(|b| b.x == block.x + dx && b.y == test_y && b.z == block.z + dz) {
                            break 'expand_y;
                        }
                    }
                }
                height += 1;
            }
            
            // Rozszerz w osi Z
            'expand_z: loop {
                let test_z = block.z + depth;
                for dx in 0..width {
                    for dy in 0..height {
                        let test_key = (block.x + dx, block.y + dy, test_z);
                        if used.contains(&test_key) ||
                           !type_blocks.iter().any(|b| b.x == block.x + dx && b.y == block.y + dy && b.z == test_z) {
                            break 'expand_z;
                        }
                    }
                }
                depth += 1;
            }
            
            // Oznacz użyte bloki
            for dx in 0..width {
                for dy in 0..height {
                    for dz in 0..depth {
                        used.insert((block.x + dx, block.y + dy, block.z + dz));
                    }
                }
            }
            
            optimized.push(OptimizedBlock {
                x: block.x,
                y: block.y,
                z: block.z,
                width,
                height,
                depth,
                block_type: block_type.clone(),
            });
        }
        
        println!("[OPTIMIZE] {} blocks -> {} optimized meshes", type_blocks.len(), optimized.len());
    }
    
    println!("[OPTIMIZE] Total optimization: {} blocks -> {} meshes", 
        optimized.iter().map(|b| b.width * b.height * b.depth).sum::<i32>(),
        optimized.len());
    
    optimized
}

fn create_optimized_solid(block: &OptimizedBlock, block_size: i32, config: &HashMap<String, MaterialConfig>) -> VmfSolid {
    let solid_id = get_next_id();
    
    let top_material = get_material_for_side(&block.block_type, BlockSide::Top, config);
    let bottom_material = get_material_for_side(&block.block_type, BlockSide::Bottom, config);
    let north_material = get_material_for_side(&block.block_type, BlockSide::North, config);
    let south_material = get_material_for_side(&block.block_type, BlockSide::South, config);
    let east_material = get_material_for_side(&block.block_type, BlockSide::East, config);
    let west_material = get_material_for_side(&block.block_type, BlockSide::West, config);
    
    // Oblicz współrzędne z uwzględnieniem rozmiaru
    let x1 = block.x * block_size;
    let x2 = x1 + (block.width * block_size);
    let y1 = block.z * block_size;
    let y2 = y1 + (block.depth * block_size);
    let z1 = block.y * block_size;
    let z2 = z1 + (block.height * block_size);

    let sides = vec![
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

fn create_cube_solid(block: &VmfBlock, block_size: i32, config: &HashMap<String, MaterialConfig>) -> VmfSolid {
    let optimized = OptimizedBlock {
        x: block.x,
        y: block.y,
        z: block.z,
        width: 1,
        height: 1,
        depth: 1,
        block_type: block.block_type.clone(),
    };
    create_optimized_solid(&optimized, block_size, config)
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

pub fn convert_and_write_vmf(
    blocks: Vec<VmfBlock>, 
    vmf_path: &str,
    app: tauri::AppHandle,
    optimize: bool,
) -> Result<(), String> {
    let total_blocks = blocks.len();
    let block_size = 40;
    
    println!("[DEBUG] Starting VMF conversion with {} blocks, optimization: {}", total_blocks, optimize);
    
    let material_config = load_material_config();
    println!("[DEBUG] Loaded {} exception materials from config", material_config.len());
    
    let mut file = std::fs::File::create(vmf_path).map_err(|e| e.to_string())?;
    write_vmf_header(&mut file)?;
    
    let _ = app.emit("detail_progress", ProgressUpdate {
        progress: 0,
        label: "Starting VMF generation...".to_string(),
    });
    
    if optimize {
        println!("[DEBUG] Using optimized mode - greedy mesh algorithm");
        
        let _ = app.emit("detail_progress", ProgressUpdate {
            progress: 10,
            label: "Optimizing block meshes...".to_string(),
        });
        
        // PRAWDZIWA OPTYMALIZACJA - łączenie bloków
        let optimized_blocks = optimize_blocks(blocks);
        let mesh_count = optimized_blocks.len();
        
        println!("[DEBUG] Optimization complete: {} original blocks -> {} meshes", 
            total_blocks, mesh_count);
        
        let _ = app.emit("detail_progress", ProgressUpdate {
            progress: 30,
            label: format!("Optimized to {} meshes", mesh_count),
        });
        
        let batch_size = calculate_batch_size();
        let mut processed = 0;
        
        for (batch_idx, chunk) in optimized_blocks.chunks(batch_size).enumerate() {
            let _ = app.emit("detail_progress", ProgressUpdate {
                progress: (30 + ((processed * 60) / mesh_count)) as u16,
                label: format!("Writing batch {}: meshes {}-{}", 
                    batch_idx + 1, processed, processed + chunk.len()),
            });
            
            let mut solids = Vec::with_capacity(chunk.len());
            for block in chunk {
                solids.push(create_optimized_solid(block, block_size, &material_config));
            }
            
            for solid in &solids {
                write_solid_to_file(&mut file, solid)?;
            }
            
            processed += chunk.len();
            drop(solids);
            
            if batch_idx % 10 == 0 {
                file.flush().map_err(|e| e.to_string())?;
            }
        }
        
        println!("[DEBUG] Optimized: {} meshes written to VMF", mesh_count);
            
    } else {
        println!("[DEBUG] Using standard mode - no optimization");
        
        let batch_size = calculate_batch_size();
        let mut processed_blocks = 0;
        
        for (batch_idx, chunk) in blocks.chunks(batch_size).enumerate() {
            let batch_start = batch_idx * batch_size;
            
            let _ = app.emit("detail_progress", ProgressUpdate {
                progress: ((batch_start as f32 / total_blocks as f32) * 90.0) as u16,
                label: format!("Processing batch {}: blocks {}-{}", 
                    batch_idx + 1, batch_start, batch_start + chunk.len()),
            });
            
            let mut solids = Vec::with_capacity(chunk.len());
            for block in chunk {
                solids.push(create_cube_solid(block, block_size, &material_config));
            }
            
            for solid in &solids {
                write_solid_to_file(&mut file, solid)?;
            }
            
            processed_blocks += chunk.len();
            drop(solids);
            
            if batch_idx % 10 == 0 {
                file.flush().map_err(|e| e.to_string())?;
            }
        }
    }
    
    write_vmf_footer(&mut file)?;
    file.flush().map_err(|e| e.to_string())?;
    
    let _ = app.emit("detail_progress", ProgressUpdate {
        progress: 100,
        label: "VMF generation complete".to_string(),
    });
    
    println!("[DEBUG] VMF file generation completed successfully");
    Ok(())
}