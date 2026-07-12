#!/usr/bin/env python3
"""Generate low-poly Bevy Open Siege GLB unit models with Blender.

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
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def material(name: str, color: tuple[float, float, float, float]) -> bpy.types.Material:
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = color
    return mat


def add_uv_sphere(
    name: str,
    loc: tuple[float, float, float],
    scale: tuple[float, float, float],
    mat: bpy.types.Material,
    segments: int = 16,
    rings: int = 8,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=rings, location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    obj.data.materials.append(mat)
    return obj


def add_cube(
    name: str,
    loc: tuple[float, float, float],
    scale: tuple[float, float, float],
    mat: bpy.types.Material,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cube_add(location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    obj.data.materials.append(mat)
    return obj


def add_cylinder(
    name: str,
    loc: tuple[float, float, float],
    radius: float,
    depth: float,
    mat: bpy.types.Material,
    vertices: int = 16,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.data.materials.append(mat)
    return obj


def add_cone(
    name: str,
    loc: tuple[float, float, float],
    radius1: float,
    radius2: float,
    depth: float,
    mat: bpy.types.Material,
    vertices: int = 16,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cone_add(
        vertices=vertices, radius1=radius1, radius2=radius2, depth=depth, location=loc
    )
    obj = bpy.context.object
    obj.name = name
    obj.data.materials.append(mat)
    return obj


def rotate(obj: bpy.types.Object, x: float = 0.0, y: float = 0.0, z: float = 0.0) -> None:
    obj.rotation_euler = (x, y, z)


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


def plant_common_mats() -> dict[str, bpy.types.Material]:
    return {
        "stem": material("stem_leaf_green", (0.18, 0.52, 0.16, 1.0)),
        "leaf": material("leaf_green", (0.28, 0.72, 0.20, 1.0)),
        "dark_leaf": material("dark_leaf", (0.10, 0.35, 0.12, 1.0)),
        "pod": material("pod_green", (0.42, 0.80, 0.26, 1.0)),
        "yellow": material("sun_yellow", (0.98, 0.78, 0.18, 1.0)),
        "orange": material("warm_orange", (0.94, 0.34, 0.10, 1.0)),
        "blue": material("frost_blue", (0.42, 0.82, 0.98, 1.0)),
        "wood": material("bark_brown", (0.46, 0.27, 0.12, 1.0)),
        "thorn": material("thorn", (0.12, 0.20, 0.08, 1.0)),
        "cream": material("cream", (0.82, 0.76, 0.52, 1.0)),
    }


def add_leaf_pair(mats: dict[str, bpy.types.Material], y: float = 0.22) -> None:
    left = add_uv_sphere("left_leaf", (-0.22, y, 0.02), (0.24, 0.08, 0.13), mats["leaf"])
    right = add_uv_sphere("right_leaf", (0.22, y, 0.02), (0.24, 0.08, 0.13), mats["leaf"])
    rotate(left, 0.0, 0.0, math.radians(18))
    rotate(right, 0.0, 0.0, math.radians(-18))


def plant_sprout_slinger(path: Path) -> None:
    clear_scene()
    mats = plant_common_mats()
    add_cylinder("stem", (0, 0.42, 0), 0.07, 0.65, mats["stem"])
    add_leaf_pair(mats)
    add_uv_sphere("head", (0.02, 0.84, 0), (0.24, 0.20, 0.22), mats["pod"])
    muzzle = add_cylinder("muzzle", (0.34, 0.84, 0), 0.10, 0.24, mats["pod"])
    rotate(muzzle, 0, math.radians(90), 0)
    add_uv_sphere("dark_mouth", (0.47, 0.84, 0), (0.06, 0.06, 0.06), mats["dark_leaf"])
    export_glb(path)


def plant_sunbloom(path: Path) -> None:
    clear_scene()
    mats = plant_common_mats()
    add_cylinder("stem", (0, 0.36, 0), 0.06, 0.55, mats["stem"])
    add_leaf_pair(mats, 0.20)
    for i in range(12):
        angle = i / 12.0 * math.tau
        petal = add_uv_sphere(
            f"petal_{i}",
            (math.cos(angle) * 0.22, 0.84 + math.sin(angle) * 0.22, 0),
            (0.08, 0.18, 0.035),
            mats["yellow"],
            12,
            6,
        )
        rotate(petal, 0, 0, angle)
    add_uv_sphere("face", (0, 0.84, 0.02), (0.20, 0.20, 0.06), mats["orange"], 16, 8)
    export_glb(path)


def plant_bark_bulwark(path: Path) -> None:
    clear_scene()
    mats = plant_common_mats()
    add_uv_sphere("shell", (0, 0.55, 0), (0.38, 0.52, 0.32), mats["wood"], 16, 8)
    add_uv_sphere("cap", (0, 1.08, 0), (0.32, 0.08, 0.28), mats["cream"], 16, 6)
    for x in [-0.15, 0.15]:
        add_uv_sphere("brow", (x, 0.70, 0.27), (0.08, 0.04, 0.025), mats["dark_leaf"], 8, 4)
    export_glb(path)


def plant_frost_sprout(path: Path) -> None:
    clear_scene()
    mats = plant_common_mats()
    add_cylinder("stem", (0, 0.42, 0), 0.06, 0.66, mats["blue"])
    add_leaf_pair(mats)
    add_uv_sphere("frost_head", (0, 0.84, 0), (0.25, 0.20, 0.22), mats["blue"])
    nozzle = add_cylinder("ice_nozzle", (0.35, 0.84, 0), 0.09, 0.24, mats["blue"])
    rotate(nozzle, 0, math.radians(90), 0)
    for i in range(5):
        spike = add_cone(
            f"ice_spike_{i}",
            (-0.15 + i * 0.08, 1.05, 0),
            0.035,
            0.0,
            0.18,
            mats["cream"],
            8,
        )
        rotate(spike, 0, 0, math.radians(-12 + i * 6))
    export_glb(path)


def plant_twin_pod(path: Path) -> None:
    clear_scene()
    mats = plant_common_mats()
    add_cylinder("stem", (0, 0.40, 0), 0.07, 0.62, mats["stem"])
    add_leaf_pair(mats)
    for z in [-0.13, 0.13]:
        add_uv_sphere("head", (0.04, 0.84, z), (0.22, 0.18, 0.18), mats["pod"])
        muzzle = add_cylinder("muzzle", (0.34, 0.84, z), 0.08, 0.22, mats["pod"])
        rotate(muzzle, 0, math.radians(90), 0)
    export_glb(path)


def plant_leaf_lobber(path: Path) -> None:
    clear_scene()
    mats = plant_common_mats()
    add_cylinder("base", (0, 0.22, 0), 0.18, 0.30, mats["stem"])
    arm = add_cylinder("lobber_arm", (0.12, 0.62, 0), 0.07, 0.72, mats["leaf"])
    rotate(arm, 0, 0, math.radians(-32))
    add_uv_sphere("basket", (0.36, 0.88, 0), (0.20, 0.12, 0.18), mats["pod"])
    add_uv_sphere("cabbage", (0.48, 1.02, 0), (0.15, 0.13, 0.13), mats["cream"])
    export_glb(path)


def plant_briar_mat(path: Path) -> None:
    clear_scene()
    mats = plant_common_mats()
    add_cube("mat", (0, 0.08, 0), (0.48, 0.06, 0.32), mats["dark_leaf"])
    for x in [-0.30, -0.15, 0.0, 0.15, 0.30]:
        for z in [-0.18, 0.0, 0.18]:
            spike = add_cone("thorn", (x, 0.23, z), 0.045, 0.0, 0.24, mats["thorn"], 8)
            rotate(spike, 0, 0, math.radians(0))
    export_glb(path)


def plant_blast_berry(path: Path) -> None:
    clear_scene()
    mats = plant_common_mats()
    add_uv_sphere("berry", (0, 0.45, 0), (0.34, 0.34, 0.34), mats["orange"], 16, 8)
    add_cone("fuse", (0.05, 0.86, 0), 0.035, 0.015, 0.30, mats["wood"], 8)
    add_uv_sphere("spark", (0.11, 1.05, 0), (0.08, 0.08, 0.08), mats["yellow"], 8, 4)
    export_glb(path)


def plant_ember_stump(path: Path) -> None:
    clear_scene()
    mats = plant_common_mats()
    add_cylinder("stump", (0, 0.44, 0), 0.26, 0.78, mats["wood"], 18)
    add_uv_sphere("ember", (0.08, 0.88, 0.03), (0.15, 0.08, 0.12), mats["orange"], 12, 6)
    for x in [-0.13, 0.13]:
        flame = add_cone("flame", (x, 1.05, 0), 0.08, 0.0, 0.28, mats["yellow"], 10)
        rotate(flame, 0, 0, math.radians(x * 45))
    export_glb(path)


def plant_scent_root(path: Path) -> None:
    clear_scene()
    mats = plant_common_mats()
    add_uv_sphere("bulb", (0, 0.42, 0), (0.28, 0.34, 0.24), mats["cream"], 16, 8)
    for i, angle in enumerate([-35, -18, 0, 18, 35]):
        root = add_cylinder(f"root_{i}", (0.0, 0.13, (i - 2) * 0.08), 0.035, 0.55, mats["wood"], 8)
        rotate(root, 0, math.radians(90), math.radians(angle))
    add_leaf_pair(mats, 0.70)
    export_glb(path)


def monster_mats() -> dict[str, bpy.types.Material]:
    return {
        "skin": material("moss_undead_skin", (0.37, 0.48, 0.25, 1.0)),
        "dark": material("dark_moss", (0.12, 0.16, 0.10, 1.0)),
        "cloth": material("torn_cloth", (0.44, 0.40, 0.34, 1.0)),
        "cone": material("traffic_cone", (0.93, 0.36, 0.10, 1.0)),
        "bucket": material("dull_bucket", (0.46, 0.48, 0.48, 1.0)),
        "frost": material("frostbite_blue", (0.50, 0.80, 0.94, 1.0)),
        "purple": material("healer_purple", (0.40, 0.22, 0.55, 1.0)),
    }


def base_monster(mats: dict[str, bpy.types.Material], scale: float = 1.0) -> None:
    add_uv_sphere("torso", (0, 0.70 * scale, 0), (0.24 * scale, 0.36 * scale, 0.15 * scale), mats["skin"])
    add_uv_sphere("head", (0.02, 1.14 * scale, 0), (0.18 * scale, 0.18 * scale, 0.15 * scale), mats["skin"])
    for x in [-0.16, 0.16]:
        arm = add_cylinder("arm", (x, 0.70 * scale, 0), 0.045 * scale, 0.55 * scale, mats["skin"], 10)
        rotate(arm, math.radians(12), 0, math.radians(25 if x < 0 else -25))
    for x in [-0.10, 0.12]:
        leg = add_cylinder("leg", (x, 0.25 * scale, 0), 0.055 * scale, 0.50 * scale, mats["dark"], 10)
        rotate(leg, math.radians(5), 0, math.radians(-8 if x < 0 else 8))
    add_cube("ragged_shirt", (0, 0.68 * scale, 0.01), (0.25 * scale, 0.20 * scale, 0.08 * scale), mats["cloth"])


def monster_walker(path: Path) -> None:
    clear_scene()
    base_monster(monster_mats())
    export_glb(path)


def monster_conehead(path: Path) -> None:
    clear_scene()
    mats = monster_mats()
    base_monster(mats)
    add_cone("cone_hat", (0.02, 1.42, 0), 0.17, 0.04, 0.34, mats["cone"], 16)
    export_glb(path)


def monster_runner(path: Path) -> None:
    clear_scene()
    mats = monster_mats()
    base_monster(mats, 0.92)
    add_uv_sphere("runner_badge", (0.05, 0.76, 0.13), (0.07, 0.04, 0.02), mats["cone"], 8, 4)
    export_glb(path)


def monster_buckethead(path: Path) -> None:
    clear_scene()
    mats = monster_mats()
    base_monster(mats)
    add_cylinder("bucket_helmet", (0.02, 1.32, 0), 0.18, 0.24, mats["bucket"], 18)
    export_glb(path)


def monster_brute(path: Path) -> None:
    clear_scene()
    mats = monster_mats()
    base_monster(mats, 1.18)
    add_cube("shoulders", (0, 0.98, 0), (0.42, 0.10, 0.16), mats["dark"])
    export_glb(path)


def monster_healer(path: Path) -> None:
    clear_scene()
    mats = monster_mats()
    base_monster(mats)
    add_uv_sphere("healing_orb", (0.28, 1.00, 0), (0.10, 0.10, 0.10), mats["purple"], 12, 6)
    add_cylinder("staff", (0.34, 0.67, 0), 0.025, 0.82, mats["dark"], 8)
    export_glb(path)


def monster_jumper(path: Path) -> None:
    clear_scene()
    mats = monster_mats()
    base_monster(mats, 0.96)
    spring = add_cylinder("spring_legs", (0, 0.18, 0), 0.16, 0.12, mats["bucket"], 16)
    rotate(spring, math.radians(90), 0, 0)
    export_glb(path)


def monster_digger(path: Path) -> None:
    clear_scene()
    mats = monster_mats()
    base_monster(mats, 0.95)
    shovel = add_cube("shovel", (0.34, 0.52, 0), (0.06, 0.34, 0.03), mats["bucket"])
    rotate(shovel, 0, 0, math.radians(-24))
    add_cylinder("handle", (0.25, 0.80, 0), 0.025, 0.72, mats["dark"], 8)
    export_glb(path)


def monster_frostbite(path: Path) -> None:
    clear_scene()
    mats = monster_mats()
    base_monster(mats)
    for i in range(5):
        spike = add_cone("ice_back", (-0.16 + i * 0.08, 1.36, -0.05), 0.035, 0.0, 0.20, mats["frost"], 8)
        rotate(spike, math.radians(12), 0, math.radians(-16 + i * 8))
    export_glb(path)


def monster_gargantuar(path: Path) -> None:
    clear_scene()
    mats = monster_mats()
    base_monster(mats, 1.38)
    club = add_cylinder("club", (0.46, 0.85, 0), 0.08, 0.95, mats["dark"], 10)
    rotate(club, 0, 0, math.radians(-32))
    add_cube("club_head", (0.67, 1.18, 0), (0.13, 0.18, 0.13), mats["wood"] if "wood" in mats else mats["dark"])
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
