#!/usr/bin/env python3
"""Downloads the enemy, fish and hardiness item images from https://dogonline.wiki.

Item names come from src/generate_*_data.rs. Run from anywhere with:

    python3 -I scripts/fetch_wiki_assets.py [--dry-run]

Images land in assets/wiki/<tab>/<slug>.<ext>, listed in assets/wiki/manifest.json along
with where each one came from (the wiki is CC BY-SA 4.0, so keep that attribution, along
with the licence text in assets/wiki/LICENSE).

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

BATCH_SIZE = 50
REQUEST_DELAY_SECONDS = 1.0
MAX_ATTEMPTS = 5

# tab name -> (data file, prefix every image for that tab must have)
TABS = {
    "bravery": ("generate_bravery_data.rs", "Enemy_"),
    "coordination": ("generate_coordination_data.rs", "Item_"),
    "hardiness": ("generate_hardiness_data.rs", "Item_"),
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


def resolve_images(entries: list[dict]) -> list[str]:
    """Fills in each entry's image details, returning a description of any problems."""
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
        image = page_images.get(title)
        prefix = TABS[entry["tab"]][1]

        if image is None:
            problems.append(f"{entry['name']}: no image on page {title!r}")
        elif not image.startswith(prefix):
            problems.append(f"{entry['name']}: {image!r} doesn't start with {prefix!r}")
        else:
            entry["wiki_file"] = image
            entry["page"] = title

    files = sorted({entry["wiki_file"] for entry in entries if "wiki_file" in entry})
    file_info: dict[str, dict] = {}

    for batch in batches(files):
        result = api_query(
            titles="|".join(f"File:{name}" for name in batch),
            redirects="1",
            prop="imageinfo",
            iiprop="url|sha1",
        )
        for title, page in result["pages"].items():
            if page.get("imageinfo"):
                info = page["imageinfo"][0]
                file_info[title.removeprefix("File:").replace(" ", "_")] = info

    for entry in entries:
        if "wiki_file" not in entry:
            continue

        info = file_info.get(entry["wiki_file"])
        if info is None:
            problems.append(f"{entry['name']}: no file info for {entry['wiki_file']!r}")
            del entry["wiki_file"]
            continue

        suffix = Path(entry["wiki_file"]).suffix.lower()
        entry["path"] = f"{entry['tab']}/{slugify(entry['name'])}{suffix}"
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


def write_manifest(entries: list[dict]) -> None:
    manifest = {
        "license": LICENSE,
        "attribution": ATTRIBUTION,
        "assets": [
            {
                "tab": entry["tab"],
                "name": entry["name"],
                "path": entry["path"],
                "source": "https://dogonline.wiki/wiki/File:"
                + urllib.parse.quote(entry["wiki_file"]),
                "sha1": entry["sha1"],
            }
            for entry in entries
            if "path" in entry
        ],
    }
    (OUT_DIR / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


def main() -> int:
    dry_run = "--dry-run" in sys.argv[1:]
    entries = load_entries()
    print(f"resolving images for {len(entries)} names")
    problems = resolve_images(entries)
    resolved = [entry for entry in entries if "path" in entry]

    if dry_run:
        for entry in resolved:
            print(f"  {entry['path']} <- {entry['wiki_file']}")
    else:
        fetched, reused, download_problems = download(resolved)
        problems += download_problems
        write_manifest(entries)
        print(f"downloaded {fetched}, already had {reused}")

    for problem in problems:
        print(f"problem: {problem}", file=sys.stderr)

    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
