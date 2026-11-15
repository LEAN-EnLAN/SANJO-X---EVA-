# Renders the passenger deck floor plan (public/img/deck-base.png) from src/screens/dashboard/deck-data.js.
# Top-down orthographic camera, one frame, Eevee (soft sun shadows + horizon-scan AO, ~1 min on CPU) or
# Workbench (seconds, flatter). Muted greys, transparent outside the hull so the image sits on the card.
# 50 px = 1 m, so pixel (x, y) here is exactly viewBox (x, y) in the React overlay (src/ds/deck-map.jsx).
#
#   node tools/deck-json.mjs > /tmp/deck.json
#   blender -b --factory-startup -noaudio -P tools/deck-base.blender.py -- /tmp/deck.json public/img/deck-base.png [scale] [eevee|workbench]
import bpy, bmesh, json, math, sys

argv = sys.argv[sys.argv.index("--") + 1:]
data = json.load(open(argv[0]))
OUT = argv[1]
SCALE = 1 if len(argv) < 3 else int(argv[2])  # render scale (1 = 1400 x 800)
ENGINE = "eevee" if len(argv) < 4 else argv[3]  # eevee (soft sun shadows + AO) or workbench (fastest)

D = data["DECK"]
PX = D["pxPerMetre"]
W, H = D["width"], D["height"]


def srgb(hexstr):
    h = hexstr.lstrip("#")
    c = [int(h[i:i + 2], 16) / 255 for i in (0, 2, 4)]
    lin = [v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4 for v in c]
    return (*lin, 1.0)


# muted greys on black; heights do the rest (shadow + cavity)
TONE = {
    "floor": "#121214", "grid": "#18181b", "corridor": "#1a1a1e", "shelter": "#16181b", "wet": "#151618",
    "hub": "#141518", "wall": "#7c7c84", "hull": "#55555c", "water": "#3a4650", "locker": "#2a2a2f",
    "counter": "#333338", "table": "#3c3c42", "seat": "#45454c", "seatback": "#34343a", "fixture": "#55555b",
    "machine": "#38383e", "appliance": "#46464c", "rack": "#2e2e34", "aed": "#9a9a9e", "mat": "#1e1e22",
    "tray": "#242428", "bench": "#36363c", "inset": "#1b1b1e", "chair": "#3a3a40", "jamb": "#7c7c84",
}


MATS = {}


def material(tone):
    if tone not in MATS:
        m = bpy.data.materials.new(tone)
        m.use_nodes = True
        bsdf = m.node_tree.nodes["Principled BSDF"]
        bsdf.inputs["Base Color"].default_value = srgb(TONE[tone])
        bsdf.inputs["Roughness"].default_value = 0.85 if tone != "water" else 0.45
        m.diffuse_color = srgb(TONE[tone])
        MATS[tone] = m
    return MATS[tone]


def to_bu(x, y):
    return (x / PX, -y / PX)


def rr_points(x, y, w, h, radii, seg=10):
    """rounded rectangle outline in px, clockwise from top-left; radii = (tl, tr, br, bl) in px"""
    tl, tr, br, bl = radii
    pts = []

    def arc(cx, cy, r, a0, a1):
        if r <= 0:
            pts.append((cx, cy))
            return
        for i in range(seg + 1):
            a = math.radians(a0 + (a1 - a0) * i / seg)
            pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))

    arc(x + tl, y + tl, tl, 180, 270) if tl else pts.append((x, y))
    arc(x + w - tr, y + tr, tr, 270, 360) if tr else pts.append((x + w, y))
    arc(x + w - br, y + h - br, br, 0, 90) if br else pts.append((x + w, y + h))
    arc(x + bl, y + h - bl, bl, 90, 180) if bl else pts.append((x, y + h))
    return pts


def prism(name, loops, z0, z1, tone):
    """closed (manifold) extruded polygon, so Workbench shadows stay clean"""
    loop = loops[0]
    clean = [p for i, p in enumerate(loop) if i == 0 or math.dist(p, loop[i - 1]) > 1e-3]
    if math.dist(clean[0], clean[-1]) < 1e-3:
        clean.pop()
    # make it counter-clockwise in Blender space (y is flipped) so normals face out
    area = sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(clean, clean[1:] + clean[:1]))
    if area < 0:
        clean.reverse()
    bm = bmesh.new()
    vs = [bm.verts.new((*to_bu(*p), z0)) for p in clean]
    face = bm.faces.new(vs)
    face.normal_update()
    if face.normal.z > 0:
        face.normal_flip()
    ext = bmesh.ops.extrude_face_region(bm, geom=[face])
    top = [g for g in ext["geom"] if isinstance(g, bmesh.types.BMVert)]
    bmesh.ops.translate(bm, verts=top, vec=(0, 0, max(z1 - z0, 0.0005)))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    ob.color = srgb(TONE[tone])
    me.materials.append(material(tone))
    bpy.context.collection.objects.link(ob)
    return ob


def box(name, x, y, w, h, z1, tone, r=0.0, z0=0.0):
    rad = min(w, h) / 2 * r
    return prism(name, [rr_points(x, y, w, h, (rad,) * 4)], z0, z1, tone)


def overlaps(a, b):
    return a["x"] < b["x"] + b["w"] and b["x"] < a["x"] + a["w"] and a["y"] < b["y"] + b["h"] and b["y"] < a["y"] + a["h"]


def cut(rect, holes):
    """split an axis-aligned wall rectangle around door openings (along its long axis)"""
    horiz = rect["w"] >= rect["h"]
    a0, a1 = (rect["x"], rect["x"] + rect["w"]) if horiz else (rect["y"], rect["y"] + rect["h"])
    spans = [(a0, a1)]
    for d in holes:
        if not overlaps(rect, d):
            continue
        d0, d1 = (d["x"], d["x"] + d["w"]) if horiz else (d["y"], d["y"] + d["h"])
        nxt = []
        for s0, s1 in spans:
            if d1 <= s0 or d0 >= s1:
                nxt.append((s0, s1))
                continue
            if d0 > s0:
                nxt.append((s0, d0))
            if d1 < s1:
                nxt.append((d1, s1))
        spans = nxt
    out = []
    for s0, s1 in spans:
        if s1 - s0 < 0.5:
            continue
        if horiz:
            out.append({"x": s0, "y": rect["y"], "w": s1 - s0, "h": rect["h"]})
        else:
            out.append({"x": rect["x"], "y": s0, "w": rect["w"], "h": s1 - s0})
    return out


# ---------- scene ----------
for ob in list(bpy.data.objects):
    bpy.data.objects.remove(ob)

hull = data["hull"]
hi, ho = hull["inner"], hull["outer"]


def inner_extent(axis, v):
    """[lo, hi] of the inner hull rounded rectangle along a grid line at x=v (axis 'x') or y=v"""
    r = hi["r"]
    if axis == "x":
        lo, hi_ = hi["y"], hi["y"] + hi["h"]
        d = max(hi["x"] + r - v, v - (hi["x"] + hi["w"] - r), 0)
    else:
        lo, hi_ = hi["x"], hi["x"] + hi["w"]
        d = max(hi["y"] + r - v, v - (hi["y"] + hi["h"] - r), 0)
    inset = r - math.sqrt(max(r * r - d * d, 0)) if d else 0
    return lo + inset, hi_ - inset


def corner_radii(rect):
    """round a floor zone's corner where it meets a rounded hull corner"""
    r = hi["r"]
    x0, y0, x1, y1 = rect["x"], rect["y"], rect["x"] + rect["w"], rect["y"] + rect["h"]
    L, T, R, B = hi["x"], hi["y"], hi["x"] + hi["w"], hi["y"] + hi["h"]
    return (r if (x0 <= L and y0 <= T) else 0, r if (x1 >= R and y0 <= T) else 0,
            r if (x1 >= R and y1 >= B) else 0, r if (x0 <= L and y1 >= B) else 0)


# deck plate
prism("deck", [rr_points(hi["x"], hi["y"], hi["w"], hi["h"], (hi["r"],) * 4, 16)], -0.06, 0.0, "floor")

# floor tints per zone
zone_tone = {"corridor": "corridor", "shelter": "shelter", "wet": "wet", "hub": "hub"}
for rm in data["rooms"]:
    t = zone_tone.get(rm["kind"])
    if t:
        prism("zone-" + rm["id"], [rr_points(rm["x"], rm["y"], rm["w"], rm["h"], corner_radii(rm))], 0.0, 0.01, t)

# 1 m floor grid, clipped to the hull
lw = 1.0
for gx in range(int(hi["x"]) + PX, int(hi["x"] + hi["w"]), PX):
    a, b = inner_extent("x", gx)
    box(f"gx{gx}", gx - lw / 2, a, lw, b - a, 0.014, "grid", z0=0.012)
for gy in range(int(hi["y"]) + PX, int(hi["y"] + hi["h"]), PX):
    a, b = inner_extent("y", gy)
    box(f"gy{gy}", a, gy - lw / 2, b - a, lw, 0.014, "grid", z0=0.012)

# pressure hull: two C-shaped halves, the hatches at both ends of the corridor are the gaps between them
SEG = 16
k = SEG + 1
out_pts = rr_points(ho["x"], ho["y"], ho["w"], ho["h"], (ho["r"],) * 4, SEG)
in_pts = rr_points(hi["x"], hi["y"], hi["w"], hi["h"], (hi["r"],) * 4, SEG)
gap_top = min(hz["y"] for hz in hull["hatches"])
gap_bot = max(hz["y"] + hz["h"] for hz in hull["hatches"])
ox0, ox1, ix0, ix1 = ho["x"], ho["x"] + ho["w"], hi["x"], hi["x"] + hi["w"]
upper = [(ox0, gap_top)] + out_pts[0:2 * k] + [(ox1, gap_top), (ix1, gap_top)] + in_pts[0:2 * k][::-1] + [(ix0, gap_top)]
lower = [(ox1, gap_bot)] + out_pts[2 * k:4 * k] + [(ox0, gap_bot), (ix0, gap_bot)] + in_pts[2 * k:4 * k][::-1] + [(ix1, gap_bot)]
prism("hull-upper", [upper], 0.0, 1.3, "hull")
prism("hull-lower", [lower], 0.0, 1.3, "hull")
for i, hz in enumerate(hull["hatches"]):
    # hatch frame: two jambs and a threshold
    box(f"hatch-sill{i}", hz["x"], hz["y"], hz["w"], hz["h"], 0.03, "corridor")
    box(f"hatch-j1{i}", hz["x"], hz["y"] - 6, hz["w"], 6, 1.4, "jamb")
    box(f"hatch-j2{i}", hz["x"], hz["y"] + hz["h"], hz["w"], 6, 1.4, "jamb")

# interior walls with door gaps
for i, wl in enumerate(data["walls"]):
    for j, seg in enumerate(cut(wl, data["doors"])):
        box(f"wall{i}-{j}", seg["x"], seg["y"], seg["w"], seg["h"], 1.15, "wall")

# storm shelter: thick walls built from water tank cells
sh = data["shelter"]
o, n, t = sh["outer"], sh["inner"], sh["thickness"]
gap = 3
sides = [
    {"x": o["x"], "y": o["y"], "w": o["w"], "h": t},                    # north (corridor side)
    {"x": o["x"], "y": o["y"] + o["h"] - t, "w": o["w"], "h": t},       # south (hull side)
    {"x": o["x"], "y": o["y"] + t, "w": t, "h": o["h"] - 2 * t},       # west
    {"x": o["x"] + o["w"] - t, "y": o["y"] + t, "w": t, "h": o["h"] - 2 * t},  # east
]
for si, side in enumerate(sides):
    for j, seg in enumerate(cut(side, [sh["door"]])):
        horiz = seg["w"] >= seg["h"]
        length = seg["w"] if horiz else seg["h"]
        cells = max(1, round(length / 46))
        step = length / cells
        for k in range(cells):
            if horiz:
                box(f"tank{si}-{j}-{k}", seg["x"] + k * step + gap / 2, seg["y"] + gap / 2, step - gap, seg["h"] - gap, 1.1, "water", r=0.15)
            else:
                box(f"tank{si}-{j}-{k}", seg["x"] + gap / 2, seg["y"] + k * step + gap / 2, seg["w"] - gap, step - gap, 1.1, "water", r=0.15)
d = sh["door"]
box("shelter-sill", d["x"], d["y"], d["w"], d["h"], 0.03, "jamb")

# emergency station niches: back plate, jambs, AED cabinet
for st in data["stations"]:
    nc = st["niche"]
    box(f"niche-back-{st['id']}", nc["x"], nc["y"], nc["w"], 6, 1.15, "wall")
    box(f"niche-j1-{st['id']}", nc["x"], nc["y"], 5, nc["h"], 1.15, "wall")
    box(f"niche-j2-{st['id']}", nc["x"] + nc["w"] - 5, nc["y"], 5, nc["h"], 1.15, "wall")
    box(f"aed-{st['id']}", nc["x"] + 8, nc["y"] + 8, nc["w"] - 16, nc["h"] - 12, 0.9, "aed", r=0.2)

# seats: base, backrest (aft side), armrests
for s in data["seats"]:
    cx, cy = s["x"], s["y"]
    box(f"seat-{s['id']}", cx - 26, cy - 27, 54, 54, 0.46, "seat", r=0.25)
    box(f"back-{s['id']}", cx - 30, cy - 27, 14, 54, 1.05, "seatback", r=0.4)
    box(f"arm1-{s['id']}", cx - 18, cy - 29, 40, 6, 0.64, "seatback", r=1)
    box(f"arm2-{s['id']}", cx - 18, cy + 23, 40, 6, 0.64, "seatback", r=1)

# galley stools around the table
for i, sx in enumerate((900, 955, 1010)):
    box(f"stool-a{i}", sx - 13, 166, 26, 26, 0.5, "chair", r=1)
    box(f"stool-b{i}", sx - 13, 264, 26, 26, 0.5, "chair", r=1)

for i, f in enumerate(data["furniture"]):
    box(f"f{i}-{f['tone']}", f["x"], f["y"], f["w"], f["h"], f["z"], f["tone"], r=f.get("r", 0.06))

# ---------- camera + workbench ----------
cam_data = bpy.data.cameras.new("cam")
cam_data.type = "ORTHO"
cam_data.ortho_scale = W / PX
cam = bpy.data.objects.new("cam", cam_data)
cam.location = (W / 2 / PX, -H / 2 / PX, 30)
bpy.context.collection.objects.link(cam)

sc = bpy.context.scene
sc.camera = cam
sc.render.resolution_x = W * SCALE
sc.render.resolution_y = H * SCALE
sc.render.resolution_percentage = 100
sc.render.film_transparent = True
sc.render.image_settings.file_format = "PNG"
sc.render.image_settings.color_mode = "RGBA"
sc.render.filepath = OUT
sc.view_settings.view_transform = "Standard"
sc.view_settings.look = "None"

if ENGINE == "eevee":
    sc.render.engine = "BLENDER_EEVEE_NEXT" if "BLENDER_EEVEE_NEXT" in {e.identifier for e in bpy.types.RenderSettings.bl_rna.properties["engine"].enum_items} else "BLENDER_EEVEE"
    ee = sc.eevee
    ee.taa_render_samples = 32
    ee.use_shadows = True
    ee.shadow_ray_count = 2
    ee.shadow_step_count = 6
    ee.use_fast_gi = True          # horizon scan: the ambient occlusion in corners and under furniture
    ee.fast_gi_distance = 0.8
    # a soft key light from the upper left (aft, port) and a dim sky fill
    sun = bpy.data.lights.new("sun", "SUN")
    sun.energy = 1.5
    sun.angle = math.radians(14)
    sun_ob = bpy.data.objects.new("sun", sun)
    sun_ob.rotation_euler = (math.radians(24), math.radians(-14), math.radians(-30))
    bpy.context.collection.objects.link(sun_ob)
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (1, 1, 1, 1)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.12
    sc.world = world
else:
    sc.render.engine = "BLENDER_WORKBENCH"
    sc.display.render_aa = "16"
    sh_ = sc.display.shading
    sh_.light = "STUDIO"
    sh_.color_type = "OBJECT"
    sh_.show_cavity = True
    sh_.cavity_type = "BOTH"
    sh_.show_specular_highlight = False
    sc.display.matcap_ssao_distance = 1.2
    sc.display.matcap_ssao_samples = 24

bpy.ops.render.render(write_still=True)
print("wrote", OUT)
