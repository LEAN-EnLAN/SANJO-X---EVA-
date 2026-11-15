"""Generate all assets in prompts.json with OpenAI gpt-image-2.
Usage: OPENAI_API_KEY=... python generate.py [--only S1,L1]
Check the current Images API docs first; adjust parameter names if they changed."""
import base64, json, os, sys, time, datetime, urllib.request

API = "https://api.openai.com/v1/images/generations"
cfg = json.load(open("prompts.json", encoding="utf-8"))
only = set(sys.argv[sys.argv.index("--only") + 1].split(",")) if "--only" in sys.argv else None
manifest = {"project": "Eva · Sanjo-X · NASA Space Apps 2026", "orchestrator_model": "gpt-6.1-terra",
            "image_model": cfg["image_model"], "generated_at": datetime.datetime.utcnow().isoformat() + "Z", "assets": []}

def call(asset):
    body = {"model": cfg["image_model"], "prompt": asset["prompt"], "size": asset["size"], "n": 1,
            "quality": cfg.get("quality", "high"), "output_format": cfg.get("output_format", "png")}
    if asset["background"] == "transparent":
        body["background"] = "transparent"
    req = urllib.request.Request(API, data=json.dumps(body).encode(), method="POST",
                                 headers={"Authorization": f"Bearer {os.environ['OPENAI_API_KEY']}", "Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=300) as r:
        return base64.b64decode(json.load(r)["data"][0]["b64_json"])

for a in cfg["assets"]:
    if only and a["id"] not in only:
        continue
    os.makedirs(os.path.dirname(a["file"]), exist_ok=True)
    for attempt in range(3):
        try:
            open(a["file"], "wb").write(call(a)); break
        except Exception as e:
            print(a["id"], "error:", e); time.sleep(5)
    manifest["assets"].append({**{k: a[k] for k in ("id", "file", "size", "background", "prompt")}, "retries": attempt, "notes": ""})
    print("ok", a["id"], a["file"])

json.dump(manifest, open("manifest.json", "w", encoding="utf-8"), indent=2, ensure_ascii=False)
print("Done. Now: review every image against AGENTS.md rules, watermark scenes/, build contact-sheet.png, zip as eva-assets.zip")
