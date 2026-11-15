#!/usr/bin/env python3
"""Eva by WearMeds - Codex kit v2: black (#000) and white (#FFF) asset variants with GPT Image 2.5.

Setup:   pip install openai pillow        (and export OPENAI_API_KEY)
Drafts:  python generate.py --model flare                  -> drafts/   (fast, cheap review pass)
Finals:  python generate.py --model sunburst               -> out/
Subset:  python generate.py --variant white --ids S1,OR0   (S1 = S1-B and S1-W)
Prompts: python generate.py --dry-run [--emit prompts-final.jsonl]
Post:    python generate.py --post-only --upscale --qa --derive --sheet --zip   (no API calls)

Check the current OpenAI Images API docs before the first run. Model ids, sizes and parameter
names may differ from this script: fix "api" / "models" in prompts.json first, code second.
If you generate with a built-in image tool instead, save each raw image to <root>/_raw/<file>
and run --post-only: it applies edge lock, watermark, shadows and checks exactly the same way.
"""
import argparse, base64, datetime, json, os, sys, time, zipfile
from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont, ImageStat

HERE = os.path.dirname(os.path.abspath(__file__))
CFG = json.load(open(os.path.join(HERE, "prompts.json"), encoding="utf-8"))
BY_ID = {a["id"]: a for a in CFG["assets"]}
FINAL, DRAFT = os.path.join(HERE, "out"), os.path.join(HERE, "drafts")
PAGE = {"#000000": (0, 0, 0), "#FFFFFF": (255, 255, 255)}


def args_():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--variant", choices=["black", "white", "both"], default="both",
                   help="shared transparent layers are always included")
    p.add_argument("--model", choices=["flare", "sunburst"], default="sunburst",
                   help="flare = fast drafts into drafts/, sunburst = finals into out/")
    p.add_argument("--ids", default="", help="comma list of ids or base ids")
    p.add_argument("--root", help="override output root")
    p.add_argument("--force", action="store_true", help="regenerate files that already exist")
    p.add_argument("--note", default="", help="text appended to manifest notes of the selected ids (log retry reasons)")
    p.add_argument("--dry-run", action="store_true", help="print assembled prompts, no API calls")
    p.add_argument("--emit", help="write assembled prompts to this JSONL file")
    p.add_argument("--post-only", action="store_true", help="skip API; post-process files in <root>/_raw")
    p.add_argument("--upscale", action="store_true",
                   help="resize to each asset's 'target' (Lanczos stopgap; prefer native API size or a real upscaler)")
    p.add_argument("--derive", action="store_true", help="build page-colour chips/inventory squares from 'derived'")
    p.add_argument("--qa", action="store_true", help="build qa/ alignment composites for layer sets")
    p.add_argument("--sheet", action="store_true", help="build contact-sheet-black.png and contact-sheet-white.png")
    p.add_argument("--zip", action="store_true", help="package <root> as eva-assets-v2.zip")
    return p.parse_args()


def selected(a, args):
    if args.variant != "both" and a["variant"] not in ("shared", args.variant):
        return False
    ids = {s.strip() for s in args.ids.split(",") if s.strip()}
    return not ids or a["id"] in ids or a.get("base") in ids


def assemble(a):
    parts = [a["prompt"]]
    for d in a.get("design", []):
        parts.append(CFG["designs"][d])
    if a.get("camera"):
        parts.append("Camera: " + CFG["cameras"][a["camera"]].replace("{yaw}", str(a.get("yaw", 0))))
    if a.get("refs"):
        parts.append("Reference images are attached in the order they are mentioned (reference 1, reference 2).")
    if CFG["style"].get(a["style"]):
        parts.append(CFG["style"][a["style"]])
    parts.append(CFG["rules"])
    return " ".join(parts)


def resolve(ref, root):
    if ref in BY_ID:
        for r in (root, FINAL):
            path = os.path.join(r, BY_ID[ref]["file"])
            if os.path.exists(path):
                return path
        raise FileNotFoundError(f"reference {ref} not generated yet ({BY_ID[ref]['file']}); generate it first")
    path = os.path.join(HERE, ref)
    if not os.path.exists(path):
        raise FileNotFoundError(f"reference file missing: {ref}")
    return path


def call_api(a, model_id, prompt, refs):
    from openai import OpenAI  # imported here so --dry-run/--post-only work without the SDK
    client, api = OpenAI(), CFG["api"]
    kw = {"model": model_id, "prompt": prompt, "size": a["size"], "n": 1, **api["common"],
          "background": "transparent" if a["background"] == "transparent" else "opaque"}
    if refs:
        files = [open(p, "rb") for p in refs]
        try:
            r = client.images.edit(image=files, **kw, **api["edit_extra"])
        finally:
            for f in files:
                f.close()
    else:
        r = client.images.generate(**kw, **api["generate_extra"])
    return base64.b64decode(r.data[0].b64_json), getattr(r, "model", None) or model_id


# ---------------------------------------------------------------- post-processing
def edge_ring(size, frac):
    W, H = size
    n = max(1, int(frac * min(W, H)))
    m = Image.new("L", size, 255)
    ImageDraw.Draw(m).rectangle([n, n, W - 1 - n, H - 1 - n], fill=0)
    return m


def edge_lock(im, rgb):
    """Measure how far the outer ring is from the page colour, then blend the outer band into it exactly."""
    im = im.convert("RGB")
    W, H = im.size
    solid = Image.new("RGB", im.size, rgb)
    diff = ImageChops.difference(im, solid).convert("L")
    dev = round(ImageStat.Stat(diff, mask=edge_ring(im.size, 0.02)).mean[0], 2)
    n = max(1, int(CFG["post"]["edge_band"] * min(W, H)))
    mask = Image.new("L", im.size, 0)
    d = ImageDraw.Draw(mask)
    for i in range(n):  # smoothstep falloff from the frame edge inward
        t = i / n
        d.rectangle([i, i, W - 1 - i, H - 1 - i], outline=int(255 * (1 - t * t * (3 - 2 * t))))
    out = Image.composite(solid, im, mask)
    c = CFG["post"]["clamp"]  # snap near-page values so no visible box appears on the page
    lut = [0 if v < c else v for v in range(256)] if rgb == (0, 0, 0) else [255 if v > 255 - c else v for v in range(256)]
    return out.point(lut * 3), dev


def font(size):
    for name in ("DejaVuSans.ttf", "Arial.ttf", "Helvetica.ttc", "/System/Library/Fonts/Helvetica.ttc",
                 "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", "C:/Windows/Fonts/arial.ttf"):
        try:
            return ImageFont.truetype(name, size)
        except OSError:
            pass
    try:
        return ImageFont.load_default(size=size)
    except TypeError:
        return ImageFont.load_default()


def watermark(im, rgb):
    """Visible 'AI-generated' mark, bottom-right, ~2% of height, 60% opacity, contrasting with the page."""
    im = im.convert("RGBA")
    W, H = im.size
    layer = Image.new("RGBA", im.size, (0, 0, 0, 0))
    d, f, text = ImageDraw.Draw(layer), font(max(10, int(H * 0.02))), "AI-generated"
    x0, y0, x1, y1 = d.textbbox((0, 0), text, font=f)
    m = int(H * 0.025)
    col = (255, 255, 255, 153) if rgb == (0, 0, 0) else (0, 0, 0, 153)
    d.text((W - m - (x1 - x0) - x0, H - m - (y1 - y0) - y0), text, font=f, fill=col)
    return Image.alpha_composite(im, layer).convert("RGB")


def make_shadow(src_path, kind):
    """Transparent black contact/float shadow from the source layer's alpha (same canvas, stack under it)."""
    im = Image.open(src_path).convert("RGBA")
    W, H = im.size
    box = im.getchannel("A").point(lambda v: 255 if v > 24 else 0).getbbox()
    if not box:
        raise ValueError(f"{src_path} has an empty alpha channel")
    x0, y0, x1, y1 = box
    cx, bw = (x0 + x1) / 2, x1 - x0
    base = min(H - 4, y1 + (int(0.04 * H) if kind == "float" else 0))
    k = 0.6 if kind == "float" else 1.0
    wide, core = Image.new("L", im.size, 0), Image.new("L", im.size, 0)
    rw, rh = bw * 0.50, max(4, H * 0.035)
    ImageDraw.Draw(wide).ellipse([cx - rw, base - rh, cx + rw, base + rh], fill=int(255 * 0.22 * k))
    cw, ch = bw * 0.34, max(2, H * 0.010)
    ImageDraw.Draw(core).ellipse([cx - cw, base - ch, cx + cw, base + ch], fill=int(255 * 0.45 * k))
    alpha = ImageChops.lighter(wide.filter(ImageFilter.GaussianBlur(W * 0.03)),
                               core.filter(ImageFilter.GaussianBlur(W * (0.02 if kind == "float" else 0.008))))
    out = Image.new("RGBA", im.size, (0, 0, 0, 0))
    out.putalpha(alpha)
    return out


def fit_target(im, target):
    """Centre-crop to the target aspect ratio, then resize to the target size."""
    tw, th = (int(v) for v in target.split()[0].split("x"))
    W, H = im.size
    if abs(W / H - tw / th) > 0.01:
        if W / H > tw / th:
            nw = int(H * tw / th)
            im = im.crop(((W - nw) // 2, 0, (W - nw) // 2 + nw, H))
        else:
            nh = int(W * th / tw)
            im = im.crop((0, (H - nh) // 2, W, (H - nh) // 2 + nh))
    return im if im.size == (tw, th) else im.resize((tw, th), Image.LANCZOS)


def postprocess(a, raw, final, upscale=False):
    post, checks = [], {}
    im = Image.open(raw)
    if upscale and a.get("target"):
        im = fit_target(im, a["target"])
        post.append("fit_target " + a["target"])
    if a["background"] != "transparent":
        rgb = PAGE.get(a["background"], (0, 0, 0))
        im = im.convert("RGB")
        if a["background"] in PAGE and a.get("edge_lock", True):
            im, dev = edge_lock(im, rgb)
            checks["edge_dev_before"] = dev
            if dev > CFG["post"]["edge_dev_max"]:
                checks["flag"] = "edges far from page colour before lock; consider regenerating (background must fall off)"
            post.append("edge_lock")
        if a.get("watermark"):
            im = watermark(im, rgb)
            post.append("watermark")
    else:
        im = im.convert("RGBA")
        W, H = im.size
        corners = [im.getpixel(p)[3] for p in ((0, 0), (W - 1, 0), (0, H - 1), (W - 1, H - 1))]
        checks["transparent_ok"] = max(corners) == 0
        if not checks["transparent_ok"]:
            checks["flag"] = "corners not transparent; background must be alpha"
    os.makedirs(os.path.dirname(final), exist_ok=True)
    im.save(final)
    return post, checks


# ---------------------------------------------------------------- QA, sheets, zip
def checker(size, s=16):
    im = Image.new("RGB", size, (200, 200, 200))
    d = ImageDraw.Draw(im)
    for y in range(0, size[1], s):
        for x in range((y // s) % 2 * s, size[0], 2 * s):
            d.rectangle([x, y, x + s - 1, y + s - 1], fill=(235, 235, 235))
    return im.convert("RGBA")


def load(root, aid):
    p = os.path.join(root, BY_ID[aid]["file"])
    return Image.open(p).convert("RGBA") if os.path.exists(p) else None


def qa(root):
    os.makedirs(os.path.join(root, "qa"), exist_ok=True)
    for set_name in sorted({a["set"] for a in CFG["assets"] if a.get("set")}):
        members = [a for a in CFG["assets"] if a.get("set") == set_name]
        anchor = next((load(root, a["id"]) for a in members if a.get("role") == "anchor"), None)
        imgs = [(a, load(root, a["id"])) for a in members]
        imgs = [(a, i) for a, i in imgs if i is not None]
        if not imgs:
            continue
        tiles = []
        if anchor is not None:
            comp = Image.new("RGBA", anchor.size, (0, 0, 0, 0))
            for a, i in imgs:
                if a.get("role") == "part" and i.size == anchor.size:
                    comp = Image.alpha_composite(comp, i)
            ma = anchor.getchannel("A").point(lambda v: 255 if v > 24 else 0)
            mc = comp.getchannel("A").point(lambda v: 255 if v > 24 else 0)
            inter = ImageStat.Stat(ImageChops.multiply(ma, mc)).sum[0]
            union = ImageStat.Stat(ImageChops.lighter(ma, mc)).sum[0] or 1
            print(f"QA {set_name}: alpha IoU parts vs anchor = {inter / union:.3f} (aim > 0.90)")
            tiles += [Image.alpha_composite(checker(anchor.size), anchor), Image.alpha_composite(checker(anchor.size), comp)]
            for page, rgb in (("black", (0, 0, 0, 255)), ("white", (255, 255, 255, 255))):
                t = Image.alpha_composite(Image.new("RGBA", anchor.size, rgb), anchor)
                for a, i in imgs:
                    if a.get("role") == "glow" and a["variant"] in ("shared", page) and i.size == anchor.size:
                        t = Image.alpha_composite(t, i)
                tiles.append(t)
        else:
            tiles = [Image.alpha_composite(checker(i.size), i) for _, i in imgs]
        h = 360
        tiles = [t.resize((max(1, int(t.width * h / t.height)), h)) for t in tiles]
        sheet = Image.new("RGB", (sum(t.width for t in tiles) + 10 * (len(tiles) + 1), h + 20), (128, 128, 128))
        x = 10
        for t in tiles:
            sheet.paste(t.convert("RGB"), (x, 10))
            x += t.width + 10
        sheet.save(os.path.join(root, "qa", f"{set_name}.png"))


def sheet(root):
    for page, rgb, ink in (("black", (0, 0, 0), (150, 150, 150)), ("white", (255, 255, 255), (110, 110, 110))):
        items = [a for a in CFG["assets"] if a["variant"] in ("shared", page)
                 and os.path.exists(os.path.join(root, a["file"]))]
        if not items:
            continue
        cols, tw, th = 8, 240, 200
        rows = (len(items) + cols - 1) // cols
        im = Image.new("RGB", (cols * tw, rows * (th + 28)), rgb)
        d, f = ImageDraw.Draw(im), font(14)
        for n, a in enumerate(items):
            src = Image.open(os.path.join(root, a["file"])).convert("RGBA")
            src.thumbnail((tw - 16, th - 16))
            x, y = (n % cols) * tw, (n // cols) * (th + 28)
            im.paste(src, (x + (tw - src.width) // 2, y + (th - src.height) // 2), src)
            d.text((x + 8, y + th + 4), a["id"], font=f, fill=ink)
        im.save(os.path.join(root, f"contact-sheet-{page}.png"))


def derive(root):
    d, margin = CFG["derived"], CFG["derived"]["margin"]
    for item in d["items"]:
        for page, ids in item["pages"].items():
            layers = [load(root, i) for i in ids]
            if any(l is None for l in layers) or len({l.size for l in layers}) != 1:
                print("derive skip:", item["name"], page, "(missing source or size mismatch)")
                continue
            comp = Image.new("RGBA", layers[0].size, (0, 0, 0, 0))
            for l in layers:
                comp = Image.alpha_composite(comp, l)
            comp = comp.crop(comp.getchannel("A").point(lambda v: 255 if v > 8 else 0).getbbox())
            N = item["size"]
            inner = int(N * (1 - 2 * margin))
            k = min(inner / comp.width, inner / comp.height)
            comp = comp.resize((max(1, int(comp.width * k)), max(1, int(comp.height * k))), Image.LANCZOS)
            out = Image.new("RGBA", (N, N), PAGE["#000000" if page == "black" else "#FFFFFF"] + (255,))
            out.alpha_composite(comp, ((N - comp.width) // 2, (N - comp.height) // 2))
            path = os.path.join(root, page, "chips", item["name"] + ".png")
            os.makedirs(os.path.dirname(path), exist_ok=True)
            out.convert("RGB").save(path)
            print("derived", path)


def make_zip(root):
    path = os.path.join(HERE, "eva-assets-v2.zip")
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for folder, _, files in os.walk(root):
            if "_raw" in os.path.relpath(folder, root).split(os.sep):
                continue
            for fn in files:
                full = os.path.join(folder, fn)
                z.write(full, os.path.relpath(full, root))
        z.write(os.path.join(HERE, "AI-NOTICE.md"), "AI-NOTICE.md")
    print("zip:", path)


# ---------------------------------------------------------------- main
def main():
    args = args_()
    root = args.root or (DRAFT if args.model == "flare" else FINAL)
    model_id = CFG["models"][args.model]
    mpath = os.path.join(root, "manifest.json")
    manifest = json.load(open(mpath, encoding="utf-8")) if os.path.exists(mpath) else {
        "kit": CFG["kit"], "orchestrator_model": CFG["models"]["orchestrator"], "assets": {}}
    emit = open(args.emit, "w", encoding="utf-8") if args.emit else None

    for a in CFG["assets"]:
        if not selected(a, args):
            continue
        final, raw = os.path.join(root, a["file"]), os.path.join(root, "_raw", a["file"])
        entry = manifest["assets"].get(a["id"], {"retries": 0, "notes": ""})
        prompt = assemble(a)

        if a.get("engine") == "local-shadow":
            if args.dry_run:
                print(f"[{a['id']}] local shadow from {a['source']}")
                continue
            try:
                src = resolve(a["source"], root)
            except FileNotFoundError as e:
                print(a["id"], "skip:", e)
                continue
            os.makedirs(os.path.dirname(final), exist_ok=True)
            make_shadow(src, a["shadow"]).save(final)
            entry.update({"id": a["id"], "variant": a["variant"], "file": a["file"], "model": "local (Pillow)",
                          "source": a["source"], "background": "transparent", "prompt_final": a["prompt"]})
            manifest["assets"][a["id"]] = entry
            print("ok", a["id"], a["file"])
            continue

        if args.dry_run or emit:
            rec = {"id": a["id"], "file": a["file"], "size": a["size"], "background": a["background"],
                   "refs": a.get("refs", []), "prompt": prompt}
            if emit:
                emit.write(json.dumps(rec, ensure_ascii=False) + "\n")
            if args.dry_run:
                print(f"\n[{a['id']}] {a['file']} {a['size']} bg={a['background']} refs={rec['refs']}\n{prompt}")
                continue

        if not args.post_only and (args.force or not os.path.exists(raw)):
            try:
                refs = [resolve(r, root) for r in a.get("refs", [])]
            except FileNotFoundError as e:
                print(a["id"], "skip:", e)
                continue
            if os.path.exists(raw):
                entry["retries"] = entry.get("retries", 0) + 1
            os.makedirs(os.path.dirname(raw), exist_ok=True)
            for attempt in range(3):
                try:
                    data, reported = call_api(a, model_id, prompt, refs)
                    open(raw, "wb").write(data)
                    entry["model"] = reported
                    break
                except Exception as e:  # API/network errors only; content review happens afterwards
                    print(a["id"], f"api error (attempt {attempt + 1}):", e)
                    time.sleep(5 * (attempt + 1))
            else:
                continue
            entry["generated_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")
        if not os.path.exists(raw):
            print(a["id"], "no raw image at", raw)
            continue
        post, checks = postprocess(a, raw, final, args.upscale)
        entry.update({"id": a["id"], "base": a.get("base"), "variant": a["variant"], "file": a["file"],
                      "size": a["size"], "target": a.get("target"), "background": a["background"], "refs": a.get("refs", []),
                      "prompt_final": prompt, "post": post, "checks": checks})
        entry.setdefault("model", model_id)
        if args.note:
            entry["notes"] = (entry.get("notes", "") + " " + args.note).strip()
        manifest["assets"][a["id"]] = entry
        print("ok", a["id"], a["file"], checks.get("flag", ""))

    if emit:
        emit.close()
    if args.dry_run:
        return
    os.makedirs(root, exist_ok=True)
    manifest["updated_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")
    json.dump(manifest, open(mpath, "w", encoding="utf-8"), indent=2, ensure_ascii=False)
    if args.qa:
        qa(root)
    if args.derive:
        derive(root)
    if args.sheet:
        sheet(root)
    if args.zip:
        make_zip(root)
    print("Done. Review every image against AGENTS.md (hard rules + QA loop) before zipping finals.")


if __name__ == "__main__":
    sys.exit(main())
