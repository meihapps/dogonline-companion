#!/usr/bin/env python3
"""Downloads the enemy, fish, book, hardiness item and food images from https://dogonline.wiki.

Item names come from src/generate_*_data.rs. Run from anywhere with:

    python3 -I scripts/fetch_wiki_assets.py [--dry-run] [--force]

Images land in assets/wiki/<tab>/<slug>.<ext>, listed in assets/wiki/manifest.json along
with where each one came from (the wiki is CC BY-SA 4.0, so keep that attribution, along
with the licence text in assets/wiki/LICENSE).

Names that already have an image on disk and an entry in the manifest are skipped
entirely; pass --force to look them all up again (re-downloading any whose checksum no
longer matches the wiki).

To go easy on the wiki, the API is only asked for up to 50 titles at a time, downloads run
one at a time from the image CDN with a pause between them, and any file already on disk
with the right checksum is skipped.
"""

import hashlib
import json
import re
import shutil
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "assets" / "wiki"
API_URL = "https://dogonline.wiki/w/api.php"
USER_AGENT = (
    "dogonline-companion-asset-fetch/1.0 "
    "(https://github.com/meihapps/dogonline-companion)"
)
LICENSE = "CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/)"
ATTRIBUTION = "Dog Online Wiki contributors (https://dogonline.wiki)"

# item name -> wiki file, for images whose name doesn't match the page title and that
# aren't picked up as the page image either.
IMAGE_EXCEPTIONS: dict[str, str] = {}

BATCH_SIZE = 50
REQUEST_DELAY_SECONDS = 1.0
MAX_ATTEMPTS = 5

# tab name -> (data file, prefix every image for that tab must have)
TABS = {
    "bravery": ("generate_bravery_data.rs", "Enemy_"),
    "coordination": ("generate_coordination_data.rs", "Item_"),
    "creativity": ("generate_creativity_data.rs", "Item_"),
    "hardiness": ("generate_hardiness_data.rs", "Item_"),
    "nutrition": ("generate_nutrition_data.rs", "Item_"),
}


def request(url: str) -> bytes:
    """Fetches `url`, waiting and retrying if the server asks us to back off."""
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})

    for attempt in range(1, MAX_ATTEMPTS + 1):
        try:
            with urllib.request.urlopen(req, timeout=60) as response:
                return response.read()
        except urllib.error.HTTPError as err:
            if err.code not in (429, 500, 502, 503, 504) or attempt == MAX_ATTEMPTS:
                raise
            retry_after = err.headers.get("Retry-After", "")
            wait = int(retry_after) if retry_after.isdigit() else 30 * attempt
        except urllib.error.URLError:
            if attempt == MAX_ATTEMPTS:
                raise
            wait = 30 * attempt

        print(f"  backing off for {wait}s", file=sys.stderr)
        time.sleep(wait)

    raise AssertionError("unreachable")


def api_query(**params: str) -> dict:
    """Runs one API query, following continuations and yielding to replica lag."""
    pages: dict[str, dict] = {}
    redirects: dict[str, str] = {}
    cont: dict[str, str] = {}

    while True:
        query = {
            "action": "query",
            "format": "json",
            "formatversion": "2",
            "maxlag": "5",
            **params,
            **cont,
        }
        body = json.loads(request(f"{API_URL}?{urllib.parse.urlencode(query)}"))
        time.sleep(REQUEST_DELAY_SECONDS)

        if body.get("error", {}).get("code") == "maxlag":
            time.sleep(30)
            continue
        if "error" in body:
            raise RuntimeError(f"API error: {body['error']}")

        for redirect in body["query"].get("redirects", []):
            redirects[redirect["from"]] = redirect["to"]
        for page in body["query"]["pages"]:
            merged = pages.setdefault(page["title"], {})
            merged.update(page)

        if "continue" not in body:
            return {"pages": pages, "redirects": redirects}
        cont = body["continue"]


def batches(items: list[str]) -> list[list[str]]:
    return [items[i : i + BATCH_SIZE] for i in range(0, len(items), BATCH_SIZE)]


def slugify(name: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", name.lower().replace("'", "")).strip("-")


def sha1_of(path: Path) -> str | None:
    return hashlib.sha1(path.read_bytes()).hexdigest() if path.exists() else None


def load_entries() -> list[dict]:
    entries = []

    for tab, (file_name, _) in TABS.items():
        source = (ROOT / "src" / file_name).read_text()

        for name in re.findall(r'name:\s*"([^"]+)"', source):
            entries.append({"tab": tab, "name": name, "page": name})

    return entries


def load_manifest() -> list[dict]:
    path = OUT_DIR / "manifest.json"
    return json.loads(path.read_text())["assets"] if path.exists() else []


def resolve_images(entries: list[dict]) -> list[str]:
    """Fills in each entry's image details, returning a description of any problems.

    Each entry's image is the first of these that exists on the wiki: the page image of
    its page, the file name built from the page title (e.g. `Item_Oak_Leaf.png`), then
    any exception listed in `IMAGE_EXCEPTIONS`.
    """
    problems = []
    titles = sorted({entry["page"] for entry in entries})
    page_images: dict[str, str] = {}
    redirects: dict[str, str] = {}

    for batch in batches(titles):
        result = api_query(
            titles="|".join(batch),
            redirects="1",
            prop="pageimages",
            piprop="name",
            pilimit=str(BATCH_SIZE),
        )
        redirects |= result["redirects"]
        for title, page in result["pages"].items():
            if "pageimage" in page:
                page_images[title] = page["pageimage"]

    for entry in entries:
        title = redirects.get(entry["page"], entry["page"])
        prefix = TABS[entry["tab"]][1]
        entry["page"] = title
        entry["candidates"] = [
            image
            for image in (
                page_images.get(title),
                f"{prefix}{title.replace(' ', '_')}.png",
                IMAGE_EXCEPTIONS.get(entry["name"]),
            )
            if image is not None
        ]

    files = sorted({image for entry in entries for image in entry["candidates"]})
    # file name (or name of a redirect to it) -> (actual file name, image info)
    file_info: dict[str, tuple[str, dict]] = {}

    for batch in batches(files):
        result = api_query(
            titles="|".join(f"File:{name}" for name in batch),
            redirects="1",
            prop="imageinfo",
            iiprop="url|sha1",
        )
        def file_name(title: str) -> str:
            return title.removeprefix("File:").replace(" ", "_")

        found = {
            file_name(title): (file_name(title), page["imageinfo"][0])
            for title, page in result["pages"].items()
            if page.get("imageinfo")
        }
        file_info |= found
        for source, target in result["redirects"].items():
            if file_name(target) in found:
                file_info[file_name(source)] = found[file_name(target)]

    for entry in entries:
        candidates = entry.pop("candidates")
        prefix = TABS[entry["tab"]][1]
        image = next((image for image in candidates if image in file_info), None)

        if image is None:
            problems.append(f"{entry['name']}: none of {candidates} exist on the wiki")
            continue
        if not file_info[image][0].startswith(prefix):
            problems.append(f"{entry['name']}: {image!r} doesn't start with {prefix!r}")
            continue

        image, info = file_info[image]
        entry["wiki_file"] = image
        entry["path"] = f"{entry['tab']}/{slugify(entry['name'])}{Path(image).suffix.lower()}"
        entry["url"] = info["url"]
        entry["sha1"] = info["sha1"]

    return problems


def download(entries: list[dict]) -> tuple[int, int, list[str]]:
    fetched = reused = 0
    problems = []
    on_disk: dict[str, Path] = {}

    for entry in entries:
        target = OUT_DIR / entry["path"]
        target.parent.mkdir(parents=True, exist_ok=True)

        if sha1_of(target) == entry["sha1"]:
            on_disk[entry["sha1"]] = target
            reused += 1
            continue

        if entry["sha1"] in on_disk:
            shutil.copyfile(on_disk[entry["sha1"]], target)
            reused += 1
            continue

        data = request(entry["url"])
        time.sleep(REQUEST_DELAY_SECONDS)
        fetched += 1

        if hashlib.sha1(data).hexdigest() != entry["sha1"]:
            problems.append(f"{entry['name']}: checksum mismatch for {entry['url']}")
            continue

        target.write_bytes(data)
        on_disk[entry["sha1"]] = target
        print(f"  {entry['path']}")

    return fetched, reused, problems


def manifest_entry(entry: dict) -> dict:
    return {
        "tab": entry["tab"],
        "name": entry["name"],
        "path": entry["path"],
        "source": "https://dogonline.wiki/wiki/File:"
        + urllib.parse.quote(entry["wiki_file"]),
        "sha1": entry["sha1"],
    }


def write_manifest(assets: list[dict]) -> None:
    manifest = {
        "license": LICENSE,
        "attribution": ATTRIBUTION,
        "assets": assets,
    }
    (OUT_DIR / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


def main() -> int:
    dry_run = "--dry-run" in sys.argv[1:]
    force = "--force" in sys.argv[1:]
    entries = load_entries()
    old_assets = load_manifest()

    # (tab, name) -> manifest entry, for images already on disk.
    present = {
        (asset["tab"], asset["name"]): asset
        for asset in old_assets
        if (OUT_DIR / asset["path"]).exists()
    }
    to_resolve = [
        entry
        for entry in entries
        if force or (entry["tab"], entry["name"]) not in present
    ]

    print(
        f"resolving images for {len(to_resolve)} names"
        f" (skipping {len(entries) - len(to_resolve)} already present)"
    )
    problems = resolve_images(to_resolve) if to_resolve else []
    resolved = [entry for entry in to_resolve if "path" in entry]

    if dry_run:
        for entry in resolved:
            print(f"  {entry['path']} <- {entry['wiki_file']}")
    else:
        fetched, reused, download_problems = download(resolved)
        problems += download_problems
        # Keep the existing entry for anything that was skipped or couldn't be resolved.
        new_assets = {(entry["tab"], entry["name"]): manifest_entry(entry) for entry in resolved}
        write_manifest(
            [
                new_assets.get(key) or present[key]
                for key in ((entry["tab"], entry["name"]) for entry in entries)
                if key in new_assets or key in present
            ]
        )
        print(f"downloaded {fetched}, already had {reused}")

    for problem in problems:
        print(f"problem: {problem}", file=sys.stderr)

    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
