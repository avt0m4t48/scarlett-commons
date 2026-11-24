#pragma once

// The size and alignment the console (RDNA2, GFX10.3) gives a texture: what its layout library answers and what
// device slots 25 and 108 (GetResourceAllocationInfo, GetResourceAllocationInfo2) must answer. The retail title keeps
// these sizes in its own texture tables and asserts when an answer disagrees with them.
//
// Only sizes: block dimensions, the mip tail and the padded size of each mip. No element addressing (see
// gfx10_tiling.h for two modes of that). The rules are addrlib's (AMD, MIT; Mesa src/amd/addrlib, gfx10addrlib.cpp and
// addrlib2.cpp: ComputeThinBlockDimension, ComputeThickBlockDimension, GetMipTailDim, GetMaxNumMipsInTail,
// HwlComputeSurfaceInfoLinear, ComputeSurfaceInfoMacroTiled), as VIctory (GPL-3.0, crates/victory-formats/src/
// texture.rs) applies them to the console, whose choices it also documents: the swizzle mode the library picks for a
// layout word of 0 and the extra stencil plane. Written here from those descriptions; the test vectors are checked
// against both.
//
// Not covered, so smaller here than on the console: compression metadata planes (HTILE, CMASK, FMASK, DCC).

#include <algorithm>
#include <bit>
#include <cstdint>
#include <optional>

namespace leonida::gfx10 {

enum class TexDimension { tex1d, tex2d, tex3d };

struct TextureShape {
    TexDimension dimension = TexDimension::tex2d;
    std::uint32_t width = 1, height = 1;
    std::uint32_t depth = 1;          // depth of a 3D texture, else the array size
    std::uint32_t mips = 1;           // at least 1
    std::uint32_t element_bytes = 4;  // 1, 2, 4, 8 or 16
    std::uint32_t block = 1;          // texels per element side: 1, or 4 for block compression
    std::uint32_t samples = 1;
    std::uint32_t swizzle = 0;        // hardware swizzle mode number (addrlib ADDR_SW_* order); 0 is linear
};

struct TextureSize {
    std::uint64_t size = 0;
    std::uint64_t alignment = 0;
};

namespace texture_size_detail {

inline constexpr std::uint32_t kMaxExtent = 1u << 14;  // GFX10 image descriptors hold at most 16384 per side
inline constexpr std::uint32_t kMaxMips = 16;

// log2 of the block in bytes: 0 linear, 8, 12 or 16; nullopt for a number GFX10 has no mode for.
inline std::optional<std::uint32_t> block_log2(std::uint32_t mode) {
    switch (mode) {
        case 0: return 0;
        case 1: case 2: return 8;
        case 5: case 6: case 21: case 22: case 23: return 12;
        case 9: case 10: case 17: case 18: case 24: case 25: case 26: case 27: return 16;
        default: return std::nullopt;
    }
}
inline bool standard_mode(std::uint32_t m) { return m == 1 || m == 5 || m == 9 || m == 17 || m == 21 || m == 25; }
inline bool display_mode(std::uint32_t m) { return m == 2 || m == 6 || m == 10 || m == 18 || m == 22 || m == 26; }

inline std::uint64_t round_up(std::uint64_t v, std::uint64_t to) { return (v + to - 1) / to * to; }
inline std::uint32_t ceil_div(std::uint32_t v, std::uint32_t by) { return (v + by - 1) / by; }

// Elements along one side of a mip as placement sees it: mip 0's elements, halved per level rounding up.
inline std::uint32_t padded_elements(std::uint32_t texels, std::uint32_t mip, std::uint32_t block) {
    return ceil_div(ceil_div(std::max(texels, 1u), block), 1u << mip);
}

// 256-byte micro block in elements by log2 element size: 2D (w, h) and 3D (w, h, d); and the 1 KiB 3D block.
inline constexpr std::uint32_t kMicro2d[5][2] = {{16, 16}, {16, 8}, {8, 8}, {8, 4}, {4, 4}};
inline constexpr std::uint32_t kKib3d[5][3] = {{16, 8, 8}, {8, 8, 8}, {8, 8, 4}, {8, 4, 4}, {4, 4, 4}};

inline bool valid(const TextureShape& s) {
    const auto in = [](std::uint32_t v, std::uint32_t lo, std::uint32_t hi) { return v >= lo && v <= hi; };
    if (!in(s.width, 1, kMaxExtent) || !in(s.height, 1, kMaxExtent) || !in(s.depth, 1, kMaxExtent)) return false;
    if (!in(s.mips, 1, kMaxMips) || !std::has_single_bit(s.element_bytes) || s.element_bytes > 16) return false;
    if (s.block != 1 && s.block != 4) return false;
    if (!std::has_single_bit(s.samples) || s.samples > 8) return false;
    if (s.samples > 1 && (s.dimension != TexDimension::tex2d || s.mips != 1)) return false;
    if (s.dimension == TexDimension::tex1d && s.height != 1) return false;
    if (s.dimension == TexDimension::tex3d) {
        const std::uint32_t m = s.swizzle;
        if (!(m == 0 || m == 5 || m == 9 || m == 17 || m == 21 || (m >= 24 && m <= 27))) return false;
    }
    return true;
}

// Rows aligned to 256 bytes; every mip of a slice packed; each array or depth slice holds the whole chain.
inline TextureSize linear_size(const TextureShape& s) {
    const std::uint32_t row_elements = 256 / s.element_bytes;
    std::uint64_t slice = 0;
    for (std::uint32_t i = 0; i < s.mips; ++i) {
        const std::uint64_t pitch = round_up(padded_elements(s.width, i, s.block), row_elements);
        slice += pitch * padded_elements(s.height, i, s.block) * s.element_bytes;
    }
    return {slice * s.depth, 256};
}

inline TextureSize tiled_size(const TextureShape& s, std::uint32_t blog) {
    const std::uint32_t elog = static_cast<std::uint32_t>(std::countr_zero(s.element_bytes));
    const std::uint64_t block_bytes = 1ull << blog;
    const bool thick = s.dimension == TexDimension::tex3d && (standard_mode(s.swizzle) || display_mode(s.swizzle));

    std::uint32_t bw, bh, bd;
    if (thick) {
        const std::uint32_t steps = (blog - 10) / 3, rest = (blog - 10) % 3;
        bw = kKib3d[elog][0] << steps;
        bh = kKib3d[elog][1] << (steps + rest / 2);
        bd = kKib3d[elog][2] << (steps + (rest ? 1 : 0));
    } else {
        const std::uint32_t slog = static_cast<std::uint32_t>(std::countr_zero(s.samples));
        const std::uint32_t elements = blog - elog - slog;
        const std::uint32_t wide = ((slog & 1) == 0 || (blog & 1) != 0) ? 1 : 0;
        const std::uint32_t w = (elements + wide) / 2;
        bw = 1u << w;
        bh = 1u << (elements - w);
        bd = 1;
    }

    std::uint64_t slice = 0;  // bytes of one slice (thin) or of one depth layer of a slab (thick)
    if (s.mips == 1) {
        slice = round_up(ceil_div(s.width, s.block), bw) * round_up(ceil_div(s.height, s.block), bh) *
                s.element_bytes * s.samples;
    } else {
        // The largest mip the tail holds: the block with one side halved.
        std::uint32_t tail_w = bw, tail_h = bh;
        if (!thick || blog % 3 == 1) tail_w >>= 1;
        else if (blog % 3 == 0) tail_h >>= 1;
        // How many mips fit in the tail; 256-byte blocks have none.
        const std::uint32_t effective = thick ? blog - (blog - 8) / 3 : blog;
        const std::uint32_t max_in_tail =
            effective == 8 ? 0 : effective <= 11 ? 1 + (1u << (effective - 9)) : effective - 4;
        // Z-order depth of 8 or 16 bits enters the tail as if it were 32-bit (addrlib's dsMipmapHtileFix).
        if (s.swizzle == 24 && elog <= 1) {
            tail_w /= kMicro2d[elog][0] / kMicro2d[2][0];
            tail_h /= kMicro2d[elog][1] / kMicro2d[2][1];
        }
        for (std::uint32_t i = 0; i < s.mips; ++i) {
            const std::uint32_t w = padded_elements(s.width, i, s.block), h = padded_elements(s.height, i, s.block);
            if (w <= tail_w && h <= tail_h && s.mips - i <= max_in_tail) {
                slice += block_bytes / bd;  // the rest of the chain shares one block
                break;
            }
            slice += round_up(w, bw) * round_up(h, bh) * s.element_bytes;
        }
    }
    const std::uint64_t slabs = round_up(s.depth, bd) / bd;
    return {slice * bd * slabs, block_bytes};
}

}  // namespace texture_size_detail

// nullopt for a shape the console cannot lay out (or that this model refuses: sizes past the hardware limit, element
// sizes that are not a power of two).
inline std::optional<TextureSize> console_plane_size(const TextureShape& s) {
    using namespace texture_size_detail;
    const auto blog = block_log2(s.swizzle);
    if (!blog || !valid(s)) return std::nullopt;
    return *blog == 0 ? linear_size(s) : tiled_size(s, *blog);
}

// What a DXGI format is for the layout: element bytes, block side, and whether a stencil plane follows (the console
// stores depth and stencil as two planes). nullopt for 96-bit, 1-bit, packed 4:2:2 and video formats.
struct ElementFormat {
    std::uint32_t bytes, block;
    bool stencil;
};
inline std::optional<ElementFormat> console_element_format(std::uint32_t f) {
    const auto in = [f](std::uint32_t lo, std::uint32_t hi) { return f >= lo && f <= hi; };
    if (in(1, 4)) return ElementFormat{16, 1, false};
    if (in(9, 18)) return ElementFormat{8, 1, false};
    if (in(19, 22) || in(44, 47)) return ElementFormat{4, 1, true};
    if (in(23, 43) || f == 67 || in(87, 93) || f == 116 || f == 117) return ElementFormat{4, 1, false};
    if (in(48, 59) || f == 85 || f == 86 || f == 115) return ElementFormat{2, 1, false};
    if (in(118, 120)) return ElementFormat{2, 1, true};  // the console's 16-bit depth with 8-bit stencil
    if (in(60, 65)) return ElementFormat{1, 1, false};
    if (in(70, 72) || in(79, 81)) return ElementFormat{8, 4, false};
    if (in(73, 78) || in(82, 84) || in(94, 99)) return ElementFormat{16, 4, false};
    return std::nullopt;
}

// A console resource description's texture: dimension 2/3/4 (1D/2D/3D, D3D12 numbering), the console layout word
// (0: the library picks by usage, 1: linear, 0x100 | mode: that mode), flags bit 0 render target and bit 1 depth.
// mips 0 means the full chain. The answer is rounded to the alignment, which is the description's when it asks for
// more. nullopt for buffers and anything console_plane_size refuses.
inline std::optional<TextureSize> console_texture_size(std::uint32_t dimension, std::uint64_t width,
                                                       std::uint32_t height, std::uint32_t depth_or_array,
                                                       std::uint32_t mips, std::uint32_t format,
                                                       std::uint32_t samples, std::uint32_t layout,
                                                       std::uint32_t flags, std::uint64_t asked_alignment) {
    TextureShape s;
    switch (dimension) {
        case 2: s.dimension = TexDimension::tex1d; break;
        case 3: s.dimension = TexDimension::tex2d; break;
        case 4: s.dimension = TexDimension::tex3d; break;
        default: return std::nullopt;
    }
    if (layout == 0) {
        s.swizzle = s.dimension == TexDimension::tex1d ? 0 : (flags & 2) ? 24 : (flags & 1) ? 27 : 5;
    } else if (layout == 1) {
        s.swizzle = 0;
    } else if ((layout & ~0xffu) == 0x100) {
        s.swizzle = layout & 0xff;
    } else {
        return std::nullopt;
    }
    const auto f = console_element_format(format);
    if (!f || width == 0 || width > texture_size_detail::kMaxExtent) return std::nullopt;
    s.width = static_cast<std::uint32_t>(width);
    s.height = height;
    s.depth = depth_or_array;
    s.block = f->block;
    s.samples = std::max(samples, 1u);
    const std::uint32_t depth3d = s.dimension == TexDimension::tex3d ? s.depth : 1;
    s.mips = mips ? mips : static_cast<std::uint32_t>(std::bit_width(std::max({s.width, s.height, depth3d, 1u})));
    s.element_bytes = f->bytes;
    auto plane = console_plane_size(s);
    if (!plane) return std::nullopt;
    TextureSize total = *plane;
    if (f->stencil) {
        s.element_bytes = 1;
        const auto stencil = console_plane_size(s);
        if (!stencil) return std::nullopt;
        total.size = texture_size_detail::round_up(total.size, stencil->alignment) + stencil->size;
        total.alignment = std::max(total.alignment, stencil->alignment);
    }
    const std::uint64_t alignment = std::max(asked_alignment, total.alignment);
    if (!std::has_single_bit(alignment)) return std::nullopt;
    return TextureSize{texture_size_detail::round_up(total.size, alignment), alignment};
}

}  // namespace leonida::gfx10
