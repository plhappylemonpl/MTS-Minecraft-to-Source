# MTS_optimization.py

from PyVMF import SolidGenerator, Vertex
from MTS_block import TEXTURE_SCALE, compute_texture_config


def partition_layer(coords):
    """
    Given a set of coordinates (x, z) in one layer,
    creates a matrix of occupied cells and ensures blocks are connected.
    """
    if not coords:
        return []
    
    # Convert coords to set for O(1) lookup
    coords_set = set(coords)
    
    # Find boundaries
    min_x = min(x for x, z in coords)
    max_x = max(x for x, z in coords)
    min_z = min(z for x, z in coords)
    max_z = max(z for x, z in coords)
    
    width = max_x - min_x + 1
    depth = max_z - min_z + 1
    
    # Create grid
    grid = [[0] * width for _ in range(depth)]
    for (x, z) in coords:
        grid[z - min_z][x - min_x] = 1
    
    rects = []
    for i in range(depth):
        for j in range(width):
            if grid[i][j] == 1:
                # Check horizontal continuity
                max_w = 0
                while (j + max_w < width and 
                       grid[i][j + max_w] == 1 and 
                       (j + max_w + 1 >= width or grid[i][j + max_w + 1] == 1 or max_w == 0)):
                    max_w += 1
                
                # Check vertical continuity
                max_d = 1
                is_continuous = True
                while i + max_d < depth and is_continuous:
                    # Verify entire row is filled AND connected
                    for col in range(j, j + max_w):
                        if grid[i + max_d][col] != 1 or (
                           # Check if this block is connected to the previous row
                           not any(grid[i + max_d - 1][c] == 1 
                                 for c in range(max(0, col-1), min(width, col+2)))):
                            is_continuous = False
                            break
                    if is_continuous:
                        max_d += 1
                    else:
                        break
                
                # Mark used cells
                for di in range(max_d):
                    for dj in range(max_w):
                        grid[i + di][j + dj] = 0
                
                rect_min_x = j + min_x
                rect_min_z = i + min_z
                rects.append((rect_min_x, rect_min_z, max_w, max_d))
    
    return rects

def merge_layers(rect_dict, block_type, properties):
    """
    Merges rectangles from consecutive layers (key = y).
    Returns a list of cuboids as tuples:
    (min_x, min_y, min_z, size_x, size_y, size_z, block_type, properties)
    """
    merged = []
    ys = sorted(rect_dict.keys())
    processed = {y: [False] * len(rect_dict[y]) for y in ys}

    for i, y in enumerate(ys):
        for j, rect in enumerate(rect_dict[y]):
            if processed[y][j]:
                continue
            current_rect = rect
            min_y = y
            max_y = y
            processed[y][j] = True
            for y_next in ys[i+1:]:
                found = False
                for k, rect_next in enumerate(rect_dict[y_next]):
                    if processed[y_next][k]:
                        continue
                    if rect_next == current_rect:
                            processed[y_next][k] = True
                            max_y = y_next 
                            found = True
                            break
                if not found:
                    break
            size_x = current_rect[2]
            size_z = current_rect[3]
            size_y = max_y - min_y + 1
            merged.append((current_rect[0], min_y, current_rect[1], size_x, size_z, size_y, block_type, properties))
    return merged

def merge_layers_horizontal(layer_rects, block_type, properties):
    """Merging layers for the horizontal direction – we use the default method."""
    return merge_layers(layer_rects, block_type, properties)

def merge_layers_vertical(layer_rects, block_type, properties):
    """
    Merging layers for the vertical direction:
    - Transpose coordinates (swap x and z)
    - Merge using merge_layers
    - Transpose the result back
    """

    transposed = {}
    for y, rects in layer_rects.items():
        # Zamiana: (min_x, min_z, width, depth) -> (min_z, min_x, depth, width)
        transposed[y] = [(r[0], r[1], r[2], r[3]) for r in rects]

    merged_transposed = merge_layers(transposed, block_type, properties)

    merged = []
    for (min_z, min_y, min_x, size_z, size_y, size_x, btype, props) in merged_transposed:
        merged.append((min_x, min_y, min_z, size_x, size_y, size_z, btype, props))

    return merged

def simulate_optimization(blocks, direction="horizontal"):
    """
    Performs an optimization simulation for the given direction.
    Returns a metric - here the number of merged cuboids.
    """
    merged = optimize_blocks(blocks, direction)
    error = len(merged)
    return {"error": error, "merged": merged}

def analyze_blocks(blocks):
    """
    Analyzes the block layout, performing simulation for both horizontal and vertical directions,
    and returns the direction with the least number of resulting objects.
    """
    stats_h = simulate_optimization(blocks, direction="horizontal")
    stats_v = simulate_optimization(blocks, direction="vertical")
    if stats_h["error"] <= stats_v["error"]:
        return "horizontal"
    else:
        return "vertical"


def can_merge_blocks(block1_type, block1_props, block2_type, block2_props):
    """
    Enhanced block merging validation
    """
    if block1_type != block2_type:
        return False

    # Rozszerzona lista właściwości krytycznych
    critical_properties = [
        "variant",      
        "axis",         
        "facing",       
        "charges",      
        "snowy",        
        "material",     
        "lit",          
        "powered",      
        "type",         
        "stripped",     
        "coral_type",   
        "dead",         
        "polished",     
        "color",
        "north",        # Dodane właściwości kierunkowe
        "south",
        "east",
        "west",
        "rotation"
    ]

    # Jeśli którykolwiek blok ma właściwości kierunkowe, traktuj je jako różne
    directional_props = ["facing", "axis", "rotation", "north", "south", "east", "west"]
    if any(prop in block1_props or prop in block2_props for prop in directional_props):
        return False

    # Standardowe sprawdzenie pozostałych właściwości
    for prop in critical_properties:
        val1 = block1_props.get(prop)
        val2 = block2_props.get(prop)
        if val1 != val2:
            return False

    return True

def optimize_blocks(blocks, direction=None):
    """
    Optimizes blocks by combining blocks with identical properties.
    """
    if direction is None:
        direction = analyze_blocks(blocks)
        print(f"Selected optimization direction: {direction}")
    
    # Group files by type and properties
    groups = {}
    for bx, by, bz, btype, props in blocks:
        # The key contains the block type and all critical properties
        key = (btype, tuple(sorted((k, str(v)) for k, v in props.items())))
        groups.setdefault(key, {}).setdefault(by, set()).add((bx, bz))
    
    merged_objs = []
    for (btype, props_tuple), layers in groups.items():
        layer_rects = {}
        for y, coords in layers.items():
            rects = partition_layer(coords)
            layer_rects[y] = rects
        
        # Convert props_tuple back to dictionary
        props = dict(props_tuple)
        
        if direction == "horizontal":
            merged_objs.extend(merge_layers_horizontal(layer_rects, btype, props))
        else:
            merged_objs.extend(merge_layers_vertical(layer_rects, btype, props))
    
    return merged_objs

from PyVMF import SolidGenerator, Vertex
from MTS_block import TEXTURE_SCALE, compute_texture_config

def setSolidSides(solid, texture_config, properties):
    if (str(properties.get("axis", "y")).lower() == "y"):
        solid.side[4].material = texture_config.get("top", texture_config.get("sides", ""))
        solid.side[5].material = texture_config.get("bottom", texture_config.get("sides", ""))
        for i in range(0, 4):
            solid.side[i].material = texture_config.get("sides", texture_config.get("all", ""))
    elif (str(properties.get("axis", "y")).lower() == "x"):
        for i in range(0, 2):
            solid.side[i].material = texture_config.get("sides", texture_config.get("all", ""))
        for i in range(2, 4):
            solid.side[i].material = texture_config.get("top", texture_config.get("sides", ""))
        for i in range(4, 6):
            solid.side[i].material = texture_config.get("sides", texture_config.get("all", ""))
    elif (str(properties.get("axis", "y")).lower() == "z"):
        for i in range(0, 2):
            solid.side[i].material = texture_config.get("top", texture_config.get("sides", ""))
        for i in range(2, 6):
            solid.side[i].material = texture_config.get("sides", texture_config.get("all", ""))

    # TOP
    solid.side[4].uaxis = "[1 0 0 0] " + str(TEXTURE_SCALE)
    solid.side[4].vaxis = "[0 -1 0 0] " + str(TEXTURE_SCALE)
    # BOTTOM
    solid.side[5].uaxis = "[1 0 0 0] " + str(TEXTURE_SCALE)
    solid.side[5].vaxis = "[0 -1 0 0] " + str(TEXTURE_SCALE)
    # XSide
    for i in range(2, 4):
        solid.side[i].uaxis = "[0 1 0 0] " + str(TEXTURE_SCALE)
        solid.side[i].vaxis = "[0 0 -1 0] " + str(TEXTURE_SCALE)
    # ZSide
    for i in range(0, 2):
        solid.side[i].uaxis = "[1 0 0 0] " + str(TEXTURE_SCALE)
        solid.side[i].vaxis = "[0 0 -1 0] " + str(TEXTURE_SCALE)

    return solid

def create_cuboid(vmf, x, y, z, dx, dy, dz, block_type, properties):
    """
    Creates a cuboid in VMF with position (x, y, z) and dimensions (dx, dy, dz).
    Sets textures and UVs to maintain default rotation.

    Assuming:
    solid.side[4] is TOP,
    solid.side[5] is BOTTOM,
    solid.side[2]-[3] are XSIDES.
    solid.side[0]-[1] are ZSIDES.
    """
    solid_gen = SolidGenerator()
    vertex = Vertex(x, y, z)
    try:
        solid = solid_gen.cuboid(vertex, dx, dy, dz)
    except AttributeError:
        solid = solid_gen.cube(vertex, dx, dy, dz)
    
    texture_config, orientation = compute_texture_config(block_type, properties)
    
    solid = setSolidSides(solid, texture_config, properties)

    vmf.world.solids.append(solid)

# Optional testing
if __name__ == "__main__":
    sample_blocks = [
        (0, 0, 0, "grass_block", {"snowy": "false"}),
        (1, 0, 0, "grass_block", {"snowy": "false"}),
        (0, 0, 1, "grass_block", {"snowy": "false"}),
        (1, 0, 1, "grass_block", {"snowy": "false"}),
        (0, 1, 0, "grass_block", {"snowy": "false"}),
        (1, 1, 0, "grass_block", {"snowy": "false"}),
        (0, 1, 1, "grass_block", {"snowy": "false"}),
        (1, 1, 1, "grass_block", {"snowy": "false"}),
        (3, 0, 0, "dirt", {"moist": "true"})
    ]
    optimized = optimize_blocks(sample_blocks)
    for obj in optimized:
        print(obj)
