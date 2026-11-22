"""Fixed reference points for the synthetic guest's renderer and input trace (no GPU)."""
import importlib.util
from pathlib import Path
import struct
import unittest


path = Path(__file__).resolve().parents[1] / "scripts" / "run_guest_scene.py"
spec = importlib.util.spec_from_file_location("run_guest_scene", path)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class GuestSceneReferenceTest(unittest.TestCase):
    def test_trace_and_pixel_digest_at_scene_boundaries(self):
        # These records were read back independently on the RTX 5090 and Apple M3 Max.
        anchors = [
            (0, 512, 9, 8, 18, 1, 376597285, 314582078),
            (137, 132, 44, 32, 153, 3, 3593765925, 2667449936),
            (299, 320, 8, 58, 169, 2, 1840220197, 1732923390),
            (300, 64, 8, 57, 177, 2, 1840220197, 1732923390),
            (422, 4, 58, 8, 228, 1, 1662446373, 2331208379),
            (599, 576, 58, 8, 30, 1, 1662446373, 2331208379),
        ]
        steps = runner.trace()
        for frame, buttons, x, y, scene_byte, bias, low, high in anchors:
            with self.subTest(frame=frame):
                self.assertEqual(steps[frame], (buttons, x, y, scene_byte, bias))
                digest, _ = runner.predicted_image(x % 8, bias)
                self.assertEqual(digest, (high << 32) | low)

    def test_pcm_envelope_and_period(self):
        samples = struct.unpack("<11025h", runner.expected_pcm(9000))
        self.assertEqual(samples[0], 0)
        self.assertEqual(samples[500:550], (9000,) * 50)
        self.assertEqual(samples[550:600], (-9000,) * 50)
        self.assertEqual(samples[-1], 0)


if __name__ == "__main__":
    unittest.main()
