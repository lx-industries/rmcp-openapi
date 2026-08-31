#!/usr/bin/env python3
"""Smoke-test the renovate.json Dockerfile customManager regex.

Guards against the "swallowed field" bug class: when a `# renovate:` annotation
carries a field the regex does not capture (e.g. `registryUrl=`, `extractVersion=`),
the non-greedy `depName` absorbs it and Renovate silently stops tracking the dep.

Checks:
  1. Synthetic cases — the regex captures registryUrl / versioning / extractVersion
     and leaves `depName` clean. Removing any of those capture groups from
     renovate.json makes the matching synthetic case fail (annotation no longer
     matches, or depName is polluted).
  2. Real Dockerfile — every `# renovate:` annotation in .devcontainer/Dockerfile
     matches the regex and yields a whitespace-free `depName`.

Note: Renovate uses RE2/JS named groups `(?<name>)`; Python's `re` spells them
`(?P<name>)`. We translate for testing only — the pattern subset used here
(named groups, `\\S`, optional groups, non-greedy) is semantically identical.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RENOVATE_JSON = REPO / "renovate.json"
DOCKERFILE = REPO / ".devcontainer" / "Dockerfile"


def js_to_py(pattern: str) -> str:
    """Translate JS/RE2 named groups to Python's syntax (test shim only)."""
    return pattern.replace("(?<", "(?P<")


def dockerfile_matchstrings(cfg: dict) -> list[str]:
    out: list[str] = []
    for mgr in cfg.get("customManagers", []):
        patterns = " ".join(mgr.get("managerFilePatterns", []))
        if "Dockerfile" in patterns:
            out.extend(mgr.get("matchStrings", []))
    if not out:
        sys.exit("FAIL: no Dockerfile customManager matchStrings found in renovate.json")
    return out


# (annotation block, expected depName, expected (field, value) the regex must capture)
SYNTHETIC = [
    (
        '# renovate: datasource=gitlab-releases depName=gitlab-org/cli registryUrl=https://gitlab.com\nARG GLAB_VERSION="1.86.0"',
        "gitlab-org/cli",
        ("registryUrl", "https://gitlab.com"),
    ),
    (
        '# renovate: datasource=github-tags depName=kubernetes/kubernetes extractVersion=^v(?<version>.+)$\nARG KUBECTL_VERSION="1.34.0"',
        "kubernetes/kubernetes",
        ("extractVersion", "^v(?<version>.+)$"),
    ),
    (
        '# renovate: datasource=docker depName=debian versioning=docker\nARG DEBIAN_VERSION="12"',
        "debian",
        ("versioning", "docker"),
    ),
    (
        '# renovate: datasource=github-releases depName=cli/cli\nARG GH_VERSION="2.88.1"',
        "cli/cli",
        (None, None),
    ),
]


def main() -> int:
    cfg = json.loads(RENOVATE_JSON.read_text())
    regexes = [re.compile(js_to_py(ms)) for ms in dockerfile_matchstrings(cfg)]

    def first_match(text: str):
        for rx in regexes:
            m = rx.search(text)
            if m:
                return m
        return None

    failures: list[str] = []

    # 1. Synthetic cases
    for block, exp_dep, (field, value) in SYNTHETIC:
        m = first_match(block)
        if not m:
            failures.append(f"synthetic: annotation did not match: {block.splitlines()[0]!r}")
            continue
        gd = m.groupdict()
        if gd.get("depName") != exp_dep:
            failures.append(f"synthetic: depName={gd.get('depName')!r} expected {exp_dep!r} (field swallowed?)")
        if field and gd.get(field) != value:
            failures.append(f"synthetic: {field}={gd.get(field)!r} expected {value!r}")

    # 2. Real Dockerfile — only annotations whose target is an ENV/ARG *_VERSION
    # line are this customManager's domain. Annotations on a FROM line (e.g.
    # `datasource=docker depName=debian`) are handled by Renovate's built-in
    # image manager, so skip them here.
    if DOCKERFILE.exists():
        lines = DOCKERFILE.read_text().splitlines()
        target_re = re.compile(r"^\s*(ENV|ARG)\s.*_VERSION")
        matched_deps: list[str] = []
        for i, ln in enumerate(lines):
            if not ln.lstrip().startswith("# renovate:"):
                continue
            nxt = next((lines[j] for j in range(i + 1, len(lines)) if lines[j].strip()), "")
            if not target_re.search(nxt):
                continue  # built-in manager target (FROM, etc.)
            m = first_match(ln + "\n" + nxt)
            if not m:
                failures.append(f"Dockerfile: annotation did not parse: {ln!r}")
                continue
            dep = m.groupdict().get("depName")
            if dep is None or re.search(r"\s", dep):
                failures.append(f"Dockerfile: polluted depName={dep!r} (uncaptured trailing field)")
            else:
                matched_deps.append(dep)
        print(f"Dockerfile: {len(matched_deps)} ARG/ENV-versioned annotations parsed; depNames={matched_deps}")
    else:
        print(f"note: {DOCKERFILE} not found — skipping live-Dockerfile check")

    if failures:
        print("\nFAIL:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("\nOK: renovate customManager regex captures registryUrl/versioning/extractVersion; no depName pollution")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
