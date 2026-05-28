#!/usr/bin/env python3
"""Fix a Cargo.toml from a workspace member to work as a standalone crate.

Replaces:
- `edition.workspace = true` → `edition = "2024"`
- `rust-version.workspace = true` → `rust-version = "1.91"`
- `license.workspace = true` → `license = "MIT OR Apache-2.0"`
- `repository.workspace = true` → `repository = "https://github.com/omics-rust/rsomics-world"`
- `authors.workspace = true` → `authors = ["Zane Leong <efd@live.com>"]`
- `dep.workspace = true` → resolved version
- `dep = { workspace = true, ... }` → resolved version + features
- `path = "..."` removed from deps
- `[lints] workspace = true` → removed (or replaced with inline)
"""

import re
import sys

WORKSPACE_PACKAGE = {
    "edition": '"2024"',
    "rust-version": '"1.91"',
    "license": '"MIT OR Apache-2.0"',
    "repository": '"https://github.com/omics-rust/rsomics-world"',
    "authors": '["Zane Leong <efd@live.com>"]',
}

WORKSPACE_DEPS = {
    "anyhow": '"1"',
    "thiserror": '"2"',
    "clap": '{ version = "4", features = ["derive"] }',
    "serde": '{ version = "1", features = ["derive"] }',
    "serde_json": '"1"',
    "rayon": '"1"',
    "flate2": '{ version = "1", default-features = false, features = ["zlib-rs"] }',
    "noodles": '"0.110"',
    "criterion": '{ version = "0.7", default-features = false, features = ["cargo_bench_support"] }',
}


def fix_toml(content: str) -> str:
    lines = content.split("\n")
    out = []
    skip_lints = False

    for line in lines:
        # Replace workspace package fields
        for key, val in WORKSPACE_PACKAGE.items():
            if line.strip() == f"{key}.workspace = true":
                line = f"{key} = {val}"
                break

        # Remove path = "..." from dependency lines
        if "path = " in line and "version" in line:
            line = re.sub(r',?\s*path\s*=\s*"[^"]*"', '', line)
            # Clean up {, version → { version
            line = line.replace("{, ", "{ ")
            line = line.replace("{ ,", "{")

        # Replace simple `dep.workspace = true`
        m = re.match(r'^(\w[\w-]*)\.(workspace)\s*=\s*true$', line.strip())
        if m and m.group(1) in WORKSPACE_DEPS:
            dep = m.group(1)
            line = f"{dep} = {WORKSPACE_DEPS[dep]}"

        # Replace `dep = { workspace = true }` or `dep = { workspace = true, features = [...] }`
        m = re.match(r'^([\w-]+)\s*=\s*\{.*workspace\s*=\s*true(.*)$', line.strip())
        if m:
            dep = m.group(1)
            rest = m.group(2).strip()
            if dep in WORKSPACE_DEPS:
                base = WORKSPACE_DEPS[dep]
                if rest and rest.startswith(","):
                    extra = rest.lstrip(",").strip().rstrip("}")
                    if base.startswith('"'):
                        line = f'{dep} = {{ version = {base}, {extra} }}'
                    else:
                        # Merge: if extra duplicates what's in base, skip it
                        inner = base.strip("{}").strip()
                        # Deduplicate features
                        if "features" in inner and "features" in extra:
                            line = f'{dep} = {{ {inner} }}'
                        else:
                            line = f'{dep} = {{ {inner}, {extra} }}'
                else:
                    line = f"{dep} = {base}"

        # Remove [lints] workspace = true section
        if line.strip() == "[lints]":
            skip_lints = True
            continue
        if skip_lints:
            if line.strip() == "workspace = true":
                skip_lints = False
                continue
            if line.strip() == "" or line.startswith("["):
                skip_lints = False
            else:
                continue

        out.append(line)

    return "\n".join(out)


if __name__ == "__main__":
    path = sys.argv[1]
    with open(path) as f:
        content = f.read()
    fixed = fix_toml(content)
    with open(path, "w") as f:
        f.write(fixed)
    print(f"Fixed: {path}")
