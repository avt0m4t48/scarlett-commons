# Hardware results

Nobody can check a hardware or real-title result in CI, so results count by agreement between independent
people, not by trust in one.

- A report is a pull request that adds `results/<test>/<commit>/<your login>.json` (format in
  `tools/results.py`). The bot rejects a report whose file name is not the pull request author's login.
- `digest` is the sha256 of the test's output. The output itself is never committed when it could contain game
  data; the digest is enough to compare.
- **CONFIRMED:** two or more different people, same test, commit and hardware class, same digest.
  **DISPUTED:** different digests; a maintainer investigates. **UNCONFIRMED:** one report so far.
- To show you are testing the build everyone else is, check your own copy with `tools/check_build.py` against
  the hash list a maintainer publishes. Hashes identify files without containing them; the files themselves
  are never in this repository, in issues, or in CI.

This makes cheating expensive, not impossible: two accounts run by one person can agree. Maintainers may
discount reports from new accounts with no merged pull request (not automated yet).
