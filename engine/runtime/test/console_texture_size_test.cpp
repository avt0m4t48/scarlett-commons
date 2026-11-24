// src/console_texture_size.h: the console's texture sizes, hand-computed from addrlib's rules and cross-checked against
// the values VIctory's layout tests assert (GPL-3.0, crates/victory-formats/src/texture.rs).
#include "../src/console_texture_size.h"

#include <cstdio>

#define CHECK(condition) do {                                        \
    if (!(condition)) {                                              \
        std::fprintf(stderr, "line %d: %s\n", __LINE__, #condition); \
        return 1;                                                    \
    }                                                                \
} while (0)

using namespace leonida::gfx10;

namespace {

TextureShape shape(std::uint32_t w, std::uint32_t h, std::uint32_t mips, std::uint32_t bpe, std::uint32_t mode) {
    TextureShape s;
    s.width = w;
    s.height = h;
    s.mips = mips;
    s.element_bytes = bpe;
    s.swizzle = mode;
    return s;
}

constexpr std::uint32_t kLinear = 0, k4kbS = 5, k64kbD = 10, k64kbZX = 24, k64kbRX = 27;
constexpr std::uint32_t kTex2d = 3, kTex3d = 4, kBuffer = 1;

}  // namespace

int main() {
    // One 64 KiB D block of 16-byte elements is 64x64: a single 64x64 level fills it.
    auto s = console_plane_size(shape(64, 64, 1, 16, k64kbD));
    CHECK(s && s->size == 65536 && s->alignment == 65536);

    // A 9-level chain in 64 KiB D: mip 0 is a block, mips 1..8 share one tail block.
    s = console_plane_size(shape(64, 64, 9, 16, k64kbD));
    CHECK(s && s->size == 2 * 65536);

    // In 4 KiB S (16x16 blocks of 16-byte elements) the tail holds fewer mips: 64, 32 and 16 stay outside it.
    s = console_plane_size(shape(64, 64, 9, 16, k4kbS));
    CHECK(s && s->size == 65536 + 16384 + 4096 + 4096 && s->alignment == 4096);

    // Linear: rows aligned to 256 bytes, every level packed.
    s = console_plane_size(shape(30, 20, 1, 4, kLinear));
    CHECK(s && s->size == 64 * 20 * 4 && s->alignment == 256);

    // Array slices each hold the whole chain.
    TextureShape array = shape(64, 64, 1, 16, k64kbD);
    array.depth = 3;
    s = console_plane_size(array);
    CHECK(s && s->size == 3 * 65536);

    // Refused: past the 16384-texel limit, a mode number GFX10 has not, a 12-byte element, MSAA with mips.
    CHECK(!console_plane_size(shape(16385, 1, 1, 4, k64kbD)));
    CHECK(!console_plane_size(shape(64, 64, 1, 4, 3)));
    CHECK(!console_plane_size(shape(64, 64, 1, 12, k64kbD)));
    TextureShape msaa = shape(64, 64, 2, 4, k64kbRX);
    msaa.samples = 4;
    CHECK(!console_plane_size(msaa));

    // A 1920x1080 D24S8 depth buffer: the library's pick for a depth layout word of 0 is 64 KiB Z_X. Depth in
    // 128x128 blocks of 4 bytes (1920x1152), then the stencil plane in 256x256 blocks of 1 byte (2048x1280).
    auto t = console_texture_size(kTex2d, 1920, 1080, 1, 1, 45 /* D24_UNORM_S8_UINT */, 1, 0, 2 /* depth */, 0);
    CHECK(t && t->size == 1920ull * 1152 * 4 + 2048ull * 1280 && t->alignment == 65536);
    // The same with the mode named in the layout word (0x100 | 24).
    auto named = console_texture_size(kTex2d, 1920, 1080, 1, 1, 45, 1, 0x100 | k64kbZX, 0, 0);
    CHECK(named && named->size == t->size);

    // A render target picks 64 KiB R_X; BC7 (format 98) 256x256 is 64x64 elements of 16 bytes.
    t = console_texture_size(kTex2d, 256, 256, 1, 1, 98, 1, 0, 1 /* render target */, 0);
    CHECK(t && t->size == 65536 && t->alignment == 65536);

    // Mips 0 means the full chain: 256x256 BC7 has 9 levels, as the 64x64-element chain above.
    t = console_texture_size(kTex2d, 256, 256, 1, 0, 98, 1, 0x100 | k64kbD, 0, 0);
    CHECK(t && t->size == 2 * 65536);

    // A larger requested alignment wins and the size is rounded to it.
    t = console_texture_size(kTex2d, 64, 64, 1, 9, 2 /* R32G32B32A32 */, 1, 0x100 | k4kbS, 0, 1 << 20);
    CHECK(t && t->alignment == (1u << 20) && t->size == (1u << 20));

    // Not a texture, an unknown layout word, an unknown format: no answer (the device's is used).
    CHECK(!console_texture_size(kBuffer, 65536, 1, 1, 1, 0, 1, 1, 0, 0));
    CHECK(!console_texture_size(kTex2d, 64, 64, 1, 1, 2, 1, 0x205, 0, 0));
    CHECK(!console_texture_size(kTex2d, 64, 64, 1, 1, 6 /* R32G32B32: 12 bytes */, 1, 0, 0, 0));

    // A 3D texture in 64 KiB Z_X is thin: each depth slice holds the chain.
    t = console_texture_size(kTex3d, 64, 64, 4, 1, 2, 1, 0x100 | k64kbZX, 0, 0);
    CHECK(t && t->size == 4 * 65536);
    // In an S or D mode (S_X too) a 3D texture is thick: 16-byte elements in 64 KiB make 16x16x16 blocks, so 4 slices
    // of 64x64 take one slab of 4x4 blocks.
    t = console_texture_size(kTex3d, 64, 64, 4, 1, 2, 1, 0x100 | 25, 0, 0);
    CHECK(t && t->size == 16ull * 65536);
    // 4 KiB S, thick too.
    t = console_texture_size(kTex3d, 64, 32, 16, 1, 28 /* RGBA8 */, 1, 0x100 | k4kbS, 0, 0);
    // 4 KiB thick, 4-byte elements: 8x16x8 blocks; 64x32x16 is 8x2x2 blocks of 4 KiB.
    CHECK(t && t->size == 8ull * 2 * 2 * 4096 && t->alignment == 4096);

    std::printf("console_texture_size: sizes, tails, planes and refusals as the console lays them out\n");
    return 0;
}
