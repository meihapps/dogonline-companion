#!/usr/bin/env python3
"""Compares the app's data in src/generate_*_data.rs against https://dogonline.wiki.

Run from anywhere with:

    python3 -I scripts/check_wiki_data.py [--tab <tab>] [--gaps] [--json]

Prints only the differences, one line each:

    + only on the wiki
    - only in the app (the wiki may have a page for it, but not its item data)
    ~ on both, but a field differs
    ? the wiki has the item, but not this value: a gap in the wiki rather than the app.
      These are counted on one line per field; pass --gaps to list them.

and exits 1 if there were any (`?` lines alone don't count), 0 if not, and 3 if the check
itself failed (the wiki couldn't be reached, say). The wiki is the source of truth, so a
difference means the app is out of date (or the wiki has a gap worth knowing about).

Food, book, crop/bug/gem and fish data comes from the wiki's Bucket tables (the same data its
tables are generated from). Enemies have no Bucket table, so their bravery is read from the
table on the Bravery page. That table doesn't list HP or damage type, so those aren't
checked.

To go easy on the wiki, this makes one request per table with a pause between them.
"""

import difflib
import json
import re
import sys
import time
import traceback
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
API_URL = "https://dogonline.wiki/w/api.php"
USER_AGENT = (
    "dogonline-companion-data-check/1.0 "
    "(https://github.com/meihapps/dogonline-companion)"
)

REQUEST_DELAY_SECONDS = 1.0
MAX_ATTEMPTS = 5
# Bucket queries return at most this many rows; hitting it means rows were cut off.
BUCKET_LIMIT = 5000

# Wiki spelling -> app spelling, for values that don't match the app's enum variant names
# once those are split into words (lowercased before lookup).
ALIASES = {
    "old fishin' hole": "Old Fishing Hole",
}

# Wiki food types the app treats as no type at all.
NO_FOOD_TYPE = {"?", "Liquid"}


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


def api(**params: str) -> dict:
    query = urllib.parse.urlencode({"format": "json", "formatversion": "2", **params})
    body = json.loads(request(f"{API_URL}?{query}"))
    time.sleep(REQUEST_DELAY_SECONDS)

    if "error" in body:
        raise RuntimeError(f"API error: {body['error']}")
    return body


def bucket(name: str, fields: list[str], where: str = "") -> list[dict]:
    """Runs a Bucket query, e.g. `bucket("items", ["name"], ".where('nutrition', 5)")`."""
    select = ", ".join(f"'{field}'" for field in fields)
    query = f"bucket('{name}').select({select}){where}.limit({BUCKET_LIMIT}).run()"
    rows = api(action="bucket", query=query)["bucket"]

    if len(rows) >= BUCKET_LIMIT:
        raise RuntimeError(f"{name} has {BUCKET_LIMIT}+ rows; raise BUCKET_LIMIT")
    return rows


def page_wikitext(title: str) -> str:
    page = api(
        action="query", prop="revisions", rvprop="content", rvslots="main", titles=title
    )["query"]["pages"][0]
    return page["revisions"][0]["slots"]["main"]["content"]


def words(variant: str) -> str:
    """`OldFishingHole` -> `Old Fishing Hole`, `BucketOfWorms` -> `Bucket of Worms`."""
    split = re.sub(r"(?<!^)(?=[A-Z])", " ", variant)
    return re.sub(r" (Of|The|And) ", lambda m: m.group(0).lower(), split)


def normalise(value: str) -> str:
    """Wiki spelling of an enum value -> the app's, so both compare equal."""
    value = value.strip()
    value = ALIASES.get(value.lower(), value)
    return value[:1].upper() + value[1:]


def name_from_image(image: str) -> str:
    """`Item Bucket of Worms.png` -> `Bucket of Worms`."""
    return re.sub(r"^(Item|Enemy) ", "", Path(image.replace("_", " ")).stem)


def wiki_list(value: str, all_values: list[str]) -> set[str]:
    """`Morning, Afternoon` -> {Morning, Afternoon}; `All` -> every value; `?` -> none."""
    if value.strip().lower() == "all":
        return set(all_values)
    return {normalise(part) for part in value.split(",") if part.strip() not in ("", "?")}


def is_set(row: dict, field: str) -> bool:
    """Bucket leaves out false booleans, and gives true ones as an empty string."""
    return row.get(field) is not None


# --- app data --------------------------------------------------------------------------


def rust_records(file_name: str, struct: str) -> list[str]:
    """The body of every `Struct { ... }` literal in `src/<file_name>`."""
    source = (ROOT / "src" / file_name).read_text()
    return re.findall(rf"\b{struct} \{{([^{{}}]*)\}}", source)


def field(record: str, name: str) -> str:
    match = re.search(rf"\b{name}:\s*(.*?),?\n", record + "\n")
    if not match:
        raise ValueError(f"no {name!r} in {record!r}")
    return match.group(1)


def string_field(record: str, name: str) -> str:
    return re.search(r'"([^"]*)"', field(record, name)).group(1)


def int_field(record: str, name: str) -> int | None:
    """`5` -> 5, `Some(5)` -> 5, `None` -> None."""
    value = field(record, name)
    if value == "None":
        return None
    return int(re.fullmatch(r"(?:Some\()?(\d+)\)?", value).group(1))


def enum_field(record: str, name: str) -> str | None:
    """`kind: Some(FoodType::Dairy)` -> `Dairy`; `kind: None` -> `None`."""
    variant = re.search(r"::(\w+)", field(record, name))
    return words(variant.group(1)) if variant else None


def enum_list_field(record: str, name: str) -> set[str]:
    """`location: vec![FishingSpot::OldFishingHole, ...]` -> {`Old Fishing Hole`, ...}."""
    values = re.search(rf"\b{name}:\s*vec!\[(.*?)\]", record, re.DOTALL).group(1)
    return {words(variant) for variant in re.findall(r"::(\w+)", values)}


def enum_variants(file_name: str, enum: str) -> list[str]:
    source = (ROOT / "src" / file_name).read_text()
    body = re.search(rf"pub enum {enum} \{{([^}}]*)\}}", source).group(1)
    return [words(variant) for variant in re.findall(r"\b(\w+),", body)]


# --- per-tab data, as {name: {field: value}} -------------------------------------------


def app_bravery() -> dict[str, dict]:
    return {
        string_field(r, "name"): {"bravery": int_field(r, "bravery")}
        for r in rust_records("generate_bravery_data.rs", "Enemy")
    }


def wiki_bravery() -> dict[str, dict]:
    """Rows of the enemy table on the Bravery page: image, [[Name]], +N, ..."""
    text = page_wikitext("Bravery")
    enemies = {}

    for row in text.split("\n|-")[1:]:
        cells = [cell.strip() for cell in re.split(r"\n[|!]", row) if cell.strip()]
        link = next((c for c in cells if re.fullmatch(r"\[\[[^\]]+\]\]", c)), None)
        value = next((c for c in cells if re.fullmatch(r"\+?\d+", c)), None)
        if link is None or value is None:
            continue
        # The page title, not the link text: `[[Cheese (Enemy)|Cheese]]` -> `Cheese (Enemy)`.
        name = link.strip("[]").split("|")[0].strip()
        enemies[name] = {"bravery": int(value)}

    return enemies


def app_coordination() -> dict[str, dict]:
    return {
        string_field(r, "name"): {
            "coordination": int_field(r, "coordination"),
            "bait": {enum_field(r, "bait")},
            "location": enum_list_field(r, "location"),
            "time": enum_list_field(r, "time"),
            "weather": enum_list_field(r, "weather"),
        }
        for r in rust_records("generate_coordination_data.rs", "Fish")
    }


def wiki_coordination() -> dict[str, dict]:
    """Fish are items marked as fish or with a coordination value; where to catch them is
    in `fishing`.

    The app keeps one list of times and weathers per fish, so a fish caught in several
    places gets the union of them.
    """
    times = enum_variants("coordination.rs", "TimeOfDay")
    weathers = enum_variants("coordination.rs", "Weather")
    fish = {
        row["name"]: {
            "coordination": row.get("coordination"),
            "bait": set(),
            "location": set(),
            "time": set(),
            "weather": set(),
        }
        for where in (".where('is_fish', true)", ".where('coordination', '!=', bucket.Null())")
        for row in bucket("items", ["name", "coordination"], where)
    }

    for row in bucket("fishing", ["page_name", "fish", "bait", "timeofday", "weather"]):
        entry = fish.get(name_from_image(row["fish"]))
        if entry is None:
            continue  # not a fish (Water Bucket, Gold Boot, ...)
        entry["bait"].add(normalise(name_from_image(row["bait"])))
        entry["location"].add(normalise(row["page_name"]))
        entry["time"] |= wiki_list(row.get("timeofday", ""), times)
        entry["weather"] |= wiki_list(row.get("weather", ""), weathers)

    return fish


def app_creativity() -> dict[str, dict]:
    return {
        string_field(r, "name"): {"creativity": int_field(r, "creativity")}
        for r in rust_records("generate_creativity_data.rs", "Book")
    }


def wiki_creativity() -> dict[str, dict]:
    """Books are items marked as books or with a creativity value."""
    return {
        row["name"]: {"creativity": row.get("creativity")}
        for where in (".where('is_book', true)", ".where('creativity', '!=', bucket.Null())")
        for row in bucket("items", ["name", "creativity"], where)
    }


def app_hardiness() -> dict[str, dict]:
    return {
        string_field(r, "name"): {
            "type": enum_field(r, "item_type"),
            "hardiness": int_field(r, "hardiness"),
        }
        for r in rust_records("generate_hardiness_data.rs", "HardinessItem")
    }


def wiki_hardiness() -> dict[str, dict]:
    def item_type(row: dict) -> str | None:
        for flag, kind in (("is_crop", "Crop"), ("is_bug", "Bug"), ("is_gem", "Gem")):
            if is_set(row, flag):
                return kind
        return None

    return {
        row["name"]: {"type": item_type(row), "hardiness": row["hardiness"]}
        for row in bucket(
            "items",
            ["name", "hardiness", "is_crop", "is_bug", "is_gem"],
            ".where('hardiness', '!=', bucket.Null())",
        )
    }


def app_nutrition() -> dict[str, dict]:
    return {
        string_field(r, "name"): {
            "type": enum_field(r, "kind"),
            "nutrition": int_field(r, "nutrition"),
        }
        for r in rust_records("generate_nutrition_data.rs", "Food")
    }


def wiki_nutrition() -> dict[str, dict]:
    def food_type(row: dict) -> str | None:
        kind = normalise(row.get("food_type") or "")
        return None if kind in NO_FOOD_TYPE or not kind else kind

    return {
        row["name"]: {
            "type": food_type(row),
            "nutrition": row["nutrition"],
        }
        for row in bucket(
            "items",
            ["name", "food_type", "nutrition"],
            ".where('nutrition', '!=', bucket.Null())",
        )
    }


# tab -> (app data, wiki data, fields the wiki can't tell us, so a gap there isn't news)
TABS = {
    "bravery": (app_bravery, wiki_bravery, set()),
    "coordination": (app_coordination, wiki_coordination, set()),
    "creativity": (app_creativity, wiki_creativity, set()),
    "hardiness": (app_hardiness, wiki_hardiness, {"type"}),
    "nutrition": (app_nutrition, wiki_nutrition, set()),
}


# --- comparison ------------------------------------------------------------------------


def show(value) -> str:
    if isinstance(value, set):
        return ", ".join(sorted(value)) if value else "(none)"
    return "(none)" if value is None else str(value)


def compare(app: dict, wiki: dict, optional: set[str], wiki_items: set[str]) -> list[dict]:
    """`wiki_items` is every item on the wiki, to tell items it lacks a value for apart
    from ones it doesn't have at all."""
    issues = []
    only_wiki = sorted(wiki.keys() - app.keys())
    only_app = sorted(app.keys() - wiki.keys())

    def similar(name: str, others: list[str]) -> str | None:
        match = difflib.get_close_matches(name, others, n=1, cutoff=0.8)
        return match[0] if match else None

    for name in only_wiki:
        issues.append(
            {"kind": "+", "name": name, "wiki": wiki[name], "similar": similar(name, only_app)}
        )
    for name in only_app:
        if name in wiki_items:
            issues.append({"kind": "?", "name": name, "field": "any"})
        else:
            issues.append({"kind": "-", "name": name, "similar": similar(name, only_wiki)})

    for name in sorted(app.keys() & wiki.keys()):
        for key, wiki_value in wiki[name].items():
            app_value = app[name][key]
            if wiki_value in (None, set()):
                if app_value not in (None, set()) and key not in optional:
                    issues.append({"kind": "?", "name": name, "field": key})
            elif app_value != wiki_value:
                issues.append(
                    {"kind": "~", "name": name, "field": key, "app": app_value, "wiki": wiki_value}
                )

    return issues


def describe(issue: dict) -> str:
    hint = f" (maybe {issue['similar']!r}?)" if issue.get("similar") else ""

    if issue["kind"] == "+":
        values = ", ".join(f"{k}={show(v)}" for k, v in issue["wiki"].items())
        return f"+ {issue['name']}: {values}{hint}"
    if issue["kind"] == "-":
        if issue.get("redirect"):
            hint = f" (wiki page is {issue['redirect']!r})"
        return f"- {issue['name']}{hint}"

    app_value, wiki_value = issue["app"], issue["wiki"]
    if isinstance(app_value, set) and isinstance(wiki_value, set):
        changes = [f"+{v}" for v in sorted(wiki_value - app_value)]
        changes += [f"-{v}" for v in sorted(app_value - wiki_value)]
        return f"~ {issue['name']}: {issue['field']} {' '.join(changes)}"
    return f"~ {issue['name']}: {issue['field']} app={show(app_value)} wiki={show(wiki_value)}"


def add_redirects(results: dict[str, list[dict]]) -> None:
    """Notes where a name only in the app redirects to another wiki page, which usually
    means the wiki renamed it."""
    issues = [i for tab in results.values() for i in tab if i["kind"] == "-"]
    names = sorted({issue["name"] for issue in issues})
    redirects: dict[str, str] = {}

    for start in range(0, len(names), 50):
        batch = names[start : start + 50]
        query = api(action="query", redirects="1", titles="|".join(batch))["query"]
        redirects |= {r["from"]: r["to"] for r in query.get("redirects", [])}

    for issue in issues:
        if issue["name"] in redirects:
            issue["redirect"] = redirects[issue["name"]]


def print_report(tab: str, issues: list[dict], list_gaps: bool) -> None:
    differences = [issue for issue in issues if issue["kind"] != "?"]
    print(f"{tab}: {f'{len(differences)} differences' if differences else 'ok'}")
    for issue in differences:
        print(f"  {describe(issue)}")

    # Gaps in the wiki, one line per field, since there can be a lot of them.
    gaps: dict[str, list[str]] = {}
    for issue in issues:
        if issue["kind"] == "?":
            gaps.setdefault(issue["field"], []).append(issue["name"])
    for key, names in gaps.items():
        what = f"no {tab} data" if key == "any" else f"no {key}"
        listed = f": {', '.join(names)}" if list_gaps else ""
        print(f"  ? {len(names)} with {what} on the wiki{listed}")


def main() -> int:
    args = sys.argv[1:]
    as_json = "--json" in args
    tabs = list(TABS)

    if "--tab" in args:
        tab = args[args.index("--tab") + 1] if args.index("--tab") + 1 < len(args) else ""
        if tab not in TABS:
            print(f"--tab must be one of: {', '.join(TABS)}", file=sys.stderr)
            return 2
        tabs = [tab]

    wiki_items = {row["name"] for row in bucket("items", ["name"])}
    results = {}
    for tab in tabs:
        app_data, wiki_data, optional = TABS[tab]
        results[tab] = compare(app_data(), wiki_data(), optional, wiki_items)
    add_redirects(results)

    if as_json:
        print(json.dumps(results, indent=2, default=lambda v: sorted(v)))
    else:
        for tab, issues in results.items():
            print_report(tab, issues, "--gaps" in args)
        if "bravery" in results:
            print("(bravery: the wiki table has no HP or damage type, so those aren't checked)")

    return 1 if any(i["kind"] != "?" for issues in results.values() for i in issues) else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception:
        # Not 1, so a failed check can't be mistaken for finding differences.
        traceback.print_exc()
        sys.exit(3)
