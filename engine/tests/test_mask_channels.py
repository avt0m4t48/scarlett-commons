"""tools/mask_channels.py on synthetic images: every texel is built here, no game data."""
import io
import json
import os
import struct
import sys
import tempfile
import unittest
from contextlib import redirect_stdout

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "tools"))

import mask_channels as mc  # noqa: E402

W = H = 16


def image():
    """R: one 8x8 square (a broad layer). G: constant 200. B: isolated texels on a 4-texel grid (spots). A: 255."""
    px = bytearray()
    for y in range(H):
        for x in range(W):
            r = 255 if 4 <= x < 12 and 4 <= y < 12 else 0
            b = 255 if x % 4 == 1 and y % 4 == 1 else 0
            px += bytes((r, 200, b, 255))
    return bytes(px)


def rgba8_dds(width, height, payload):
    header = bytearray(128)
    header[0:4] = b"DDS "
    struct.pack_into("<IIII", header, 4, 124, 0x1007, height, width)
    struct.pack_into("<II", header, 76, 32, 0x4)                       # DDPF_FOURCC
    header[84:88] = b"DX10"
    return bytes(header) + struct.pack("<IIIII", 28, 3, 0, 1, 0) + payload  # R8G8B8A8_UNORM


class MaskChannelTests(unittest.TestCase):
    def test_blobs(self):
        mask = bytearray(9)
        for i in (0, 1, 3, 8):                                            # 3x3: an L of three and a corner
            mask[i] = 1
        self.assertEqual(mc.blobs(mask, 3, 3), [3, 1])
        row_end = bytearray([0, 0, 1, 1, 0, 0])                         # 3 wide: (2,0) and (0,1) don't touch
        self.assertEqual(mc.blobs(row_end, 3, 2), [1, 1])
        row_start = bytearray([0, 0, 0, 1, 0, 1, 0, 0, 1])               # (0,1) only reaches (2,0) by wrapping
        self.assertEqual(mc.blobs(bytearray([0, 0, 1, 1, 0, 0]), 3, 2), [1, 1])
        self.assertEqual(mc.blobs(row_start, 3, 3), [2, 1])             # (2,1)-(2,2) join; (0,1) stays alone

    def test_channel_counts(self):
        c = mc.channel_counts(W, H, image(), (128,))
        self.assertEqual((c["R"]["thresholds"][128]["blobs"], c["R"]["thresholds"][128]["largest_share"]), (1, 1.0))
        self.assertEqual(c["R"]["thresholds"][128]["cover"], 0.25)
        self.assertEqual(c["B"]["thresholds"][128]["blobs"], 16)
        self.assertEqual(c["B"]["thresholds"][128]["largest_share"], 0.0625)
        self.assertEqual((c["G"]["min"], c["G"]["max"], c["G"]["distinct"]), (200, 200, 1))
        self.assertEqual(c["G"]["thresholds"][128], {"cover": 1.0, "blobs": 1, "largest_share": 1.0})
        self.assertEqual((c["A"]["min"], c["A"]["max"]), (255, 255))
        self.assertEqual(mc.channel_counts(W, H, image(), (250,))["G"]["thresholds"][250]["blobs"], 0)

    def test_cli(self):
        with tempfile.TemporaryDirectory() as tmp:
            path, out = os.path.join(tmp, "m.dds"), os.path.join(tmp, "m.json")
            with open(path, "wb") as fh:
                fh.write(rgba8_dds(W, H, image()))
            buf = io.StringIO()
            with redirect_stdout(buf):
                self.assertEqual(mc.main([path, "--thresholds", "128", "--json", out]), 0)
            self.assertIn("16x16", buf.getvalue())
            with open(out, encoding="utf-8") as fh:
                saved = json.load(fh)
            self.assertEqual(saved["channels"]["B"]["thresholds"]["128"]["blobs"], 16)
            for bad in ("0", "300", "x"):
                with self.subTest(bad=bad), redirect_stdout(io.StringIO()), \
                        self.assertRaises(SystemExit) as cm:
                    sys.stderr, saved_err = io.StringIO(), sys.stderr
                    try:
                        mc.main([path, "--thresholds", bad])
                    finally:
                        sys.stderr = saved_err
                self.assertEqual(cm.exception.code, 2)


if __name__ == "__main__":
    unittest.main()
