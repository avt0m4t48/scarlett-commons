# Architecture

```
            tasks (issues)            pull requests                    main
 people  ─▶ /claim, /split  ─▶  gate ─▶ verify ─▶ drawn reviewers ─▶ automerge ─▶ progress page
 agents                         │        │          │
                                │        │          └ blind review form, weighted score, digests must agree
                                │        └ new tests must fail on the base commit (unprivileged job)
                                └ game-data guard, task ownership, duplicates, licence provenance
```

## Pieces

| Path | What it is |
|---|---|
| `bot/rules.py` | Every rule as a pure function, unit-tested. Change a rule here, with a test. |
| `bot/main.py` | Talks to the GitHub API and applies the rules. Never runs pull request code. |
| `.github/workflows/gate.yml` | Rule check on every pull request (`pull_request_target`, write token, no checkout of PR code). |
| `.github/workflows/verify.yml` | Runs PR code: tests must fail on base (`pull_request`, read-only, no secrets). |
| `.github/workflows/bot.yml` | `/claim`, `/unclaim`, `/split`; hourly sweep: ready labels, expiry, fix tasks, merges. |
| `.github/workflows/progress.yml` | Regenerates the progress page every 6 hours on the `status` branch. |
| `tools/fails_on_base.py` | The verify step, runnable locally. |
| `tools/results.py` | Hardware results by consensus. |
| `tools/check_build.py` | Hash check of your own files against a list; never copies anything. |
| `tools/progress.py` | Builds the progress page from the task board. |
| `.github/workflows/engine-translator.yml`, `engine-runtime.yml` | Engine tests on every change under `engine/` (docs/engine-dev.md). |
| `.github/workflows/sync-engine.yml` | Daily upstream engine commits as a scanned `upstream-sync` pull request. |
| `.github/workflows/try.yml` | Tests a set of pull requests alone and stacked (`tools/try_prs.py`). |
| `.github/workflows/mirror.yml` | Mirrors the repository to a second host once a token is set. |
| `tools/local_run.py`, `tools/title_runs.py` | Title runs on your own copy, and their validation and consensus. |
| `victory/` | Track A: the VIctory runtime (GPL-3.0, imported with its history). It runs the title natively with the console APIs reimplemented. `cargo commons run <exe> --execute` starts it with track B built in. Its BCPack tables are not distributed here: `python -I tools/fetch_bcpack_tables.py` downloads them from VIctory at a pinned commit, checks the hash and puts them next to victory.exe. |
| `recomp/` | Track B (docs/track-b.md): statically recompiled functions, installed over the native ones when the image is mapped, per build manifest (`recomp/manifests/`). Anything a manifest does not list runs as track A. |
| `engine/` | The Leonida engine: the RDNA2 → SPIR-V translator and the D3D12-on-Vulkan backend (engine/README.md, docs/engine-plan.md). It is the second backend and the reference that track A is compared with. |

## Trust model

- Nothing a pull request contains is executed with write access.
- No single account can merge its own work: reviewers are drawn by the bot, the author never counts, and new
  accounts weigh half.
- Hardware results count only when independent people agree.
- Maintainers can stop anything (`hold`) and audit regularly (docs/maintainer-routine.md).
