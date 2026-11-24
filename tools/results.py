"""Hardware results by consensus.

A report is results/<test>/<commit>/<github login>.json:
  {"test": "...", "commit": "<40 hex>", "hw_class": "nvidia-ada", "driver": "621.24", "digest": "<sha256 hex>",
   "pass": true}
`digest` is the sha256 of the test's output. A (test, commit, hw_class) result is CONFIRMED when two or more
different people report the same digest, DISPUTED when reports disagree, and UNCONFIRMED with one report.

Usage: python -I tools/results.py [results_dir]   (prints a table; exit 1 on any DISPUTED)
"""

import json
import os
import re
import sys

HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
FIELDS = ("test", "commit", "hw_class", "driver", "digest", "pass")


def validate(path, report):
    """Return a problem string or None. The file name must be the reporter's login."""
    for k in FIELDS:
        if k not in report:
            return f"{path}: missing {k}"
    parts = path.replace("\\", "/").split("/")
    if len(parts) < 4 or parts[-3] != report["test"] or parts[-2] != report["commit"]:
        return f"{path}: must be results/<test>/<commit>/<login>.json matching its contents"
    if not HEX40.match(report["commit"]) or not HEX64.match(report["digest"]):
        return f"{path}: commit must be 40 hex digits and digest 64"
    return None


def consensus(reports):
    """reports: [(login, report)] -> {(test, commit, hw_class): (status, {digest: [logins]})}"""
    groups = {}
    for login, r in reports:
        key = (r["test"], r["commit"], r["hw_class"])
        groups.setdefault(key, {}).setdefault(r["digest"], set()).add(login)
    out = {}
    for key, by_digest in groups.items():
        if len(by_digest) > 1:
            status = "DISPUTED"
        elif len(next(iter(by_digest.values()))) >= 2:
            status = "CONFIRMED"
        else:
            status = "UNCONFIRMED"
        out[key] = (status, {d: sorted(u) for d, u in by_digest.items()})
    return out


def load(root):
    reports, problems = [], []
    for dirpath, _, names in os.walk(root):
        for n in names:
            if not n.endswith(".json"):
                continue
            p = os.path.join(dirpath, n)
            rel = os.path.relpath(p, os.path.dirname(os.path.abspath(root)))
            try:
                with open(p, encoding="utf-8") as f:
                    r = json.load(f)
            except (OSError, ValueError) as e:
                problems.append(f"{rel}: {e}")
                continue
            why = validate(rel, r)
            if why:
                problems.append(why)
            else:
                reports.append((n[:-5], r))
    return reports, problems


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "results"
    reports, problems = load(root)
    for p in problems:
        print("invalid:", p)
    disputed = False
    for (test, commit, hw), (status, by) in sorted(consensus(reports).items()):
        disputed |= status == "DISPUTED"
        who = "; ".join(f"{d[:12]}: {', '.join(u)}" for d, u in by.items())
        print(f"{status:11} {test} {commit[:9]} {hw}  {who}")
    return 1 if disputed or problems else 0


if __name__ == "__main__":
    sys.exit(main())
