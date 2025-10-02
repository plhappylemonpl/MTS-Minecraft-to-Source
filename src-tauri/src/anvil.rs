// Zmieniamy importy, aby zaimportować cechę `Chunk` i nadać alias `CompleteChunk` strukturze.
use fastanvil::{complete::Chunk as CompleteChunk, Chunk, Region};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionCoords {
    pub start_x: i32,
    pub start_z: i32,
    pub end_x: i32,
    pub end_z: i32,
}

#[derive(Debug, Clone)]
pub struct MinecraftBlock {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_type: String,
}

/// Checks if a chunk is "empty" based on the defined criteria.
/// An empty chunk contains only air, barrier blocks, or a single, flat layer of bedrock.
fn is_chunk_empty(chunk: &CompleteChunk) -> bool {
    let mut bedrock_y: Option<isize> = None;

    for y in chunk.y_range() {
        for x in 0..16_u8 {
            for z in 0..16_u8 {
                let block = match chunk.block(x.into(), y, z.into()) {
                    Some(b) => b,
                    None => continue, // Skip air blocks
                };

                match block.name() {
                    "minecraft:air" | "minecraft:barrier" => {
                        continue; // These blocks don't make the chunk non-empty
                    }
                    "minecraft:bedrock" => {
                        if let Some(known_y) = bedrock_y {
                            if known_y != y {
                                return false; // Not a flat layer of bedrock
                            }
                        } else {
                            bedrock_y = Some(y); // First bedrock layer found
                        }
                    }
                    _ => {
                        return false; // Found a non-air, non-barrier, non-bedrock block
                    }
                }
            }
        }
    }

    true // Chunk contains only air, barriers, or a single layer of bedrock
}

fn get_blocks_data<F>(
    region_files: Vec<PathBuf>,
    mut progress_reporter: F,
    dynamic: bool,
    coords: Option<ConversionCoords>,
) -> Result<Vec<MinecraftBlock>, String>
where
    F: FnMut(),
{
    let mut blocks: Vec<MinecraftBlock> = Vec::new();
    let total_regions = region_files.len();
    let mut processed_regions = 0;

    println!("[DEBUG] Starting to process {} region files", total_regions);
    let chunk_coords = coords.map(|c| {
        let min_cx = std::cmp::min(c.start_x, c.end_x) >> 4;
        let max_cx = std::cmp::max(c.start_x, c.end_x) >> 4;
        let min_cz = std::cmp::min(c.start_z, c.end_z) >> 4;
        let max_cz = std::cmp::max(c.start_z, c.end_z) >> 4;
        (min_cx, max_cx, min_cz, max_cz)
    });

    for (_region_index, path) in region_files.iter().enumerate() {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let parts: Vec<&str> = file_name.split('.').collect();
        if parts.len() < 3 {
            continue;
        }
        let region_x: i32 = parts[1].parse().unwrap_or(0);
        let region_z: i32 = parts[2].parse().unwrap_or(0);

        let file = File::open(&path)
            .map_err(|e| format!("Can't open file {}: {}", path.display(), e))?;
        let mut region = Region::from_stream(file)
            .map_err(|e| format!("Error reading region from file {}: {}", path.display(), e))?;

        let mut chunks_in_region = 0;
        let mut blocks_in_region = 0;

        for chunk_data_result in region.iter() {
            let chunk_data = chunk_data_result.map_err(|e| e.to_string())?;
            let chunk_x = region_x * 32 + chunk_data.x as i32;
            let chunk_z = region_z * 32 + chunk_data.z as i32;

             if !dynamic {
                if let Some((min_cx, max_cx, min_cz, max_cz)) = chunk_coords {
                    if chunk_x < min_cx || chunk_x > max_cx || chunk_z < min_cz || chunk_z > max_cz {
                        continue;
                    }
                }
            }

            // Tworzymy obiekt używając aliasu `CompleteChunk`.
            let chunk = CompleteChunk::from_bytes(&chunk_data.data).map_err(|e| e.to_string())?;

            if dynamic && is_chunk_empty(&chunk) {
                progress_reporter();
                continue;
            }

            chunks_in_region += 1;
            let mut _blocks_in_chunk = 0;

            // Iterujemy przez wszystkie bloki w chunku
            for y in chunk.y_range() {
                for x in 0..16_u8 {
                    for z in 0..16_u8 {
                        if let Some(block) = chunk.block(x.into(), y, z.into()) {
                            if block.name() != "minecraft:air" && block.name() != "minecraft:barrier" {
                                let world_x = chunk_x * 16 + x as i32;
                                let world_z = chunk_z * 16 + z as i32;
                                
                                blocks.push(MinecraftBlock {
                                    x: world_x,
                                    y: y as i32,
                                    z: world_z,
                                    block_type: block.name().to_string(),
                                });
                                _blocks_in_chunk += 1;
                                blocks_in_region += 1;
                            }
                        }
                    }
                }
            }
            
            // Debug co 100 chunków
            if chunks_in_region % 100 == 0 {
                println!("[DEBUG] Processed {} chunks in current region, {} blocks found so far", 
                    chunks_in_region, blocks_in_region);
            }
            
            progress_reporter();
        }
        
        processed_regions += 1;
        println!("[DEBUG] Region {}/{} completed: {} chunks, {} blocks", 
            processed_regions, total_regions, chunks_in_region, blocks_in_region);
    }
    
    println!("[DEBUG] Total blocks extracted: {}", blocks.len());
    Ok(blocks)
}

pub fn get_data_from_map<F>(
    path: impl AsRef<Path>,
    progress_reporter: F,
    dynamic: bool,
    coords: Option<ConversionCoords>,
) -> Result<Vec<MinecraftBlock>, String>
where
    F: FnMut(),
{
    let mut region_path = PathBuf::from(path.as_ref());
    region_path.push("region");

    if !region_path.is_dir() {
        return Err(format!(
            "Can't find 'region' folder at path: {}",
            region_path.display()
        ));
    }

    let region_files: Vec<PathBuf> = std::fs::read_dir(&region_path)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "mca"))
        .collect();

    if region_files.is_empty() {
        return Ok(Vec::new());
    }

    get_blocks_data(region_files, progress_reporter, dynamic, coords)
}

pub fn get_total_chunk_count(
    path: impl AsRef<Path>,
    dynamic: bool,
    coords: Option<ConversionCoords>,
) -> Result<usize, String> {
    let mut region_path = PathBuf::from(path.as_ref());
    region_path.push("region");

    let region_files: Vec<PathBuf> = std::fs::read_dir(&region_path)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "mca"))
        .collect();

    if dynamic {
        let mut total_chunks = 0;
        for path in region_files {
            let file = File::open(&path).map_err(|e| e.to_string())?;
            let mut region = Region::from_stream(file).map_err(|e| e.to_string())?;
            total_chunks += region.iter().count();
        }
        return Ok(total_chunks);
    }

    if let Some(c) = coords {
        let (start_cx, start_cz, end_cx, end_cz) =
            (c.start_x >> 4, c.start_z >> 4, c.end_x >> 4, c.end_z >> 4);
        let mut total_chunks = 0;

        for path in region_files {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let parts: Vec<&str> = file_name.split('.').collect();
            if parts.len() < 3 {
                continue;
            }
            let region_x: i32 = parts[1].parse().unwrap_or(0);
            let region_z: i32 = parts[2].parse().unwrap_or(0);

            let file = File::open(&path).map_err(|e| e.to_string())?;
            let mut region = Region::from_stream(file).map_err(|e| e.to_string())?;

            for chunk_data_result in region.iter() {
                let chunk_data = chunk_data_result.map_err(|e| e.to_string())?;
                let chunk_x = region_x * 32 + chunk_data.x as i32;
                let chunk_z = region_z * 32 + chunk_data.z as i32;

                if chunk_x >= start_cx && chunk_x <= end_cx && chunk_z >= start_cz && chunk_z <= end_cz {
                    total_chunks += 1;
                }
            }
        }
        Ok(total_chunks)
    } else {
        let mut total_chunks = 0;
        for path in region_files {
            let file = File::open(&path).map_err(|e| e.to_string())?;
            let mut region = Region::from_stream(file).map_err(|e| e.to_string())?;
            total_chunks += region.iter().count();
        }
        Ok(total_chunks)
    }
}