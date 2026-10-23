#!/usr/bin/env python3
"""Sync the Linux app's copy of the client specs.

Reads the specs package (CompositeCode/substratalapps-applications-specs) and
writes everything this repo derives from it:

  data/spec/screens.json        every screen: id, title, route, archetype, phase,
                                who, purpose, Linux container, loads, elements,
                                actions, states, notes, platforms
  data/spec/status-semantics.json
  data/spec/themes.json         theme ids, names and swatches (SET-01)
  data/spec/SOURCE              the specs commit these files came from
  crates/substratal-hub/resources/theme/{light,dark}/*.css
  data/icons/                   the app icon set from linux/brand-assets
  crates/substratal-hub/resources/brand/*.svg

The specs repo is the source of truth (PRODUCT-SPEC: "If this package and the
API spec disagree, the spec wins"), so these files are never edited by hand.
Run it after the specs change; CI runs it with --check.

  scripts/sync-specs.py [--specs PATH] [--check]

PATH defaults to $SPECS_DIR or ../substratalapps-applications-specs.
"""

import argparse
import filecmp
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

HEADER = re.compile(r"^### ([A-Z]{3,4}-\d{2}[a-z]?) — (.+)$")
META = re.compile(r"\*\*(Domain|Route|Archetype|Phase):\*\* ([^·]+?)\s*(?:·|$)")


def split_row(line):
    cells = line.strip().strip("|").split("|")
    return [c.strip().replace("\\|", "|") for c in cells]


def parse_tables(lines):
    """Yield (header, rows) for every markdown table in lines."""
    i = 0
    while i < len(lines):
        if lines[i].startswith("|") and i + 1 < len(lines) and set(lines[i + 1].replace("|", "").strip()) <= set("-: "):
            header = split_row(lines[i])
            rows = []
            i += 2
            while i < len(lines) and lines[i].startswith("|"):
                rows.append(split_row(lines[i]))
                i += 1
            yield header, rows
        else:
            i += 1


def field(block, name):
    for line in block:
        m = re.match(rf"\*\*{re.escape(name)}:\*\* (.*)$", line.strip())
        if m:
            return m.group(1).strip()
    return None


def strip_code(s):
    return s.strip().strip("`").strip() if s else s


def parse_screens(path, platform_control_column):
    text = path.read_text(encoding="utf-8").splitlines()
    screens = []
    current = None
    domain_heading = None
    for line in text:
        if line.startswith("## ") and not line.startswith("### "):
            domain_heading = line[3:].strip()
            if current:
                screens.append(current)
                current = None
            continue
        m = HEADER.match(line)
        if m:
            if current:
                screens.append(current)
            current = {"id": m.group(1), "title": m.group(2).strip(), "section_heading": domain_heading, "_lines": []}
            continue
        if current is not None:
            current["_lines"].append(line)
    if current:
        screens.append(current)

    out = []
    for s in screens:
        block = s.pop("_lines")
        meta_line = next((l for l in block if l.startswith("**Domain:**")), "")
        meta = {k.lower(): strip_code(v) for k, v in META.findall(meta_line)}
        s.update({
            "domain": meta.get("domain", ""),
            "route": meta.get("route", ""),
            "archetype": meta.get("archetype", ""),
            "phase": meta.get("phase", ""),
            "who": field(block, "Who") or "",
            "purpose": field(block, "Purpose") or "",
            "loads": field(block, "Loads") or "",
            "states": field(block, "States") or "",
            "notes": field(block, "Notes") or "",
            "entry_points": field(block, "Entry points") or "",
        })
        container = next((l for l in block if "container:**" in l), None)
        s["container"] = container.split("container:**", 1)[1].strip() if container else ""
        s["elements"] = []
        s["actions"] = []
        for header, rows in parse_tables(block):
            h = [x.lower() for x in header]
            if h and h[0] == "element":
                for r in rows:
                    r = r + [""] * (len(header) - len(r))
                    if platform_control_column:
                        name, kind, control, data, rules = (r + [""] * 5)[:5]
                    else:
                        name, kind, data, rules = (r + [""] * 4)[:4]
                        control = ""
                    s["elements"].append({"name": name, "kind": kind, "control": control, "data": data, "rules": rules})
            elif h and h[0] == "action":
                for r in rows:
                    action, call, result = (r + [""] * 3)[:3]
                    s["actions"].append({"action": action, "call": strip_code(call), "result": result})
        s["platforms"] = ["web"] if "web only" in s["phase"].lower() else ["all"]
        out.append(s)
    return out


def specs_commit(specs):
    try:
        return subprocess.check_output(["git", "-C", str(specs), "rev-parse", "HEAD"], text=True).strip()
    except Exception:
        return "unknown"


def build(specs, dest):
    pkg = specs / "substratal-application-specs"
    linux = pkg / "linux"
    shared = pkg / "00-shared"

    native = parse_screens(linux / "SCREENS.md", platform_control_column=True)
    native_ids = {s["id"] for s in native}
    # Screens the native apps skip (OPEN-QUESTIONS #27) only exist in the
    # shared file. Keep them so the shared help center can document them.
    shared_only = [s for s in parse_screens(shared / "SCREEN-SPECS.md", platform_control_column=False) if s["id"] not in native_ids]
    screens = native + shared_only

    (dest / "data/spec").mkdir(parents=True, exist_ok=True)
    doc = {
        "source": "CompositeCode/substratalapps-applications-specs",
        "commit": specs_commit(specs),
        "screens": screens,
    }
    (dest / "data/spec/screens.json").write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    shutil.copy(shared / "status-semantics.json", dest / "data/spec/status-semantics.json")
    themes = json.loads((shared / "themes/themes.json").read_text(encoding="utf-8"))
    slim = [{k: t[k] for k in ("id", "name", "mode", "default", "description")} | {"swatch": t["tokens"].get("accent")} for t in themes["themes"]]
    (dest / "data/spec/themes.json").write_text(json.dumps({"defaults": themes["defaults"], "themes": slim}, indent=1) + "\n", encoding="utf-8")
    (dest / "data/spec/SOURCE").write_text(f"{doc['source']}@{doc['commit']}\n")

    theme_dest = dest / "crates/substratal-hub/resources/theme"
    for mode in ("light", "dark"):
        (theme_dest / mode).mkdir(parents=True, exist_ok=True)
        for css in sorted((linux / "theme" / mode).glob("*.css")):
            shutil.copy(css, theme_dest / mode / css.name)

    brand_dest = dest / "crates/substratal-hub/resources/brand"
    brand_dest.mkdir(parents=True, exist_ok=True)
    for svg in sorted((linux / "brand-assets/svg").glob("*.svg")):
        shutil.copy(svg, brand_dest / svg.name)

    icons_dest = dest / "data/icons"
    if icons_dest.exists():
        shutil.rmtree(icons_dest)
    shutil.copytree(linux / "brand-assets/icons", icons_dest)
    return len(native), len(shared_only)


def tree_equal(a, b):
    cmp = filecmp.dircmp(a, b)
    if cmp.left_only or cmp.right_only or cmp.diff_files or cmp.funny_files:
        return False
    return all(tree_equal(Path(a) / d, Path(b) / d) for d in cmp.common_dirs)


OUTPUTS = ["data/spec", "data/icons", "crates/substratal-hub/resources/theme", "crates/substratal-hub/resources/brand"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--specs", default=os.environ.get("SPECS_DIR", str(ROOT.parent / "substratalapps-applications-specs")))
    ap.add_argument("--check", action="store_true", help="fail if the committed files differ from the specs")
    args = ap.parse_args()
    specs = Path(args.specs).resolve()
    if not (specs / "substratal-application-specs").is_dir():
        sys.exit(f"sync-specs: no specs package at {specs}")

    if args.check:
        with tempfile.TemporaryDirectory() as tmp:
            build(specs, Path(tmp))
            stale = [o for o in OUTPUTS if not (ROOT / o).exists() or not tree_equal(ROOT / o, Path(tmp) / o)]
        if stale:
            sys.exit("sync-specs: out of date with the specs: " + ", ".join(stale) + "\nRun scripts/sync-specs.py and commit.")
        print("sync-specs: up to date")
        return

    native, web = build(specs, ROOT)
    print(f"sync-specs: {native} native screens + {web} web-only screens from {specs_commit(specs)[:7]}")


if __name__ == "__main__":
    main()
