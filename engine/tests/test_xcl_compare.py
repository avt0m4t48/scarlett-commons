"""tools/xcl_compare.py: parsing of the test title's lines and the report it builds.

    python -I tests/test_xcl_compare.py

Pure functions only: nothing here starts XCL, gta6rt or a title, and no game files are read.
"""
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "tools"))
import xcl_compare as xc  # noqa: E402


class ParseResult(unittest.TestCase):
    KNOWN = ["kernelx: GetConsoleType / XSystemGetDeviceType", "files: T:\\ title scratch storage", "argv"]

    def test_pass(self):
        self.assertEqual(xc.parse_result("[PASS] argv", None, self.KNOWN), ("PASS", "argv", ""))

    def test_fail_with_colon_in_name_uses_the_running_check(self):
        name = "kernelx: GetConsoleType / XSystemGetDeviceType"
        got = xc.parse_result("[FAIL] " + name + ": GetConsoleType = 0", name, self.KNOWN)
        self.assertEqual(got, ("FAIL", name, "GetConsoleType = 0"))

    def test_fail_with_colon_in_name_uses_known_names(self):
        name = "kernelx: GetConsoleType / XSystemGetDeviceType"
        got = xc.parse_result("[FAIL] " + name + ": GetConsoleType = 0", None, self.KNOWN)
        self.assertEqual(got, ("FAIL", name, "GetConsoleType = 0"))

    def test_fail_unknown_name_splits_at_first_colon(self):
        self.assertEqual(xc.parse_result("[FAIL] new check: why", None, []), ("FAIL", "new check", "why"))

    def test_skip(self):
        self.assertEqual(xc.parse_result("[SKIP] argv", None, self.KNOWN), ("SKIP", "argv", ""))

    def test_other_lines(self):
        for line in ("[RUN] argv", "[rt] unimplemented: kernelx.dll!GetConsoleType", "xcl_testtitle: start", ""):
            self.assertIsNone(xc.parse_result(line, None, self.KNOWN))


class Report(unittest.TestCase):
    def runs(self):
        x, l = xc.Run("XCL"), xc.Run("Leonida")
        for n in ("a", "b", "c"):
            x.set(n, status="PASS")
        l.set("a", status="PASS")
        l.set("b", status="FAIL", reason="null", unimplemented=["kernelx.dll!XMemAlloc"])
        l.set("c", status="HANG", reason="no output for 15 s", stack=["main: XTaskQueueTerminate"])
        l.set("d", status="FAIL", reason="only here")
        return x, l

    def test_rows_follow_xcl_order_then_extra_checks(self):
        x, l = self.runs()
        r = xc.make_report(x, l, list(x.order), "gta6rt.exe", ".")
        self.assertEqual([row["check"] for row in r["checks"]], ["a", "b", "c", "d"])

    def test_counts_and_differences(self):
        x, l = self.runs()
        r = xc.make_report(x, l, list(x.order), "gta6rt.exe", ".")
        self.assertEqual(r["summary"]["xcl"]["PASS"], 3)
        self.assertEqual(r["summary"]["xcl"]["NOT RUN"], 1)
        self.assertEqual(r["summary"]["leonida"], {"PASS": 1, "FAIL": 2, "SKIP": 0, "HANG": 1, "NOT RUN": 0})
        self.assertEqual(r["differences"], 3)
        b = r["checks"][1]
        self.assertEqual((b["leonida"], b["unimplemented"]), ("FAIL", ["kernelx.dll!XMemAlloc"]))
        self.assertEqual(r["checks"][2]["stack"], ["main: XTaskQueueTerminate"])


class Transcript(unittest.TestCase):
    """run_title streams the title's output to the transcript file and keeps none of it on the Run."""

    def test_output_goes_to_the_file(self):
        class Sink:
            def put(self, event):
                pass

        script = "print('[RUN] a'); print('x' * 256); print('[PASS] a')"
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "out.txt")
            run = xc.Run("synthetic")
            hung = xc.run_title([sys.executable, "-I", "-c", script], dict(os.environ), tmp, run, ["a"], 15, None,
                                Sink(), [], path)
            with open(path, encoding="utf-8") as f:
                lines = f.read().split("\n")
        self.assertIsNone(hung)
        self.assertEqual(run.checks["a"]["status"], "PASS")
        self.assertTrue(lines[0].startswith("--- "))
        self.assertEqual(lines[1:4], ["[RUN] a", "x" * 256, "[PASS] a"])
        self.assertFalse(hasattr(run, "log"))


if __name__ == "__main__":
    unittest.main()
