# MTS_block.py
import logging
from PyVMF import SolidGenerator, Vertex

# Global Block Constants
BLOCK_SIZE = 40
TEXTURE_SCALE = 0.1562

log = logging.getLogger(__name__)

def compute_texture_config(block_name, properties):
    """
    Calculates texture configuration and orientation for a given block based on its name and properties.
    All textures are taken from the "mc_1.21.4/" folder.
    """
    base_path = "mc_1.21.4/"
    config = {}
    orientation = None

    # We use a shorter block name (without the "minecraft:" prefix)
    short_name = block_name.split(":")[-1]

    if short_name == "podzol":
        config = {
            "top": f"{base_path}podzol_top",
            "sides": f"{base_path}podzol_side",
            "bottom": f"{base_path}dirt"
        }
        return config, orientation

    if short_name == "mangrove_roots":
        config = {
            "top": f"{base_path}mangrove_roots_top",
            "sides": f"{base_path}mangrove_roots_side",
            "bottom": f"{base_path}mangrove_roots_top"
        }
        return config, orientation

    # Leaves – add the _leaves_y suffix unless the material is cherry, azalea or flowering_azalea
    if short_name == "leaves":
        material = str(properties.get("material", "oak")).lower()
        if material in ["cherry", "azalea", "flowering_azalea"]:
            texture = f"{base_path}{material}_leaves"
        else:
            texture = f"{base_path}{material}_leaves_y"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    # Wood blocks (log/wood)
    if "log" in short_name:
        material = str(properties.get("material", "oak")).lower()
        # Bamboo support – when the material is "bamboo"
        if material == "bamboo":
            stripped = properties.get("stripped", "false")
            is_stripped = (str(stripped).lower() == "true")
            prefix = "stripped_" if is_stripped else ""
            texture_main = f"{base_path}{prefix}bamboo_block"
            texture_top = f"{base_path}{prefix}bamboo_block_top"
            config = {"top": texture_top, "sides": texture_main, "bottom": texture_top}
            orientation = str(properties.get("axis", "y")).lower()
            return config, orientation
        # Warped and crimson stem support
        if material in ["warped", "crimson"]:
            stripped = properties.get("stripped", "false")
            is_stripped = (str(stripped).lower() == "true")
            prefix = "stripped_" if is_stripped else ""
            texture_top = f"{base_path}{prefix}{material}_stem_top"
            texture_side = f"{base_path}{prefix}{material}_stem"
            config = {"top": texture_top, "sides": texture_side, "bottom": texture_top}
            orientation = str(properties.get("axis", "y")).lower()
            return config, orientation
        # Standard wood
        stripped = properties.get("stripped", "false")
        is_stripped = (str(stripped).lower() == "true")
        axis = str(properties.get("axis", "y")).lower()
        if is_stripped:
            texture_top = f"{base_path}stripped_{material}_log_top"
            texture_side = f"{base_path}stripped_{material}_log"
        else:
            texture_top = f"{base_path}{material}_log_top"
            texture_side = f"{base_path}{material}_log"
        config = {"top": texture_top, "sides": texture_side, "bottom": texture_top}
        orientation = axis  # "x", "y" or "z"
        return config, orientation
    

    if "wood" in short_name:
        material = str(properties.get("material", "oak")).lower()
        # Warped and crimson stem support
        if material in ["warped", "crimson"]:
            stripped = properties.get("stripped", "false")
            is_stripped = (str(stripped).lower() == "true")
            prefix = "stripped_" if is_stripped else ""
            texture_top = f"{base_path}{prefix}{material}_stem"
            texture_side = f"{base_path}{prefix}{material}_stem"
            config = {"top": texture_top, "sides": texture_side, "bottom": texture_top}
            orientation = str(properties.get("axis", "y")).lower()
            return config, orientation
        # Standard wood
        stripped = properties.get("stripped", "false")
        is_stripped = (str(stripped).lower() == "true")
        axis = str(properties.get("axis", "y")).lower()
        if is_stripped:
            texture_top = f"{base_path}stripped_{material}_log"
            texture_side = f"{base_path}stripped_{material}_log"
        else:
            texture_top = f"{base_path}{material}_log"
            texture_side = f"{base_path}{material}_log"
        config = {"top": texture_top, "sides": texture_side, "bottom": texture_top}
        orientation = axis  # "x", "y" or "z"
        return config, orientation


    if short_name == "brick_block":
        config = {
            "top": f"{base_path}bricks",
            "sides": f"{base_path}bricks",
            "bottom": f"{base_path}bricks"
        }
        return config, orientation
    
    if short_name == "honey_block":
        config = {
            "top": f"{base_path}honey_block_top",
            "sides": f"{base_path}honey_block_side",
            "bottom": f"{base_path}honey_block_bottom"
        }
        return config, orientation
    
    if short_name == "lodestone":
        config = {
            "top": f"{base_path}lodestone_top",
            "sides": f"{base_path}lodestone_side",
            "bottom": f"{base_path}lodestone_top"
        }
        return config, orientation
    
    if short_name == "ancient_debris":
        config = {
            "top": f"{base_path}ancient_debris_top",
            "sides": f"{base_path}ancient_debris_side",
            "bottom": f"{base_path}ancient_debris_top"
        }
        return config, orientation

    if short_name == "jukebox":
        config = {
            "top": f"{base_path}jukebox_top",
            "sides": f"{base_path}jukebox_side",
            "bottom": f"{base_path}jukebox_side"
        }
        return config, orientation
    
    if short_name == "pumpkin":
        config = {
            "top": f"{base_path}pumpkin_top",
            "sides": f"{base_path}pumpkin_side",
            "bottom": f"{base_path}pumpkin_top"
        }
        return config, orientation

    if short_name == "basalt":
        axis = str(properties.get("axis", "y")).lower()
        config = {
            "top": f"{base_path}basalt_top",
            "sides": f"{base_path}basalt_side",
            "bottom": f"{base_path}basalt_top"
        }
        orientation = axis
        return config, orientation
    
    if short_name == "polished_basalt":
        axis = str(properties.get("axis", "y")).lower()
        config = {
            "top": f"{base_path}polished_basalt_top",
            "sides": f"{base_path}polished_basalt_side",
            "bottom": f"{base_path}polished_basalt_top"
        }
        orientation = axis
        return config, orientation
    
    if short_name == "hay_block":
        axis = str(properties.get("axis", "y")).lower()
        config = {
            "top": f"{base_path}hay_block_top",
            "sides": f"{base_path}hay_block_side",
            "bottom": f"{base_path}hay_block_top"
        }
        orientation = axis
        return config, orientation
    
    if short_name == "bone_block":
        axis = str(properties.get("axis", "y")).lower()
        config = {
            "top": f"{base_path}bone_block_top",
            "sides": f"{base_path}bone_block_side",
            "bottom": f"{base_path}bone_block_top"
        }
        orientation = axis
        return config, orientation
    
    if short_name == "infested_deepslate":
        axis = str(properties.get("axis", "y")).lower()
        config = {
            "top": f"{base_path}deepslate_top",
            "sides": f"{base_path}deepslate",
            "bottom": f"{base_path}deepslate_top"
        }
        orientation = axis
        return config, orientation

    if short_name == "deepslate":
        axis = str(properties.get("axis", "y")).lower()
        config = {
            "top": f"{base_path}deepslate_top",
            "sides": f"{base_path}deepslate",
            "bottom": f"{base_path}deepslate_top"
        }
        orientation = axis
        return config, orientation

    ##froglight VV
    if short_name == "pearlescent_froglight":
        axis = str(properties.get("axis", "y")).lower()
        config = {
            "top": f"{base_path}pearlescent_froglight_top",
            "sides": f"{base_path}pearlescent_froglight_side",
            "bottom": f"{base_path}pearlescent_froglight_top"
        }
        orientation = axis
        return config, orientation
    if short_name == "verdant_froglight":
        axis = str(properties.get("axis", "y")).lower()
        config = {
            "top": f"{base_path}verdant_froglight_top",
            "sides": f"{base_path}verdant_froglight_side",
            "bottom": f"{base_path}verdant_froglight_top"
        }
        orientation = axis
        return config, orientation
    if short_name == "ochre_froglight":
        axis = str(properties.get("axis", "y")).lower()
        config = {
            "top": f"{base_path}ochre_froglight_top",
            "sides": f"{base_path}ochre_froglight_side",
            "bottom": f"{base_path}ochre_froglight_top"
        }
        orientation = axis
        return config, orientation
    ##froglight ^^
    
    # Color blocks – wool
    if "wool" in short_name:
        color = str(properties.get("color", "white")).lower()
        texture = f"{base_path}{color}_wool"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    # Terracotta and stained_terracotta – we always use the name without "stained_"
    if short_name in ["terracotta", "stained_terracotta"]:
        color = str(properties.get("color", "white")).lower()
        texture = f"{base_path}{color}_terracotta"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    # Concrete i concrete_powder
    if short_name in ["concrete", "concrete_powder"]:
        color = str(properties.get("color", "white")).lower()
        texture = f"{base_path}{color}_{short_name}"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    # Planks
    if short_name == "planks":
        material = str(properties.get("material", "oak")).lower()
        texture = f"{base_path}{material}_planks"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    # Stained glass
    if "stained_glass" in short_name:
        color = str(properties.get("color", "clear")).lower()
        texture = f"{base_path}{color}_stained_glass"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    # Glazed Terracotta – one texture, rotated based on "facing"
    if "glazed_terracotta" in short_name:
        color = str(properties.get("color", "white")).lower()
        texture = f"{base_path}{color}_glazed_terracotta"
        facing = str(properties.get("facing", "north")).lower()  # north, east, south, west
        rotation = {"north": 0, "east": 90, "south": 180, "west": 270}.get(facing, 0)
        config = {"all": texture}
        orientation = {"rotation": rotation, "facing": facing}
        return config, orientation

    # Redstone lamp – depending on the state of 'lit'
    if short_name == "redstone_lamp":
        lit = str(properties.get("lit", "false")).lower() == "true"
        texture = f"{base_path}redstone_lamp_on" if lit else f"{base_path}redstone_lamp"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    # Infested block
    if short_name == "infested_block":
        material = str(properties.get("material", "stone_bricks")).lower()
        texture = f"{base_path}{material}"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    # New copper block support
    if short_name.startswith("waxed_"):
        original_name = short_name[len("waxed_"):]
    else:
        original_name = short_name
    if "copper" in original_name:
        lit = str(properties.get("lit", "false")).lower() == "true"
        powered = str(properties.get("powered", "false")).lower() == "true"
        suffix = ""
        if lit and not powered:
            suffix = "_lit"
        elif (not lit) and powered:
            suffix = "_powered"
        elif lit and powered:
            suffix = "_lit_powered"
        texture = f"{base_path}{original_name}{suffix}"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    # Quartz Pillar - similar to log
    if short_name == "quartz_pillar":
        axis = str(properties.get("axis", "y")).lower()
        config = {
            "top": f"{base_path}quartz_pillar_top",
            "sides": f"{base_path}quartz_pillar",
            "bottom": f"{base_path}quartz_pillar_top"
        }
        orientation = axis
        return config, orientation

    if short_name == "smooth_quartz":
        config = {
            "top": f"{base_path}quartz_block_bottom",
            "sides": f"{base_path}quartz_block_bottom",
            "bottom": f"{base_path}quartz_block_bottom"
        }
        return config, orientation

    if short_name == "quartz_bricks":
        config = {
            "top": f"{base_path}quartz_block_side",
            "sides": f"{base_path}quartz_block_side",
            "bottom": f"{base_path}quartz_block_side"
        }
        return config, orientation

    if short_name == "chiseled_quartz_block":
        config = {
            "top": f"{base_path}chiseled_quartz_block_top",
            "sides": f"{base_path}chiseled_quartz_block",
            "bottom": f"{base_path}chiseled_quartz_block_top"
        }
        return config, orientation
    if short_name == "quartz_block":
        config = {
            "top": f"{base_path}quartz_block_top",
            "sides": f"{base_path}quartz_block_side",
            "bottom": f"{base_path}quartz_block_bottom"
        }
        return config, orientation

    # Creaking heart – works similarly to log, but uses the "active" property
    if short_name == "creaking_heart":
        active = str(properties.get("active", "false")).lower() == "true"
        axis = str(properties.get("axis", "y")).lower()
        if active:
            texture_top = f"{base_path}creaking_heart_top_active"
            texture_side = f"{base_path}creaking_heart_active"
        else:
            texture_top = f"{base_path}creaking_heart_top"
            texture_side = f"{base_path}creaking_heart"
        config = {"top": texture_top, "sides": texture_side, "bottom": texture_top}
        orientation = axis
        return config, orientation

    # Muddy mangrove roots – simplified version of the log
    if short_name == "muddy_mangrove_roots":
        axis = str(properties.get("axis", "y")).lower()
        texture_top = f"{base_path}muddy_mangrove_roots_top"
        texture_side = f"{base_path}muddy_mangrove_roots_side"
        config = {"top": texture_top, "sides": texture_side, "bottom": texture_top}
        orientation = axis
        return config, orientation

    # stone: andesite, diorite, granite
    if short_name in ["andesite", "diorite", "granite"]:
        polished = properties.get("polished", "false")
        is_polished = (str(polished).lower() == "true")
        material = short_name
        if is_polished:
            texture = f"{base_path}polished_{material}"
        else:
            texture = f"{base_path}{material}"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    # Coral block 
    if short_name == "coral_block":
        coral_type = str(properties.get("coral_type", "")).lower()
        if coral_type not in ["brain", "bubble", "fire", "horn", "tube"]:
            coral_type = "brain"
        texture = f"{base_path}{coral_type}_coral_block"
        dead = str(properties.get("dead", "false")).lower() == "true"
        if dead:
            texture = f"{base_path}dead_{coral_type}_coral_block"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    if short_name == "bookshelf":
        config = {"top": f"{base_path}oak_planks",
                  "sides": f"{base_path}bookshelf",
                  "bottom": f"{base_path}oak_planks"}
        return config, orientation

    if short_name == "mycelium":
        config = {"top": f"{base_path}mycelium_top",
                  "sides": f"{base_path}mycelium_side",
                  "bottom": f"{base_path}dirt"}
        return config, orientation

    if short_name == "reinforced_deepslate":
        config = {"top": f"{base_path}reinforced_deepslate_top",
                  "sides": f"{base_path}reinforced_deepslate_side",
                  "bottom": f"{base_path}reinforced_deepslate_bottom"}
        return config, orientation

    if short_name == "melon":
        config = {"top": f"{base_path}melon_top",
                  "sides": f"{base_path}melon_side",
                  "bottom": f"{base_path}melon_top"}
        return config, orientation

    if short_name in ["suspicious_sand", "suspicious_gravel"]:
        dusted = str(properties.get("dusted", "0"))
        texture = f"{base_path}{short_name}_{dusted}"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    if short_name == "tnt":
        config = {"top": f"{base_path}tnt_top",
                  "sides": f"{base_path}tnt_side",
                  "bottom": f"{base_path}tnt_bottom"}
        return config, orientation

    if short_name == "sculk_catalyst":
        config = {"top": f"{base_path}sculk_catalyst_top",
                  "sides": f"{base_path}sculk_catalyst_side",
                  "bottom": f"{base_path}sculk_catalyst_bottom"}
        return config, orientation

    if short_name == "stone_bricks":
        variant = str(properties.get("variant", "normal")).lower()
        if variant == "cracked":
            texture = f"{base_path}cracked_stone_bricks"
        elif variant == "mossy":
            texture = f"{base_path}mossy_stone_bricks"
        else:
            texture = f"{base_path}stone_bricks"
        config = {"top": texture, "sides": texture, "bottom": texture}
        return config, orientation

    if short_name == "dried_kelp_block":
        config = {"top": f"{base_path}dried_kelp_top",
                  "sides": f"{base_path}dried_kelp_side",
                  "bottom": f"{base_path}dried_kelp_bottom"}
        return config, orientation

    if short_name == "snow_block":
        config = {"top": f"{base_path}snow",
                  "sides": f"{base_path}snow",
                  "bottom": f"{base_path}snow"}
        return config, orientation

    if short_name == "magma_block":
        config = {"top": f"{base_path}magma",
                  "sides": f"{base_path}magma",
                  "bottom": f"{base_path}magma"}
        return config, orientation

    if short_name == "red_sandstone":
        variant = str(properties.get("variant", "normal")).lower()
        if variant == "normal":
            config = {"top": f"{base_path}red_sandstone_top",
                      "sides": f"{base_path}red_sandstone",
                      "bottom": f"{base_path}red_sandstone_bottom"}
        elif variant == "smooth":
            config = {"top": f"{base_path}red_sandstone_top",
                      "sides": f"{base_path}red_sandstone_top",
                      "bottom": f"{base_path}red_sandstone_top"}
        elif variant == "cut":
            config = {"top": f"{base_path}red_sandstone_top",
                      "sides": f"{base_path}cut_red_sandstone",
                      "bottom": f"{base_path}red_sandstone_top"}
        elif variant == "chiseled":
            config = {"top": f"{base_path}red_sandstone_top",
                      "sides": f"{base_path}chiseled_red_sandstone",
                      "bottom": f"{base_path}red_sandstone_top"}
        else:
            config = {"top": f"{base_path}red_sandstone_top",
                      "sides": f"{base_path}red_sandstone",
                      "bottom": f"{base_path}red_sandstone_bottom"}
        return config, orientation

    if short_name == "sponge":
        wet = str(properties.get("wet", "false")).lower()
        if wet == "false":
            config = {"top": f"{base_path}sponge",
                      "sides": f"{base_path}sponge",
                      "bottom": f"{base_path}sponge"}
        elif wet == "true":
            config = {"top": f"{base_path}wet_sponge",
                      "sides": f"{base_path}wet_sponge",
                      "bottom": f"{base_path}wet_sponge"}
        return config, orientation
    

    if short_name == "sandstone":
        variant = str(properties.get("variant", "normal")).lower()
        if variant == "normal":
            config = {"top": f"{base_path}sandstone_top",
                      "sides": f"{base_path}sandstone",
                      "bottom": f"{base_path}sandstone_bottom"}
        elif variant == "smooth":
            config = {"top": f"{base_path}sandstone_top",
                      "sides": f"{base_path}sandstone_top",
                      "bottom": f"{base_path}sandstone_top"}
        elif variant == "cut":
            config = {"top": f"{base_path}sandstone_top",
                      "sides": f"{base_path}cut_sandstone",
                      "bottom": f"{base_path}sandstone_top"}
        elif variant == "chiseled":
            config = {"top": f"{base_path}sandstone_top",
                      "sides": f"{base_path}chiseled_sandstone",
                      "bottom": f"{base_path}sandstone_top"}
        else:
            config = {"top": f"{base_path}sandstone_top",
                      "sides": f"{base_path}sandstone",
                      "bottom": f"{base_path}sandstone_bottom"}
        return config, orientation

    if short_name == "grass_block":
        snowy = properties.get("snowy", "false")
        is_snowy = (str(snowy).lower() == "true")
        if is_snowy:
            config = {"top": f"{base_path}snow",
                      "sides": f"{base_path}grass_block_snow",
                      "bottom": f"{base_path}dirt"}
        else:
            config = {"top": f"{base_path}grass_block_top_y",
                      "sides": f"{base_path}grass_block_side_y",
                      "bottom": f"{base_path}dirt"}
        return config, orientation
    
    if short_name == "respawn_anchor":
        charges = properties.get("charges", "0")
        charges = int(str(charges))
        charges = max(0, min(4, charges))
        if charges == 0:
            top_texture = f"{base_path}respawn_anchor_top_off"
        else:
            top_texture = f"{base_path}respawn_anchor_top"
        side_texture = f"{base_path}respawn_anchor_side{charges}"
        config = {
            "top": top_texture,
            "sides": side_texture,
            "bottom": f"{base_path}respawn_anchor_bottom"
        }
        return config, orientation
    
    if short_name == "cartography_table":
        config = {
            "top": f"{base_path}cartography_table_top",
            "bottom": f"{base_path}cartography_table_side3",
            "side_0": f"{base_path}cartography_table_side3",
            "side_1": f"{base_path}cartography_table_side2",
            "side_2": f"{base_path}cartography_table_side3",
            "side_3": f"{base_path}cartography_table_side2"
        }
        return config, orientation

    default_texture = f"{base_path}{short_name}"
    config = {"top": default_texture, "sides": default_texture, "bottom": default_texture}
    return config, orientation

def create_block(vmf, x, y, z, block_type, texture_config, orientation=None):
    """
    Creates a block in a VMF file with the appropriate texture mapping and orientation settings.
    For standard blocks (top, bottom, sides) we set constant UVs using the global TEXTURE_SCALE.
    """
    print(f"Creating block at ({x}, {y}, {z}) with texture scale {TEXTURE_SCALE}")
    
    solid_gen = SolidGenerator()
    vertex = Vertex(x, y, z)
    solid = solid_gen.cube(vertex, BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE)

    # Mapping between side indices and texture config keys
    side_mapping = {
        0: ["side_0", "sides"],  # Z face (north)
        1: ["side_1", "sides"],  # Z face (south) 
        2: ["side_2", "sides"],  # X face (west)
        3: ["side_3", "sides"],  # X face (east)
        4: ["side_4", "top"],    # Top face
        5: ["side_5", "bottom"]  # Bottom face
    }

    for i, side in enumerate(solid.side):
        if orientation is not None and isinstance(orientation, dict) and "rotation" in orientation:
            # Handle special cases with rotation
            side.material = texture_config.get("all", "")
            facing = orientation.get("facing", "north")
            rot = orientation["rotation"]
            if (facing == "north" and i == 0) or (facing == "south" and i == 1) or \
               (facing == "west" and i == 2) or (facing == "east" and i == 3):
                if rot == 0:
                    side.uaxis = f"[1 0 0 0] {TEXTURE_SCALE}"
                    side.vaxis = f"[0 -1 0 0] {TEXTURE_SCALE}"
                elif rot == 90:
                    side.uaxis = f"[0 -1 0 0] {TEXTURE_SCALE}"
                    side.vaxis = f"[1 0 0 0] {TEXTURE_SCALE}"
                elif rot == 180:
                    side.uaxis = f"[-1 0 0 0] {TEXTURE_SCALE}"
                    side.vaxis = f"[0 1 0 0] {TEXTURE_SCALE}"
                elif rot == 270:
                    side.uaxis = f"[0 1 0 0] {TEXTURE_SCALE}"
                    side.vaxis = f"[-1 0 0 0] {TEXTURE_SCALE}"
            else:
                side.uaxis = f"[0 1 0 0] {TEXTURE_SCALE}"
                side.vaxis = f"[0 0 -1 0] {TEXTURE_SCALE}"
        else:
            # Handle axis-based orientation
            if str(orientation).lower() in ["x", "y", "z"]:
                if str(orientation).lower() == "y":
                    texture_keys = side_mapping[4] if i in [4, 5] else side_mapping[i]
                elif str(orientation).lower() == "x":
                    texture_keys = side_mapping[4] if i in [2, 3] else side_mapping[i]
                elif str(orientation).lower() == "z":
                    texture_keys = side_mapping[4] if i in [0, 1] else side_mapping[i]
            else:
                # Try to get texture from side_N first, then fall back to traditional mapping
                texture_keys = side_mapping.get(i, ["sides"])

            # Apply the first available texture from the keys
            for key in texture_keys:
                if key in texture_config:
                    side.material = texture_config[key]
                    break

            # Set UV axes based on side index
            if i == 5:  # Bottom
                side.uaxis = f"[0 0 0] {TEXTURE_SCALE}"
                side.vaxis = f"[1 0 0] {TEXTURE_SCALE}"
            elif i == 4:  # Top
                side.uaxis = f"[0 0 0] {TEXTURE_SCALE}"
                side.vaxis = f"[1 1 0] {TEXTURE_SCALE}"
            else:  # Sides
                side.uaxis = f"[0 0 0] {TEXTURE_SCALE}"
                side.vaxis = f"[1 1 0] {TEXTURE_SCALE}"

        side.lightmapscale = 16
        side.smoothing_groups = 0
        side.justify = 6

    vmf.add_solids([solid])
    return solid