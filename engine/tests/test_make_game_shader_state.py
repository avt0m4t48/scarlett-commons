"""make_game.py's shader resume state: the settings recorded beside a finished translation, and whether a work
folder can be reused without carving, unpacking or translating again.

Every input is synthetic and every machine-dependent answer (llvm-mc's version, the translator source digest) is
mocked: no GPU, no LLVM, no Vulkan SDK and no game data are needed.

    python -I tests/test_make_game_shader_state.py
"""
import contextlib
import io
import os
import sys
import tempfile
import unittest
from unittest import mock

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)
import make_game  # noqa: E402

SIGNATURE = '["llvm-mc", "Ubuntu LLVM version 21.0.0"]'
DIGEST = "a" * 64
FLAGS = ("--no-float-controls", "--bary-fallback")


class TempWorkTest(unittest.TestCase):
    """A temporary folder with the shaders stage's layout, and the small file helpers the tests share."""

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.work = tmp.name
        self.dxbc = os.path.join(self.work, "shaders", "dxbc")
        self.spirv = os.path.join(self.work, "shaders", "spirv")
        self.disasm = os.path.join(self.dxbc, "disasm")

    def write(self, path, text, encoding="utf-8"):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding=encoding, newline="\n") as f:
            f.write(text)
        return path

    def touch(self, path):
        return self.write(path, "")

    def assert_fails(self, fn, *args, match=None):
        err = io.StringIO()
        with contextlib.redirect_stderr(err), self.assertRaises(SystemExit) as stop:
            fn(*args)
        self.assertEqual(stop.exception.code, 1)
        if match:
            self.assertIn(match, err.getvalue())
        return err.getvalue()


class ReadShaderSettingTest(TempWorkTest):
    """read_shader_setting(): the trimmed contents of a recorded setting, or the default when it can't be read."""

    def test_missing_file_returns_the_default(self):
        self.assertEqual(make_game.read_shader_setting(os.path.join(self.work, "missing.txt")), "")
        self.assertEqual(make_game.read_shader_setting(os.path.join(self.work, "missing.txt"), default="off"),
                         "off")

    def test_surrounding_whitespace_is_trimmed(self):
        for text in ("on\n", "  on  \r\n", "\n\ton\t\n"):
            with self.subTest(text=text):
                path = self.write(os.path.join(self.work, "setting.txt"), text)
                self.assertEqual(make_game.read_shader_setting(path), "on")

    def test_empty_file_is_empty_not_the_default(self):
        path = self.touch(os.path.join(self.work, "empty.txt"))
        self.assertEqual(make_game.read_shader_setting(path, default="off"), "")

    def test_a_folder_where_the_file_should_be_returns_the_default(self):
        # A stopped run can leave a folder where a marker belongs; reading it must not raise.
        os.makedirs(os.path.join(self.work, "marker.txt"))
        self.assertEqual(make_game.read_shader_setting(os.path.join(self.work, "marker.txt"), default="off"),
                         "off")

    def test_invalid_utf8_is_replaced_not_raised(self):
        path = os.path.join(self.work, "setting.txt")
        with open(path, "wb") as f:
            f.write(b"\xffon\n")
        value = make_game.read_shader_setting(path)
        self.assertTrue(value.startswith("\ufffd"), value)
        self.assertNotEqual(value, "on")


class RequestedShaderModeTest(unittest.TestCase):
    """requested_shader_mode(): whether the translation should have wave64 emulation on."""

    CASES = (
        ("", True, False, "on"),
        ("off", True, False, "on"),
        ("on", True, False, "on"),
        ("on", False, True, "on"),
        ("on", False, False, "off"),
        ("off", False, True, "off"),
        ("off", False, False, "off"),
        ("", False, False, "off"),
        ("", False, True, "off"),
        ("ON", False, True, "off"),
    )

    def test_table(self):
        for had, emulate, keep_emulated, want in self.CASES:
            with self.subTest(had=had, emulate=emulate, keep_emulated=keep_emulated):
                self.assertEqual(make_game.requested_shader_mode(had, emulate, keep_emulated), want)

    def test_explicit_emulation_wins_over_a_native_folder(self):
        self.assertEqual(make_game.requested_shader_mode("off", True, False), "on")

    def test_only_auto_keeps_an_emulated_folder_on(self):
        self.assertEqual(make_game.requested_shader_mode("on", False, True), "on")
        self.assertEqual(make_game.requested_shader_mode("on", False, False), "off")


class TranslatorFlagsSettingTest(unittest.TestCase):
    """translator_flags_setting(): the exact text recorded in spirv/translator_flags.txt."""

    def test_no_fallbacks_is_empty(self):
        self.assertEqual(make_game.translator_flags_setting(()), "")

    def test_one_flag(self):
        self.assertEqual(make_game.translator_flags_setting(("--bary-fallback",)), "--bary-fallback")

    def test_order_is_kept_and_flags_are_space_separated(self):
        self.assertEqual(make_game.translator_flags_setting(FLAGS),
                         "--no-float-controls --bary-fallback")
        self.assertEqual(make_game.translator_flags_setting(tuple(reversed(FLAGS))),
                         "--bary-fallback --no-float-controls")


class DisassemblyMatchesTest(TempWorkTest):
    """disassembly_matches(): a disassembly folder counts only with the marker of the llvm-mc that made it."""

    def setUp(self):
        super().setUp()
        os.makedirs(self.disasm)
        self.marker = os.path.join(self.dxbc, "disasm.done")

    def test_a_matching_marker_is_a_match(self):
        self.write(self.marker, SIGNATURE + "\n")
        self.assertTrue(make_game.disassembly_matches(self.dxbc, SIGNATURE))

    def test_surrounding_whitespace_in_the_marker_is_ignored(self):
        self.write(self.marker, "  " + SIGNATURE + "  \n\n")
        self.assertTrue(make_game.disassembly_matches(self.dxbc, SIGNATURE))

    def test_a_marker_for_another_llvm_mc_is_not_a_match(self):
        self.write(self.marker, SIGNATURE + "\n")
        self.assertFalse(make_game.disassembly_matches(self.dxbc, '["llvm-mc", "LLVM version 18.0.0"]'))

    def test_a_missing_marker_is_not_a_match(self):
        self.assertFalse(make_game.disassembly_matches(self.dxbc, SIGNATURE))

    def test_a_missing_disassembly_folder_is_not_a_match(self):
        # The marker alone is not proof: a stopped or failed run can leave it without the .s files.
        self.write(self.marker, SIGNATURE + "\n")
        os.rmdir(self.disasm)
        self.assertFalse(make_game.disassembly_matches(self.dxbc, SIGNATURE))

    def test_a_folder_where_the_marker_belongs_is_not_a_match(self):
        os.makedirs(self.marker)
        self.assertFalse(make_game.disassembly_matches(self.dxbc, SIGNATURE))

    def test_an_empty_marker_is_not_a_match(self):
        self.touch(self.marker)
        self.assertFalse(make_game.disassembly_matches(self.dxbc, SIGNATURE))


class ClearShaderDisassemblyTest(TempWorkTest):
    """clear_shader_disassembly(): drop generated assembly, but nothing else and nowhere outside the work folder."""

    def setUp(self):
        super().setUp()
        os.makedirs(self.disasm)

    def test_removes_only_generated_assembly(self):
        kept = [os.path.join(self.disasm, "shader.spv"), os.path.join(self.disasm, "notes.txt")]
        for name in ("a.s", "b.s"):
            self.touch(os.path.join(self.disasm, name))
        for path in kept:
            self.touch(path)
        os.makedirs(os.path.join(self.disasm, "nested.s"))   # a folder, not a file: left alone

        make_game.clear_shader_disassembly(self.work, self.disasm)

        self.assertEqual(sorted(os.listdir(self.disasm)),
                         ["nested.s", "notes.txt", "shader.spv"])

    def test_symlinked_assembly_is_removed_without_following_it(self):
        target = self.touch(os.path.join(self.work, "elsewhere.s"))
        link = os.path.join(self.disasm, "linked.s")
        try:
            os.symlink(target, link)
        except (OSError, NotImplementedError):
            self.skipTest("this platform cannot create symbolic links")
        make_game.clear_shader_disassembly(self.work, self.disasm)
        self.assertFalse(os.path.lexists(link))
        self.assertTrue(os.path.isfile(target), "the symlink's target must survive")

    def test_a_missing_disassembly_folder_is_fine(self):
        os.rmdir(self.disasm)
        make_game.clear_shader_disassembly(self.work, self.disasm)

    def test_a_folder_outside_the_work_folder_is_refused(self):
        outside = tempfile.TemporaryDirectory()
        self.addCleanup(outside.cleanup)
        victim = self.touch(os.path.join(outside.name, "a.s"))
        self.assert_fails(make_game.clear_shader_disassembly, self.work, outside.name,
                          match="must be inside the work folder")
        self.assertTrue(os.path.isfile(victim), "nothing outside the work folder may be removed")

    def test_a_sibling_with_the_right_name_is_refused(self):
        sibling = os.path.join(self.dxbc, "disasm-old")
        os.makedirs(sibling)
        victim = self.touch(os.path.join(sibling, "a.s"))
        self.assert_fails(make_game.clear_shader_disassembly, self.work, sibling,
                          match="must be inside the work folder")
        self.assertTrue(os.path.isfile(victim))

    def test_another_spelling_of_the_same_folder_is_accepted(self):
        victim = self.touch(os.path.join(self.disasm, "a.s"))
        roundabout = os.path.join(self.work, "shaders", "dxbc", "..", "dxbc", "disasm")
        make_game.clear_shader_disassembly(self.work, roundabout)
        self.assertFalse(os.path.exists(victim))

    def test_a_link_to_a_folder_outside_is_refused(self):
        outside = tempfile.TemporaryDirectory()
        self.addCleanup(outside.cleanup)
        victim = self.touch(os.path.join(outside.name, "a.s"))
        os.rmdir(self.disasm)
        try:
            os.symlink(outside.name, self.disasm)
        except (OSError, NotImplementedError):
            self.skipTest("this platform cannot create symbolic links")
        self.assert_fails(make_game.clear_shader_disassembly, self.work, self.disasm,
                          match="must be inside the work folder")
        self.assertTrue(os.path.isfile(victim))


class ShaderOutputsReusableTest(TempWorkTest):
    """shader_outputs_reusable(): a work folder is reusable only when every recorded setting still matches."""

    def setUp(self):
        super().setUp()
        os.makedirs(self.disasm)

    def state(self, mode="off", flags="", digest=DIGEST, signature=SIGNATURE, markers=True):
        """Write the files a finished shaders stage leaves; markers=False writes none of them."""
        if not markers:
            return
        for path in (os.path.join(self.dxbc, "index.json"),
                     os.path.join(self.dxbc, "fssm_index.json"),
                     os.path.join(self.spirv, "index.json")):
            self.write(path, "{}")
        self.write(os.path.join(self.spirv, "wave64_emulate.txt"), mode + "\n")
        self.write(os.path.join(self.spirv, "translator_flags.txt"), flags + "\n" if flags else "")
        self.write(os.path.join(self.spirv, "translation_sources.sha256"), digest + "\n")
        self.write(os.path.join(self.dxbc, "disasm.done"), signature + "\n")

    def reusable(self, emulate=False, keep_emulated=False, fallbacks=()):
        with mock.patch.object(make_game, "llvm_mc_signature", return_value=SIGNATURE), \
                mock.patch.object(make_game, "translation_sources_digest", return_value=DIGEST):
            return make_game.shader_outputs_reusable(self.work, emulate, keep_emulated, fallbacks)

    def test_a_finished_native_translation_is_reusable(self):
        self.state(mode="off", flags="")
        self.assertTrue(self.reusable())

    def test_a_finished_emulated_translation_is_reusable_when_it_is_kept(self):
        self.state(mode="on")
        self.assertTrue(self.reusable(keep_emulated=True))

    def test_each_marker_is_required(self):
        for rel in (("dxbc", "index.json"), ("dxbc", "fssm_index.json"), ("spirv", "index.json")):
            with self.subTest(marker=rel):
                tmp = tempfile.TemporaryDirectory()
                self.addCleanup(tmp.cleanup)
                self.work = tmp.name
                self.dxbc = os.path.join(self.work, "shaders", "dxbc")
                self.spirv = os.path.join(self.work, "shaders", "spirv")
                self.disasm = os.path.join(self.dxbc, "disasm")
                os.makedirs(self.disasm)
                self.state()
                os.remove(os.path.join(self.work, "shaders", *rel))
                self.assertFalse(self.reusable())

    def test_no_markers_at_all_is_not_reusable(self):
        self.state(markers=False)
        self.assertFalse(self.reusable())

    def test_a_missing_disassembly_folder_is_not_reusable(self):
        self.state()
        os.rmdir(self.disasm)
        self.assertFalse(self.reusable())

    def test_a_missing_disassembly_marker_is_not_reusable(self):
        self.state()
        os.remove(os.path.join(self.dxbc, "disasm.done"))
        self.assertFalse(self.reusable())

    def test_another_llvm_mc_is_not_reusable(self):
        self.state(signature='["llvm-mc", "LLVM version 18.0.0"]')
        self.assertFalse(self.reusable())

    def test_another_translator_source_is_not_reusable(self):
        self.state(digest="b" * 64)
        self.assertFalse(self.reusable())

    def test_a_missing_translator_digest_is_not_reusable(self):
        self.state()
        os.remove(os.path.join(self.spirv, "translation_sources.sha256"))
        self.assertFalse(self.reusable())

    def test_an_emulated_folder_is_not_reused_when_emulation_is_turned_off(self):
        self.state(mode="on")
        self.assertFalse(self.reusable(keep_emulated=False))

    def test_a_native_folder_is_not_reused_when_emulation_is_asked_for(self):
        self.state(mode="off")
        self.assertFalse(self.reusable(emulate=True))

    def test_a_folder_from_before_the_mode_file_existed_counts_as_native(self):
        self.state(mode="off")
        os.remove(os.path.join(self.spirv, "wave64_emulate.txt"))
        self.assertTrue(self.reusable())

    def test_recorded_fallbacks_must_match_exactly(self):
        self.state(flags=make_game.translator_flags_setting(FLAGS))
        self.assertTrue(self.reusable(fallbacks=FLAGS))
        self.assertFalse(self.reusable(fallbacks=FLAGS[:1]))
        self.assertFalse(self.reusable(fallbacks=tuple(reversed(FLAGS))))

    def test_a_folder_without_fallbacks_is_reusable_without_them(self):
        self.state()
        os.remove(os.path.join(self.spirv, "translator_flags.txt"))
        self.assertTrue(self.reusable())
        self.assertFalse(self.reusable(fallbacks=("--mesh-compute",)))

    def test_an_unreadable_marker_is_not_reusable(self):
        self.state()
        os.remove(os.path.join(self.spirv, "translation_sources.sha256"))
        os.makedirs(os.path.join(self.spirv, "translation_sources.sha256"))
        self.assertFalse(self.reusable())


if __name__ == "__main__":
    unittest.main()
