#!/usr/bin/env python3
"""Generate stylized Bevy lane-defense unit GLB models for Blender.

Run with:
  blender --background --python scripts/generate_3d_models.py
"""

from __future__ import annotations

import math
from pathlib import Path

import bpy


ROOT = Path(__file__).resolve().parents[1]
PLANT_DIR = ROOT / "assets" / "models" / "plants"
MONSTER_DIR = ROOT / "assets" / "models" / "monsters"


def clear_scene() -> None:
    if bpy.context.view_layer is None:
        return
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)

    for collection in (bpy.data.meshes, bpy.data.materials, bpy.data.curves):
        for block in list(collection):
            if block.users == 0:
                collection.remove(block)


def material(
    name: str,
    color: tuple[float, float, float],
    *,
    roughness: float,
    emission: tuple[float, float, float] | None = None,
    emission_strength: float = 0.0,
) -> bpy.types.Material:
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    mat.use_backface_culling = False

    tree = mat.node_tree
    tree.nodes.clear()
    nodes = tree.nodes
    links = tree.links

    out = nodes.new(type="ShaderNodeOutputMaterial")
    bsdf = nodes.new(type="ShaderNodeBsdfPrincipled")

    out.location = (260, 0)
    bsdf.location = (0, 0)

    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Roughness"].default_value = max(0.35, min(0.8, roughness))
    bsdf.inputs["Metallic"].default_value = 0.0

    if emission is not None and emission_strength > 0:
        bsdf.inputs["Emission Color"].default_value = (*emission, 1.0)
        bsdf.inputs["Emission Strength"].default_value = min(3.0, emission_strength)
    else:
        bsdf.inputs["Emission Color"].default_value = (0.0, 0.0, 0.0, 1.0)
        bsdf.inputs["Emission Strength"].default_value = 0.0

    links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def material_set(prefix: str, specs: dict[str, tuple[float, float, float] | tuple[tuple[float, float, float], float, tuple[float, float, float] | None, float]]) -> dict[str, bpy.types.Material]:
    mats: dict[str, bpy.types.Material] = {}
    for key, spec in specs.items():
        color = spec[0]
        rough = spec[1]
        emission = spec[2]
        emission_strength = spec[3] if len(spec) > 3 else 0.0
        mats[key] = material(f"{prefix}_{key}", color, roughness=rough, emission=emission, emission_strength=emission_strength)
    return mats


def add_uv_sphere(
    name: str,
    loc: tuple[float, float, float],
    scale: tuple[float, float, float],
    mat: bpy.types.Material,
    *,
    segments: int = 12,
    rings: int = 6,
    smooth: bool = True,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=rings, location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    obj.data.materials.append(mat)
    if smooth:
        for poly in obj.data.polygons:
            poly.use_smooth = True
    return obj


def add_cube(
    name: str,
    loc: tuple[float, float, float],
    scale: tuple[float, float, float],
    mat: bpy.types.Material,
    *,
    smooth: bool = False,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cube_add(location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    obj.data.materials.append(mat)
    if smooth:
        for poly in obj.data.polygons:
            poly.use_smooth = True
    return obj


def add_cylinder(
    name: str,
    loc: tuple[float, float, float],
    radius: float,
    depth: float,
    mat: bpy.types.Material,
    *,
    vertices: int = 16,
    smooth: bool = True,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.scale = (1.0, 1.0, 1.0)
    obj.data.materials.append(mat)
    if smooth:
        for poly in obj.data.polygons:
            poly.use_smooth = True
    return obj


def add_cone(
    name: str,
    loc: tuple[float, float, float],
    radius1: float,
    radius2: float,
    depth: float,
    mat: bpy.types.Material,
    *,
    vertices: int = 16,
    smooth: bool = False,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cone_add(vertices=vertices, radius1=radius1, radius2=radius2, depth=depth, location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.data.materials.append(mat)
    if smooth:
        for poly in obj.data.polygons:
            poly.use_smooth = True
    return obj


def rotate(obj: bpy.types.Object, x: float = 0.0, y: float = 0.0, z: float = 0.0) -> None:
    obj.rotation_euler = (x, y, z)


def nudge(obj: bpy.types.Object, offset: tuple[float, float, float]) -> None:
    obj.location.x += offset[0]
    obj.location.y += offset[1]
    obj.location.z += offset[2]


def add_bevel(obj: bpy.types.Object, width: float = 0.025, segments: int = 2) -> None:
    mod = obj.modifiers.new(name="Bevel", type="BEVEL")
    mod.width = width
    mod.segments = segments
    mod.limit_method = "ANGLE"
    mod.angle_limit = 0.85


def add_subsurf(obj: bpy.types.Object, levels: int = 1) -> None:
    mod = obj.modifiers.new(name="Subdivision", type="SUBSURF")
    mod.levels = levels
    mod.render_levels = levels


def add_facial_feature(
    *,
    x: float,
    y: float,
    z: float,
    eye_mat: bpy.types.Material,
    pupil_mat: bpy.types.Material,
    mouth_mat: bpy.types.Material,
) -> None:
    eye = add_uv_sphere("eye", (x, y, z), (0.07, 0.045, 0.07), eye_mat, segments=10, rings=6)
    add_uv_sphere("pupil", (x + 0.03, y, z), (0.028, 0.018, 0.028), pupil_mat, segments=8, rings=5)
    # Brows and tiny mouth for silhouette and expression.
    add_cube("brow", (x - 0.02, y + 0.05, z + 0.06), (0.032, 0.008, 0.018), mouth_mat)


def add_tusks(
    loc: tuple[float, float, float],
    mat: bpy.types.Material,
    z_offsets: tuple[float, float],
    length: float = 0.09,
) -> None:
    for z in z_offsets:
        tusk = add_cylinder("tusk", (loc[0], loc[1], loc[2] + z), 0.012, length, mat, vertices=12)
        rotate(tusk, 0.0, math.radians(95), 0.0)


def add_leaf_cluster(name: str, base_x: float, base_y: float, base_z: float, mat: bpy.types.Material) -> None:
    for i, a in enumerate((0.0, 0.45, -0.45, 0.9)):
        leaf = add_uv_sphere(f"{name}_{i}", (base_x + math.cos(a) * 0.1, base_y + i * 0.02, base_z + math.sin(a) * 0.1), (0.16, 0.05, 0.16), mat, segments=10, rings=5)
        rotate(leaf, 0.0, 0.0, math.radians(a * 45))


def export_glb(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_yup=True,
    )
    print(f"exported {path}")


def plant_materials() -> dict[str, dict[str, bpy.types.Material]]:
    return {
        "sprout-slinger": material_set(
            "sprout-slinger",
            {
                "stem": ((0.22, 0.58, 0.21), 0.62, None, 0.0),
                "leaf": ((0.35, 0.84, 0.28), 0.45, None, 0.0),
                "pod": ((0.58, 0.78, 0.22), 0.38, None, 0.0),
                "eye": ((0.95, 0.98, 1.00), 0.30, None, 0.0),
                "pupil": ((0.04, 0.08, 0.05), 0.2, None, 0.0),
            },
        ),
        "sunbloom": material_set(
            "sunbloom",
            {
                "stem": ((0.25, 0.53, 0.17), 0.62, None, 0.0),
                "petal": ((0.97, 0.83, 0.20), 0.40, None, 0.0),
                "core": ((0.94, 0.52, 0.17), 0.35, None, 0.0),
                "face": ((0.16, 0.33, 0.12), 0.55, None, 0.0),
                "eye": ((0.99, 0.97, 0.85), 0.28, None, 0.0),
            },
        ),
        "bark-bulwark": material_set(
            "bark-bulwark",
            {
                "bark": ((0.35, 0.21, 0.08), 0.70, None, 0.0),
                "shell": ((0.72, 0.58, 0.26), 0.40, None, 0.0),
                "cap": ((0.22, 0.30, 0.14), 0.55, None, 0.0),
                "leaf": ((0.30, 0.70, 0.32), 0.32, None, 0.0),
                "accent": ((0.95, 0.95, 0.86), 0.25, None, 0.0),
            },
        ),
        "frost-sprout": material_set(
            "frost-sprout",
            {
                "frost": ((0.30, 0.83, 0.93), 0.28, (0.55, 0.88, 1.0), 0.6),
                "stem": ((0.29, 0.55, 0.29), 0.65, None, 0.0),
                "ice": ((0.72, 0.93, 1.0), 0.30, (0.52, 0.88, 1.0), 1.0),
                "pod": ((0.56, 0.82, 0.96), 0.35, (0.7, 0.95, 1.0), 0.4),
                "eye": ((0.98, 0.99, 1.00), 0.2, None, 0.0),
            },
        ),
        "twin-pod": material_set(
            "twin-pod",
            {
                "stem": ((0.20, 0.50, 0.15), 0.58, None, 0.0),
                "fruit": ((0.61, 0.82, 0.24), 0.42, None, 0.0),
                "leaf": ((0.34, 0.80, 0.28), 0.48, None, 0.0),
                "eye": ((0.98, 0.99, 1.00), 0.2, None, 0.0),
                "accent": ((0.99, 0.73, 0.21), 0.36, None, 0.0),
            },
        ),
        "leaf-lobber": material_set(
            "leaf-lobber",
            {
                "stem": ((0.22, 0.52, 0.18), 0.55, None, 0.0),
                "leaf": ((0.42, 0.78, 0.24), 0.42, None, 0.0),
                "leaf_dark": ((0.15, 0.33, 0.12), 0.7, None, 0.0),
                "cargo": ((0.65, 0.57, 0.20), 0.33, None, 0.0),
                "arm": ((0.82, 0.72, 0.28), 0.45, None, 0.0),
                "fruit": ((0.55, 0.80, 0.30), 0.40, None, 0.0),
            },
        ),
        "briar-mat": material_set(
            "briar-mat",
            {
                "ground": ((0.14, 0.30, 0.08), 0.75, None, 0.0),
                "root": ((0.34, 0.23, 0.09), 0.42, None, 0.0),
                "thorn": ((0.07, 0.10, 0.04), 0.25, None, 0.0),
                "flower": ((0.70, 0.32, 0.18), 0.6, None, 0.0),
                "leaf": ((0.52, 0.82, 0.38), 0.48, None, 0.0),
            },
        ),
        "blast-berry": material_set(
            "blast-berry",
            {
                "berry": ((0.96, 0.41, 0.14), 0.50, None, 0.0),
                "skin": ((0.70, 0.24, 0.06), 0.35, None, 0.0),
                "fuse": ((0.93, 0.92, 0.46), 0.28, None, 0.0),
                "core": ((1.0, 0.72, 0.10), 0.36, (0.95, 0.5, 0.08), 1.6),
                "flame": ((0.98, 0.45, 0.12), 0.30, (1.0, 0.55, 0.15), 2.2),
            },
        ),
        "ember-stump": material_set(
            "ember-stump",
            {
                "wood": ((0.44, 0.28, 0.12), 0.72, None, 0.0),
                "ash": ((0.68, 0.56, 0.31), 0.55, None, 0.0),
                "ember": ((1.0, 0.40, 0.10), 0.42, (1.0, 0.35, 0.09), 2.4),
                "flame": ((1.0, 0.56, 0.12), 0.3, (1.0, 0.8, 0.35), 2.7),
                "glow": ((0.96, 0.86, 0.46), 0.44, (1.0, 0.8, 0.35), 0.9),
            },
        ),
        "scent-root": material_set(
            "scent-root",
            {
                "bulb": ((0.74, 0.60, 0.33), 0.62, None, 0.0),
                "root": ((0.38, 0.20, 0.08), 0.6, None, 0.0),
                "leaf": ((0.26, 0.65, 0.23), 0.45, None, 0.0),
                "accent": ((0.91, 0.90, 0.43), 0.58, (0.95, 0.96, 0.42), 0.7),
                "eye": ((0.98, 0.98, 0.98), 0.2, None, 0.0),
            },
        ),
    }


def monster_materials() -> dict[str, dict[str, bpy.types.Material]]:
    return {
        "walker": material_set(
            "walker",
            {
                "skin": ((0.42, 0.50, 0.30), 0.6, None, 0.0),
                "dark": ((0.17, 0.20, 0.13), 0.5, None, 0.0),
                "cloth": ((0.54, 0.49, 0.35), 0.5, None, 0.0),
                "scar": ((0.95, 0.93, 0.73), 0.4, None, 0.0),
                "eye": ((0.06, 0.07, 0.04), 0.22, (0.9, 0.2, 0.2), 0.8),
            },
        ),
        "conehead": material_set(
            "conehead",
            {
                "skin": ((0.44, 0.53, 0.31), 0.60, None, 0.0),
                "dark": ((0.17, 0.20, 0.13), 0.45, None, 0.0),
                "cone": ((0.95, 0.42, 0.08), 0.45, None, 0.0),
                "cloth": ((0.53, 0.46, 0.34), 0.48, None, 0.0),
                "eye": ((0.90, 0.87, 0.76), 0.3, (1.0, 0.95, 0.75), 0.6),
            },
        ),
        "runner": material_set(
            "runner",
            {
                "skin": ((0.40, 0.48, 0.28), 0.50, None, 0.0),
                "dark": ((0.14, 0.17, 0.11), 0.4, None, 0.0),
                "strip": ((0.71, 0.36, 0.20), 0.35, None, 0.0),
                "cloth": ((0.42, 0.49, 0.26), 0.52, None, 0.0),
                "eye": ((0.98, 0.98, 0.96), 0.2, (0.82, 0.15, 0.15), 1.2),
            },
        ),
        "buckethead": material_set(
            "buckethead",
            {
                "cloth": ((0.36, 0.30, 0.22), 0.65, None, 0.0),
                "skin": ((0.46, 0.54, 0.33), 0.60, None, 0.0),
                "dark": ((0.16, 0.18, 0.11), 0.45, None, 0.0),
                "bucket": ((0.44, 0.47, 0.50), 0.33, None, 0.0),
                "rim": ((0.72, 0.72, 0.78), 0.24, None, 0.0),
                "eye": ((0.10, 0.12, 0.09), 0.2, (0.7, 0.2, 0.2), 0.8),
            },
        ),
        "brute": material_set(
            "brute",
            {
                "dark": ((0.12, 0.12, 0.10), 0.55, None, 0.0),
                "skin": ((0.45, 0.52, 0.33), 0.65, None, 0.0),
                "plate": ((0.58, 0.58, 0.60), 0.28, None, 0.0),
                "clasp": ((0.30, 0.24, 0.16), 0.5, None, 0.0),
                "cloth": ((0.40, 0.44, 0.22), 0.58, None, 0.0),
                "eye": ((0.85, 0.80, 0.70), 0.4, (1.0, 0.2, 0.2), 1.2),
            },
        ),
        "healer": material_set(
            "healer",
            {
                "cloth": ((0.30, 0.28, 0.38), 0.65, None, 0.0),
                "eye": ((0.95, 0.97, 1.00), 0.25, None, 0.0),
                "skin": ((0.40, 0.50, 0.34), 0.58, None, 0.0),
                "robe": ((0.50, 0.27, 0.66), 0.45, None, 0.0),
                "gold": ((0.78, 0.64, 0.25), 0.35, None, 0.0),
                "orb": ((0.64, 0.78, 0.98), 0.32, (0.6, 0.9, 1.0), 1.8),
                "dark": ((0.17, 0.20, 0.14), 0.45, None, 0.0),
            },
        ),
        "jumper": material_set(
            "jumper",
            {
                "dark": ((0.11, 0.12, 0.10), 0.55, None, 0.0),
                "skin": ((0.39, 0.48, 0.27), 0.58, None, 0.0),
                "cloth": ((0.44, 0.31, 0.18), 0.5, None, 0.0),
                "spring": ((0.66, 0.64, 0.58), 0.22, None, 0.0),
                "plate": ((0.29, 0.33, 0.18), 0.6, None, 0.0),
                "eye": ((0.1, 0.12, 0.09), 0.22, (0.9, 0.6, 0.2), 0.9),
            },
        ),
        "digger": material_set(
            "digger",
            {
                "cloth": ((0.34, 0.27, 0.18), 0.65, None, 0.0),
                "dark": ((0.10, 0.10, 0.09), 0.55, None, 0.0),
                "skin": ((0.43, 0.51, 0.31), 0.6, None, 0.0),
                "dirt": ((0.44, 0.30, 0.16), 0.65, None, 0.0),
                "shovel": ((0.62, 0.59, 0.49), 0.35, None, 0.0),
                "metal": ((0.28, 0.30, 0.34), 0.26, None, 0.0),
                "eye": ((0.96, 0.95, 0.83), 0.2, (0.9, 0.45, 0.15), 1.0),
            },
        ),
        "frostbite": material_set(
            "frostbite",
            {
                "skin": ((0.56, 0.80, 0.85), 0.40, (0.68, 0.90, 1.0), 0.8),
                "dark": ((0.16, 0.24, 0.32), 0.44, None, 0.0),
                "ice": ((0.74, 0.92, 1.0), 0.30, (0.75, 0.95, 1.0), 1.2),
                "cloth": ((0.42, 0.45, 0.52), 0.5, None, 0.0),
                "eye": ((0.18, 0.24, 0.28), 0.2, (0.7, 0.95, 1.0), 1.4),
            },
        ),
        "gargantuar": material_set(
            "gargantuar",
            {
                "skin": ((0.43, 0.53, 0.33), 0.58, None, 0.0),
                "plate": ((0.53, 0.48, 0.27), 0.3, None, 0.0),
                "cloth": ((0.30, 0.24, 0.18), 0.65, None, 0.0),
                "eye": ((0.08, 0.09, 0.06), 0.2, (1.0, 0.26, 0.06), 1.0),
                "club": ((0.26, 0.16, 0.07), 0.26, None, 0.0),
                "dark": ((0.12, 0.11, 0.09), 0.55, None, 0.0),
            },
        ),
    }


def apply_organic_modifiers(obj: bpy.types.Object) -> None:
    add_subsurf(obj, levels=1)
    for poly in obj.data.polygons:
        poly.use_smooth = True


def plant_sprout_slinger(path: Path) -> None:
    clear_scene()
    mats = plant_materials()["sprout-slinger"]

    stem = add_cylinder("stem", (0.0, 0.40, 0.0), 0.07, 0.68, mats["stem"], vertices=18)
    rotate(stem, 0, 0, math.radians(3))
    add_bevel(stem, width=0.01)
    stem.scale = (1.0, 1.0, 1.05)

    add_leaf_cluster("leaf", 0.0, 0.56, 0.17, mats["leaf"])

    head = add_uv_sphere("head", (0.06, 0.88, 0.0), (0.28, 0.20, 0.24), mats["pod"], segments=14, rings=8)
    apply_organic_modifiers(head)

    muzzle = add_cylinder("muzzle", (0.30, 0.88, 0.0), 0.10, 0.24, mats["pod"], vertices=12)
    rotate(muzzle, 0, math.radians(90), 0)
    add_bevel(muzzle, width=0.012)

    add_facial_feature(x=0.29, y=0.94, z=0.08, eye_mat=mats["eye"], pupil_mat=mats["pupil"], mouth_mat=mats["leaf"])

    mouth = add_cube("mouth", (0.34, 0.82, 0.0), (0.035, 0.018, 0.06), mats["pupil"], smooth=False)
    rotate(mouth, 0.0, 0.0, math.radians(-8))

    tongue = add_uv_sphere("tongue", (0.36, 0.82, 0.04), (0.03, 0.01, 0.04), mats["pod"], segments=8, rings=4)
    export_glb(path)


def plant_sunbloom(path: Path) -> None:
    clear_scene()
    mats = plant_materials()["sunbloom"]

    stem = add_cylinder("stem", (0.0, 0.34, 0.0), 0.06, 0.58, mats["stem"], vertices=14)
    rotate(stem, 0, 0, math.radians(-6))
    add_bevel(stem, 0.008)

    core = add_uv_sphere("flower_core", (0.0, 0.90, 0.0), (0.26, 0.22, 0.23), mats["core"], segments=14, rings=8)
    apply_organic_modifiers(core)

    for i in range(10):
        ang = i / 10.0 * math.tau
        petal = add_uv_sphere(
            f"petal_{i}",
            (
                math.cos(ang) * 0.28,
                0.90 + 0.02 * math.sin(i * 0.9),
                math.sin(ang) * 0.18,
            ),
            (0.12, 0.06, 0.09),
            mats["petal"],
            segments=10,
            rings=6,
        )
        add_bevel(petal, 0.005)
        rotate(petal, 0.0, 0.0, math.radians(i * 15))

    for x in (-0.14, 0.14):
        eye = add_uv_sphere("face_eye", (0.18, 0.98, x), (0.055, 0.035, 0.055), mats["eye"], segments=8, rings=5)
        rotate(eye, 0, 0, math.radians(-10 * x * 10))

    add_cube("nose", (0.32, 0.90, 0.0), (0.03, 0.03, 0.12), mats["face"])
    add_cube("mouth", (0.35, 0.84, 0.0), (0.01, 0.018, 0.09), mats["face"])
    export_glb(path)


def plant_bark_bulwark(path: Path) -> None:
    clear_scene()
    mats = plant_materials()["bark-bulwark"]

    shell = add_uv_sphere("shell", (0.0, 0.64, 0.0), (0.34, 0.44, 0.28), mats["shell"], segments=14, rings=8)
    apply_organic_modifiers(shell)
    for i in range(4):
        add_bevel(add_cube(f"plate_{i}", (0.0, 0.58 + i * 0.05, -0.08 + i * 0.06), (0.34, 0.03, 0.08), mats["bark"]), width=0.005)

    cap = add_uv_sphere("cap", (0.0, 1.02, 0), (0.28, 0.08, 0.20), mats["cap"], segments=12, rings=4)
    add_bevel(cap, 0.006)

    add_facial_feature(x=0.22, y=0.90, z=0.06, eye_mat=mats["accent"], pupil_mat=mats["leaf"], mouth_mat=mats["bark"])
    tusk = add_cylinder("guard", (0.07, 0.90, 0.19), 0.09, 0.02, mats["cap"], vertices=8)
    rotate(tusk, 0, math.radians(85), 0)
    export_glb(path)


def plant_frost_sprout(path: Path) -> None:
    clear_scene()
    mats = plant_materials()["frost-sprout"]

    stem = add_cylinder("stem", (0.0, 0.40, 0.0), 0.06, 0.70, mats["stem"], vertices=16)
    add_bevel(stem, 0.008)

    add_leaf_cluster("leaf", 0.0, 0.56, 0.13, mats["frost"])

    head = add_uv_sphere("head", (0.03, 0.90, 0.0), (0.26, 0.23, 0.20), mats["ice"], segments=14, rings=8)
    apply_organic_modifiers(head)
    rotate(head, 0, 0, math.radians(-10))

    nozzle = add_cylinder("muzzle", (0.31, 0.90, 0.0), 0.095, 0.22, mats["ice"], vertices=12)
    rotate(nozzle, 0, math.radians(90), 0)

    for i, offset in enumerate((-0.07, 0.00, 0.07)):
        spike = add_cone(
            f"ice_spike_{i}",
            (0.0, 1.10, offset),
            0.03,
            0.0,
            0.14,
            mats["frost"],
            vertices=10,
            smooth=True,
        )
        rotate(spike, math.radians(-10 + i * 8), 0, math.radians(offset * 120))

    add_facial_feature(x=0.25, y=0.95, z=0.06, eye_mat=mats["eye"], pupil_mat=mats["pod"], mouth_mat=mats["ice"])
    export_glb(path)


def plant_twin_pod(path: Path) -> None:
    clear_scene()
    mats = plant_materials()["twin-pod"]

    stem = add_cylinder("stem", (0.0, 0.40, 0.0), 0.07, 0.60, mats["stem"], vertices=16)
    add_bevel(stem, 0.008)
    add_leaf_cluster("leaf", 0.0, 0.60, 0.16, mats["leaf"])

    for z in (-0.14, 0.14):
        head = add_uv_sphere("pod", (0.05, 0.88, z), (0.20, 0.16, 0.16), mats["fruit"], segments=12, rings=7)
        apply_organic_modifiers(head)
        muzzle = add_cylinder("muzzle", (0.28, 0.88, z), 0.08, 0.22, mats["fruit"], vertices=12)
        rotate(muzzle, 0, math.radians(90), 0)
        add_bevel(muzzle, 0.008)

    for z in (-0.07, 0.07):
        add_facial_feature(x=0.24, y=0.97, z=z + 0.03, eye_mat=mats["eye"], pupil_mat=mats["accent"], mouth_mat=mats["fruit"])

    add_tusks((0.38, 0.91, 0.0), mats["accent"], (-0.03, 0.03), 0.07)
    export_glb(path)


def plant_leaf_lobber(path: Path) -> None:
    clear_scene()
    mats = plant_materials()["leaf-lobber"]

    base = add_cylinder("base", (0.0, 0.24, 0.0), 0.17, 0.28, mats["stem"], vertices=16)
    add_bevel(base, 0.01)

    support = add_cylinder("support", (0.0, 0.52, -0.03), 0.08, 0.10, mats["leaf_dark"], vertices=12)
    rotate(support, 0, 0, math.radians(-20))
    add_bevel(support, 0.01)

    arm = add_cylinder("lobber_arm", (0.17, 0.76, 0.06), 0.08, 0.85, mats["arm"], vertices=12)
    rotate(arm, 0.0, 0.0, math.radians(-28))
    add_bevel(arm, 0.008)

    basket = add_uv_sphere("basket", (0.43, 0.90, 0.0), (0.18, 0.13, 0.12), mats["cargo"], segments=12, rings=6)
    rotate(basket, 0, math.radians(4), 0)
    add_bevel(basket, 0.007)

    add_cone("throw_arm", (0.45, 0.82, 0.0), 0.0, 0.05, 0.12, mats["cargo"], vertices=10, smooth=True)

    add_uv_sphere("cabbage_rock", (0.59, 1.0, 0.0), (0.09, 0.09, 0.07), mats["fruit"], segments=10, rings=5)

    add_leaf_cluster("leaf", 0.05, 0.70, -0.22, mats["leaf"])
    add_facial_feature(x=0.24, y=0.93, z=0.08, eye_mat=mats["leaf"], pupil_mat=mats["leaf_dark"], mouth_mat=mats["arm"])
    export_glb(path)


def plant_briar_mat(path: Path) -> None:
    clear_scene()
    mats = plant_materials()["briar-mat"]

    mat = add_cube("mat", (0.0, 0.08, 0.0), (0.52, 0.06, 0.37), mats["ground"])
    add_bevel(mat, 0.01)

    for x in (-0.34, -0.17, 0.0, 0.17, 0.34):
        for z in (-0.22, 0.0, 0.22):
            thorn = add_cone(
                f"thorn_{x}_{z}",
                (x, 0.18, z),
                0.055,
                0.0,
                0.19,
                mats["thorn"],
                vertices=10,
            )
            rotate(thorn, 0, 0, math.radians((x + z) * 200))
            add_bevel(thorn, 0.005)

    add_uv_sphere("flower", (0.0, 0.16, 0.0), (0.18, 0.10, 0.17), mats["flower"], segments=10, rings=6)
    add_cylinder("stem", (0.0, 0.07, 0.0), 0.04, 0.16, mats["root"], vertices=10)

    for angle in (0.0, 1.2, 2.4):
        add_leaf_pair = add_uv_sphere(
            f"leaf_spread_{angle}",
            (0.19 * math.cos(angle), 0.24, 0.19 * math.sin(angle)),
            (0.12, 0.06, 0.15),
            mats["leaf"],
            segments=8,
            rings=4,
        )
        rotate(add_leaf_pair, 0.0, 0.0, angle)

    add_facial_feature(x=-0.12, y=0.14, z=0.18, eye_mat=mats["leaf"], pupil_mat=mats["flower"], mouth_mat=mats["root"])
    export_glb(path)


def plant_blast_berry(path: Path) -> None:
    clear_scene()
    mats = plant_materials()["blast-berry"]

    berry = add_uv_sphere("berry", (0.0, 0.46, 0.0), (0.34, 0.34, 0.34), mats["berry"], segments=16, rings=8)
    apply_organic_modifiers(berry)
    add_bevel(berry, 0.01)

    fuse = add_cylinder("fuse", (0.05, 0.84, 0.0), 0.02, 0.30, mats["fuse"], vertices=8)
    rotate(fuse, 0, math.radians(95), 0)

    core = add_uv_sphere("core", (0.11, 1.05, 0.0), (0.11, 0.09, 0.11), mats["core"], segments=12, rings=8)
    add_bevel(core, 0.005)

    for i in range(4):
        flame = add_cone(
            f"flame_{i}",
            (0.08, 1.15 + (i % 2) * 0.04, -0.09 + i * 0.06),
            0.06,
            0.0,
            0.12,
            mats["flame"],
            vertices=10,
            smooth=False,
        )
        rotate(flame, math.radians(-8), 0, math.radians(i * 20))

    add_facial_feature(x=0.26, y=0.96, z=0.02, eye_mat=mats["fuse"], pupil_mat=mats["core"], mouth_mat=mats["skin"])
    export_glb(path)


def plant_ember_stump(path: Path) -> None:
    clear_scene()
    mats = plant_materials()["ember-stump"]

    stump = add_cylinder("stump", (0.0, 0.44, 0.0), 0.26, 0.78, mats["wood"], vertices=20)
    add_bevel(stump, 0.012)

    ash = add_cylinder("ashes", (0.0, 0.84, 0.0), 0.18, 0.10, mats["ash"], vertices=10)
    add_bevel(ash, 0.008)

    ember = add_uv_sphere("ember", (0.0, 0.92, 0.0), (0.16, 0.08, 0.14), mats["ember"], segments=12, rings=6)
    apply_organic_modifiers(ember)

    for x in (-0.16, -0.05, 0.05, 0.16):
        flame = add_cone("flame", (x, 1.03, 0.0), 0.055, 0.0, 0.19, mats["flame"], vertices=10)
        rotate(flame, 0, 0, math.radians(x * 200))

    for i in range(3):
        smoke = add_uv_sphere(f"glow_{i}", (0.15, 1.08 + i * 0.08, -0.08 + i * 0.08), (0.05, 0.05, 0.03), mats["glow"], segments=8, rings=5)
        rotate(smoke, 0.0, 0.0, math.radians(i * 30))

    add_facial_feature(x=0.17, y=0.95, z=0.06, eye_mat=mats["flame"], pupil_mat=mats["glow"], mouth_mat=mats["wood"])
    export_glb(path)


def plant_scent_root(path: Path) -> None:
    clear_scene()
    mats = plant_materials()["scent-root"]

    bulb = add_uv_sphere("bulb", (0.0, 0.42, 0.0), (0.30, 0.40, 0.30), mats["bulb"], segments=16, rings=8)
    add_subsurf(bulb, 1)

    for i, angle in enumerate((-35, -18, 0, 18, 35)):
        root = add_cylinder(
            f"root_{i}",
            (0.0, 0.13, (i - 2) * 0.09),
            0.033,
            0.58,
            mats["root"],
            vertices=8,
        )
        rotate(root, 0, math.radians(90), math.radians(angle))
        add_bevel(root, 0.006)

    add_leaf_cluster("leaf", 0.0, 0.70, -0.06, mats["leaf"])

    for i in range(3):
        wisp = add_uv_sphere(
            f"scent_{i}",
            (0.12 * ((-1) ** i), 0.94 + i * 0.04, 0.08 * i - 0.08),
            (0.10, 0.04, 0.10),
            mats["accent"],
            segments=8,
            rings=4,
        )
        rotate(wisp, 0.0, 0.0, math.radians(20 * ((-1) ** i)))

    for i in range(3):
        add_uv_sphere(f"orb_{i}", (0.28, 0.90 + i * 0.07, -0.08 + i * 0.08), (0.05, 0.05, 0.05), mats["accent"], segments=6, rings=4)

    add_facial_feature(x=0.2, y=0.96, z=0.06, eye_mat=mats["eye"], pupil_mat=mats["root"], mouth_mat=mats["accent"])
    export_glb(path)


def monster_core(mats: dict[str, bpy.types.Material], scale: float = 1.0) -> None:
    torso = add_uv_sphere("torso", (0.0, 0.72 * scale, 0.0), (0.22 * scale, 0.38 * scale, 0.17 * scale), mats["skin"], segments=14, rings=8)
    rotate(torso, 0, 0, math.radians(6))

    head = add_uv_sphere("head", (-0.02 * scale, 1.18 * scale, 0.0), (0.17 * scale, 0.18 * scale, 0.15 * scale), mats["skin"], segments=12, rings=6)
    rotate(head, 0, 0, 0)

    for x in (-0.16, 0.16):
        limb = add_cylinder("arm", (0.0 + x * 0.55, 0.82 * scale, 0), 0.045 * scale, 0.56 * scale, mats["cloth"], vertices=10)
        rotate(limb, 0, 0, math.radians(20 if x < 0 else -20))
        add_bevel(limb, 0.006)

    for x in (-0.12, 0.12):
        leg = add_cylinder("leg", (x * scale, 0.28 * scale, 0.0), 0.058 * scale, 0.58 * scale, mats["dark"], vertices=10)
        rotate(leg, math.radians(4), 0, math.radians(-10 if x < 0 else 10))
        add_bevel(leg, 0.006)

    chest = add_cube("chest", (0.0, 0.82 * scale, 0.03), (0.28 * scale, 0.18 * scale, 0.11 * scale), mats["cloth"])

    eye_mat = mats["eye"]
    for z in (-0.055, 0.055):
        add_facial_feature(x=-0.22 * scale, y=1.16 * scale, z=z, eye_mat=eye_mat, pupil_mat=mats["dark"], mouth_mat=mats["skin"])
    add_tusks(( -0.16 * scale, 1.08 * scale, 0.0), mats["dark"], (0.03, -0.03), 0.07 * scale)

    add_cube("mouth_band", (-0.21 * scale, 1.04 * scale, 0.0), (0.08 * scale, 0.016 * scale, 0.05 * scale), mats["skin"])
    add_subsurf(head, 1)
    add_subsurf(torso, 1)


def monster_walker(path: Path) -> None:
    clear_scene()
    monster_core(monster_materials()["walker"])
    export_glb(path)


def monster_conehead(path: Path) -> None:
    clear_scene()
    mats = monster_materials()["conehead"]
    monster_core(mats)
    cone = add_cone("cone_hat", (-0.03, 1.46, 0.0), 0.22, 0.03, 0.36, mats["cone"], vertices=16)
    rotate(cone, 0, 0, math.radians(-10))
    add_bevel(cone, 0.004)
    visor = add_cube("visor", (-0.22, 1.38, 0.0), (0.04, 0.09, 0.22), mats["cloth"])
    rotate(visor, 0, 0, math.radians(5))
    export_glb(path)


def monster_runner(path: Path) -> None:
    clear_scene()
    mats = monster_materials()["runner"]
    monster_core(mats, scale=0.93)
    lean = bpy.data.objects.get("head")
    if lean is not None:
        lean.location.x -= 0.03

    sprint = add_cylinder("speed_strip", (0.06, 0.84, 0.19), 0.08, 0.20, mats["strip"], vertices=10)
    rotate(sprint, 0, 0, 0)
    add_bevel(sprint, 0.004)
    add_cube("leg_band", (-0.14, 0.58, 0.0), (0.14, 0.05, 0.08), mats["strip"])
    add_tusks((-0.17, 1.07, 0.0), mats["skin"], (0.045, -0.045), 0.06)
    export_glb(path)


def monster_buckethead(path: Path) -> None:
    clear_scene()
    mats = monster_materials()["buckethead"]
    monster_core(mats)
    bucket = add_cylinder("bucket_helmet", (-0.02, 1.35, 0.0), 0.19, 0.24, mats["bucket"], vertices=18)
    rotate(bucket, 0, 0, math.radians(-90))
    rim = add_torus = add_cylinder("bucket_rim", (-0.01, 1.40, 0.0), 0.205, 0.02, mats["rim"], vertices=18)
    rotate(add_torus, 0.0, 0.0, 0)
    add_bevel(bucket, 0.005)
    add_bevel(add_torus, 0.002)
    handle = add_torus if False else add_cylinder("bucket_handle", (0.16, 1.31, 0.0), 0.025, 0.16, mats["rim"], vertices=8)
    rotate(handle, 0, 0, math.radians(90))
    nudge(handle, (-0.16, 0, 0))
    export_glb(path)


def monster_brute(path: Path) -> None:
    clear_scene()
    mats = monster_materials()["brute"]
    monster_core(mats, scale=1.18)

    shoulders = add_cube("shoulders", (0.0, 1.0, 0.0), (0.48, 0.13, 0.18), mats["plate"])
    add_bevel(shoulders, 0.008)

    for i, x in enumerate((-0.2, 0.2)):
        gauntlet = add_cube(f"gauntlet_{i}", (x, 0.77, 0.00), (0.13, 0.06, 0.16), mats["clasp"])
        rotate(gauntlet, 0, 0, math.radians(12 if x < 0 else -12))

    for z in (-0.17, 0.17):
        knee = add_cube(f"plate_{z}", (z * 0.1, 0.42, 0.0), (0.30, 0.05, 0.09), mats["plate"])
        rotate(knee, 0, 0, math.radians(z * 4))

    export_glb(path)


def monster_healer(path: Path) -> None:
    clear_scene()
    mats = monster_materials()["healer"]
    monster_core(mats)

    orb = add_uv_sphere("lantern_orb", (0.24, 1.03, 0.16), (0.11, 0.13, 0.11), mats["orb"], segments=12, rings=6)
    rotate(orb, 0, 0, 0)
    staff = add_cylinder("healer_staff", (0.32, 0.78, 0.17), 0.025, 0.86, mats["gold"], vertices=10)
    rotate(staff, 0, 0, math.radians(16))
    add_bevel(staff, 0.004)
    bead = add_uv_sphere("staff_bead", (0.45, 1.31, 0.18), (0.02, 0.03, 0.02), mats["orb"], segments=8, rings=4)
    cross = add_cube("cross", (0.32, 1.02, 0.22), (0.06, 0.01, 0.03), mats["dark"])
    rotate(cross, 0, 0, math.radians(45))
    add_cube("cross2", (0.32, 1.02, 0.16), (0.01, 0.01, 0.06), mats["dark"])
    export_glb(path)


def monster_jumper(path: Path) -> None:
    clear_scene()
    mats = monster_materials()["jumper"]
    monster_core(mats, scale=0.96)

    for i in range(4):
        coil = add_torus = add_cylinder(
            f"spring_{i}",
            (0.0, 0.12 + i * 0.06, 0.07 * (i - 1.5)),
            0.095,
            0.12,
            mats["spring"],
            vertices=16,
        )
        rotate(coil, math.radians(90), 0, 0)
        add_bevel(coil, 0.004)

    for i in range(2):
        foot = add_cube(f"spring_boot_{i}", (-0.03 + i * 0.12, 0.08, 0.0), (0.11, 0.05, 0.12), mats["plate"])
        rotate(foot, 0, 0, math.radians(-8 if i == 0 else 8))

    export_glb(path)


def monster_digger(path: Path) -> None:
    clear_scene()
    mats = monster_materials()["digger"]
    monster_core(mats, scale=0.95)

    shovel = add_cube("shovel", (0.28, 0.64, 0.0), (0.06, 0.42, 0.03), mats["shovel"])
    rotate(shovel, math.radians(75), 0, math.radians(-22))
    handle = add_cylinder("shovel_handle", (0.16, 0.95, 0.0), 0.026, 0.72, mats["metal"], vertices=8)
    rotate(handle, 0, 0, math.radians(88))

    helmet = add_cone("mining_helmet", (-0.03, 1.33, 0.0), 0.16, 0.0, 0.14, mats["metal"], vertices=16)
    rotate(helmet, 0, 0, 0)
    add_bevel(helmet, 0.004)

    pick = add_uv_sphere("pick", (0.24, 0.66, 0.0), (0.04, 0.02, 0.06), mats["dirt"], segments=10, rings=4)
    export_glb(path)


def monster_frostbite(path: Path) -> None:
    clear_scene()
    mats = monster_materials()["frostbite"]
    monster_core(mats)

    for i in range(7):
        back_spike = add_cone(
            f"ice_spike_{i}",
            (-0.16 + i * 0.06, 1.28, -0.16 + i * 0.053),
            0.035,
            0.0,
            0.19,
            mats["ice"],
            vertices=8,
            smooth=True,
        )
        rotate(back_spike, math.radians(12), 0, math.radians(i * 7))

    frost_cap = add_uv_sphere("ice_cap", (-0.01, 1.50, 0.0), (0.16, 0.12, 0.16), mats["skin"], segments=10, rings=6)
    add_subsurf(frost_cap, 1)
    add_bevel(frost_cap, 0.004)
    export_glb(path)


def monster_gargantuar(path: Path) -> None:
    clear_scene()
    mats = monster_materials()["gargantuar"]
    monster_core(mats, scale=1.38)

    club_shaft = add_cylinder("club_shaft", (0.62, 0.92, 0.0), 0.10, 1.06, mats["club"], vertices=14)
    rotate(club_shaft, 0, 0, math.radians(-28))
    add_bevel(club_shaft, 0.01)
    club_head = add_cube("club_head", (0.99, 1.16, 0.0), (0.16, 0.24, 0.14), mats["dark"])
    rotate(club_head, 0, 0, math.radians(-18))
    add_bevel(club_head, 0.008)

    shoulder_plug = add_cube("armor", (0.0, 1.02, 0.23), (0.28, 0.12, 0.10), mats["plate"])
    rotate(shoulder_plug, 0, 0, math.radians(6))
    add_cube("strap", (0.0, 0.82, 0.22), (0.26, 0.08, 0.09), mats["cloth"])
    export_glb(path)


def main() -> None:
    PLANT_DIR.mkdir(parents=True, exist_ok=True)
    MONSTER_DIR.mkdir(parents=True, exist_ok=True)

    plant_sprout_slinger(PLANT_DIR / "sprout-slinger.glb")
    plant_sunbloom(PLANT_DIR / "sunbloom.glb")
    plant_bark_bulwark(PLANT_DIR / "bark-bulwark.glb")
    plant_frost_sprout(PLANT_DIR / "frost-sprout.glb")
    plant_twin_pod(PLANT_DIR / "twin-pod.glb")
    plant_leaf_lobber(PLANT_DIR / "leaf-lobber.glb")
    plant_briar_mat(PLANT_DIR / "briar-mat.glb")
    plant_blast_berry(PLANT_DIR / "blast-berry.glb")
    plant_ember_stump(PLANT_DIR / "ember-stump.glb")
    plant_scent_root(PLANT_DIR / "scent-root.glb")

    monster_walker(MONSTER_DIR / "walker.glb")
    monster_conehead(MONSTER_DIR / "conehead.glb")
    monster_runner(MONSTER_DIR / "runner.glb")
    monster_buckethead(MONSTER_DIR / "buckethead.glb")
    monster_brute(MONSTER_DIR / "brute.glb")
    monster_healer(MONSTER_DIR / "healer.glb")
    monster_jumper(MONSTER_DIR / "jumper.glb")
    monster_digger(MONSTER_DIR / "digger.glb")
    monster_frostbite(MONSTER_DIR / "frostbite.glb")
    monster_gargantuar(MONSTER_DIR / "gargantuar.glb")


if __name__ == "__main__":
    main()
