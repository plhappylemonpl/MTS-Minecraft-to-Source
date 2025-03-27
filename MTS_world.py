# MTS_world.py
import logging
from amulet import load_format
from amulet.api.level import World, Structure
from amulet.api.wrapper.world_format_wrapper import WorldFormatWrapper
from amulet.api.wrapper.structure_format_wrapper import StructureFormatWrapper
from amulet.api.errors import ChunkDoesNotExist, DimensionDoesNotExist
from typing import Union
from MTS_block import BLOCK_SIZE, compute_texture_config, create_block
from PyVMF import VMF, World as VMFWorld
from MTS_optimization import optimize_blocks, create_cuboid 

log = logging.getLogger(__name__)

def load_level(path: str) -> Union[World, Structure]:
    log.info(f"Loading level {path}")
    format_wrapper = load_format(path)
    print(f"FormatWrapper type: {type(format_wrapper).__name__}")
    if isinstance(format_wrapper, WorldFormatWrapper):
        try:
            world = World(path, format_wrapper)
            print(f"World initialized successfully: {world}")
            return world
        except Exception as e:
            print(f"Error during World initialization: {e}")
            raise e
    elif isinstance(format_wrapper, StructureFormatWrapper):
        try:
            structure = Structure(path, format_wrapper)
            print(f"Structure initialized successfully: {structure}")
            return structure
        except Exception as e:
            print(f"Error during Structure initialization: {e}")
            raise e
    else:
        raise Exception(
            f"FormatWrapper of type {format_wrapper.__class__.__name__} is not supported. Report this to a developer."
        )

def get_surface_blocks(world_path, x1, z1, x2, z2, dimension='minecraft:overworld'):
    """
    Iterates through all chunks in a given area and returns a list of blocks
    (excluding air and barrier) along with their coordinates and properties.
    Height range: 319 to -64.
    """
    try:
        print(f"Loading world from: {world_path}")
        world = load_level(world_path)
    except Exception as e:
        print(f"Error loading world: {e}")
        return []
    
    surface_blocks = []
    cx1, cz1 = x1 // 16, z1 // 16
    cx2, cz2 = x2 // 16, z2 // 16

    total_chunks = (max(cx1, cx2) - min(cx1, cx2) + 1) * (max(cz1, cz2) - min(cz1, cz2) + 1)
    processed_chunks = 0

    print(f"Total chunks to process: {total_chunks}")
    for cx in range(min(cx1, cx2), max(cx1, cx2) + 1):
        for cz in range(min(cz1, cz2), max(cz1, cz2) + 1):
            try:
                print(f"Processing chunk at ({cx}, {cz}) in dimension {dimension}")
                chunk = world.get_chunk(cx, cz, dimension)
                start_x = max(0, x1 - cx * 16) if cx == cx1 else 0
                end_x = min(16, x2 - cx * 16 + 1) if cx == cx2 else 16
                start_z = max(0, z1 - cz * 16) if cz == cz1 else 0
                end_z = min(16, z2 - cz * 16 + 1) if cz == cz2 else 16

                for x in range(start_x, end_x):
                    for z in range(start_z, end_z):
                        for y in range(319, -65, -1):
                            block = chunk.get_block(x, y, z)
                            if block.base_name not in ["air", "barrier"]:
                                world_x = cx * 16 + x
                                world_z = cz * 16 + z
                                surface_blocks.append((world_x, y, world_z, block.base_name, block.properties))
                                print(f"Found {block.base_name} at ({world_x}, {y}, {world_z})")
                processed_chunks += 1
                print(f"Processed {processed_chunks}/{total_chunks} chunks")
            except ChunkDoesNotExist:
                print(f"Chunk at ({cx}, {cz}) does not exist. Skipping.")
            except DimensionDoesNotExist:
                print(f"Dimension {dimension} does not exist. Skipping.")
                break

    try:
        world.close()
    except Exception as e:
        print(f"Error closing world: {e}")

    return surface_blocks

def adjust_orientation(orientation, mirror_axis):
    if orientation is None or mirror_axis not in ("x", "y"):
        return orientation
    if isinstance(orientation, str):
        orient = orientation.lower()
        if mirror_axis == "x":
            if orient == "east":
                return "west"
            elif orient == "west":
                return "east"
        elif mirror_axis == "y":
            if orient == "north":
                return "south"
            elif orient == "south":
                return "north"
        return orientation
    if isinstance(orientation, dict):
        facing = orientation.get("facing", "north").lower()
        if mirror_axis == "x":
            if facing == "east":
                facing = "west"
            elif facing == "west":
                facing = "east"
        elif mirror_axis == "y":
            if facing == "north":
                facing = "south"
            elif facing == "south":
                facing = "north"
        rot = orientation.get("rotation", 0)
        new_rot = (360 - rot) % 360
        new_orientation = orientation.copy()
        new_orientation["facing"] = facing
        new_orientation["rotation"] = new_rot
        return new_orientation
    return orientation

def get_and_convert_blocks(world_path, x1, z1, x2, z2, output_vmf, dimension='minecraft:overworld', mirror_axis=None, optimize=False):
    """
    Takes all blocks from the selected area and converts them to VMF format.
    If optimize is True, optimization (block merging) will be performed before export.
    """
    surface_blocks = get_surface_blocks(world_path, x1, z1, x2, z2, dimension)
    
    if optimize:
        print("Block optimization...")
        surface_blocks = optimize_blocks(surface_blocks)
    
    vmf = VMF()
    vmf.world = VMFWorld()

    if surface_blocks:
        if not optimize:
            # For "raw" blocks - structure: (x, y, z, block_type, properties)
            min_x = min(x for x, _, _, _, _ in surface_blocks)
            min_y = min(y for _, y, _, _, _ in surface_blocks)
            min_z = min(z for _, _, z, _, _ in surface_blocks)
            max_x = max(x for x, _, _, _, _ in surface_blocks)
            max_z = max(z for _, _, z, _, _ in surface_blocks)
            
            for x, y, z, block_type, properties in surface_blocks:
                if mirror_axis == "x":
                    hammer_x = (max_x - x) * BLOCK_SIZE
                    hammer_y = (z - min_z) * BLOCK_SIZE
                elif mirror_axis == "y":
                    hammer_x = (x - min_x) * BLOCK_SIZE
                    hammer_y = (max_z - z) * BLOCK_SIZE
                else:
                    hammer_x = (x - min_x) * BLOCK_SIZE
                    hammer_y = (z - min_z) * BLOCK_SIZE
                hammer_z = (y - min_y) * BLOCK_SIZE
            
                texture_config, orientation = compute_texture_config(block_type, properties)
                orientation = adjust_orientation(orientation, mirror_axis)
                create_block(vmf, hammer_x, hammer_y, hammer_z, block_type, texture_config, orientation)
        else:
            # For optimized cuboids - structure: (min_x, min_y, min_z, size_x, size_y, size_z, block_type, properties)
            # To correctly calculate mirror_axis, we determine the global max (on x or z axis) within the optimized data.
            global_max_x = max(min_x + size_x for min_x, _, _, size_x, _, _, _, _ in surface_blocks)
            global_max_z = max(min_z + size_z for _, _, min_z, _, _, size_z, _, _ in surface_blocks)
            
            for min_x, min_y, min_z, size_x, size_y, size_z, block_type, properties in surface_blocks:
                if mirror_axis == "x":
                    hammer_x = (global_max_x - min_x - size_x) * BLOCK_SIZE
                    hammer_y = (min_z) * BLOCK_SIZE
                elif mirror_axis == "y":
                    hammer_x = (min_x) * BLOCK_SIZE
                    hammer_y = (global_max_z - min_z - size_z) * BLOCK_SIZE
                else:
                    hammer_x = (min_x) * BLOCK_SIZE
                    hammer_y = (min_z) * BLOCK_SIZE
                hammer_z = (min_y) * BLOCK_SIZE
                
                # We create a cuboid with dimensions corresponding to the merged object
                create_cuboid(vmf, hammer_x, hammer_y, hammer_z, size_x * BLOCK_SIZE, size_y * BLOCK_SIZE, size_z * BLOCK_SIZE, block_type, properties)
            
    print(f"Saving VMF file to: {output_vmf}")
    try:
        vmf.export(output_vmf)
        print(f"Successfully saved VMF file to: {output_vmf}")
    except Exception as e:
        print(f"Error saving VMF file: {e}")
        raise
