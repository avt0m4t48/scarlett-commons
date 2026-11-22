// Value-level replay regression: full and rectangular UINT clears must carry integer values, not float bits, and
// normalised clears are rounded to nearest even like D3D (0.5 on 8 bits is 128).
// No Vulkan loader, GPU, shaders or game data. Expected conversions also match the WARP differential GPU tests.
#include "../src/vulkan_replay.h"
#include <array>
#include <cstdio>
#include <cstring>
#include <limits>

namespace {
VkClearColorValue captured{};
unsigned calls = 0;
VKAPI_ATTR void VKAPI_CALL barrier(VkCommandBuffer, VkPipelineStageFlags, VkPipelineStageFlags, VkDependencyFlags,
    std::uint32_t, const VkMemoryBarrier*, std::uint32_t, const VkBufferMemoryBarrier*,
    std::uint32_t, const VkImageMemoryBarrier*) {}
VKAPI_ATTR void VKAPI_CALL clear(VkCommandBuffer, VkImage, VkImageLayout, const VkClearColorValue* color,
    std::uint32_t, const VkImageSubresourceRange*) { captured = *color; ++calls; }
VKAPI_ATTR void VKAPI_CALL begin(VkCommandBuffer, const VkRenderingInfo*) {}
VKAPI_ATTR void VKAPI_CALL end(VkCommandBuffer) {}
VKAPI_ATTR void VKAPI_CALL attachments(VkCommandBuffer, std::uint32_t n, const VkClearAttachment* a,
    std::uint32_t, const VkClearRect*) { if (n == 1) { captured = a[0].clearValue.color; ++calls; } }

bool replay_color(VkFormat format, const leonida::ClearColor& color, bool rectangle) {
    leonida::CommandList list;
    const leonida::Rect rect{1, 1, 3, 3};
    if (!list.clear_render_target({1, 0, 0, 1}, color,
        rectangle ? std::span<const leonida::Rect>(&rect, 1) : std::span<const leonida::Rect>{}) || !list.close()) return false;
    leonida::ResourceBinding binding{};
    binding.image = reinterpret_cast<VkImage>(std::uintptr_t{1});
    binding.format = format;
    binding.width = binding.height = 4;
    leonida::ReplayFunctions vk{};
    vk.pipeline_barrier = barrier;
    vk.clear_color_image = clear;
    vk.begin_rendering = begin;
    vk.end_rendering = end;
    vk.clear_attachments = attachments;
    vk.image_view = [](const auto&, const auto&) { return reinterpret_cast<VkImageView>(std::uintptr_t{1}); };
    calls = 0;
    return leonida::replay(list, VK_NULL_HANDLE, [&](auto) { return &binding; }, vk) == leonida::ReplayResult::success && calls == 1;
}
} // namespace

int main() {
    const float nan = std::numeric_limits<float>::quiet_NaN(), inf = std::numeric_limits<float>::infinity();
    struct Case { leonida::ClearColor color; std::array<std::uint32_t, 4> u8, u32; };
    const Case cases[] = {
        {{1, 2, 3, 4}, {1, 2, 3, 4}, {1, 2, 3, 4}},
        {{1.5f, 2.5f, 3.5f, 4.5f}, {1, 2, 3, 4}, {1, 2, 3, 4}},
        {{-1, -0.0f, 254.5f, 255.5f}, {0, 0, 254, 255}, {0, 0, 254, 255}},
        {{256, 65536, 16777216, 4294967296.0f}, {255, 255, 255, 255}, {256, 65536, 16777216, 0xffffffffu}},
        {{nan, inf, -inf, 0.5f}, {0, 255, 0, 0}, {0, 0xffffffffu, 0, 0}},
        {{0.499f, 0.501f, 1.499f, 1.501f}, {0, 0, 1, 1}, {0, 0, 1, 1}},
        {{4294967040.0f, std::numeric_limits<float>::max(), 255, 0}, {255, 255, 255, 0}, {0xffffff00u, 0xffffffffu, 255, 0}},
    };
    unsigned checks = 0;
    for (const auto& c : cases) {
        for (const VkFormat format : {VK_FORMAT_R8G8B8A8_UINT, VK_FORMAT_R32_UINT, VK_FORMAT_R32G32B32A32_UINT}) {
            for (const bool rect : {false, true}) {
                const auto& expected = format == VK_FORMAT_R8G8B8A8_UINT ? c.u8 : c.u32;
                if (!replay_color(format, c.color, rect)) return 1;
                for (unsigned i = 0; i < 4; ++i) {
                    if (captured.uint32[i] != expected[i]) {
                        std::fprintf(stderr, "UINT clear: format %d rect %d channel %u: got %u, expected %u\n",
                            int(format), int(rect), i, captured.uint32[i], expected[i]);
                        return 1;
                    }
                }
                ++checks;
            }
        }
        for (const VkFormat format : {VK_FORMAT_R32_SFLOAT, VK_FORMAT_R16G16B16A16_SFLOAT}) {
            for (const bool rect : {false, true}) {
                if (!replay_color(format, c.color, rect) || std::memcmp(captured.float32, c.color.data(), sizeof(captured)) != 0)
                    return 1;
                ++checks;
            }
        }
    }
    // Normalised formats: D3D rounds to nearest even, the clamp and NaN rules are 0..1 and 0, and an exact tie goes to
    // the even neighbour. NVIDIA rounds 0.5 on 8 bits down to 127 by itself, so the value is rounded before Vulkan sees it.
    struct Norm {
        VkFormat format;
        leonida::ClearColor color;
        std::array<std::int32_t, 4> steps;  // what D3D writes, in units of the channel's largest value
        std::array<double, 4> maximum;
    };
    const Norm norms[] = {
        {VK_FORMAT_R8G8B8A8_UNORM, {0.5f, 0.499f, 0.501f, 1.0f}, {128, 127, 128, 255}, {255, 255, 255, 255}},
        {VK_FORMAT_B8G8R8A8_UNORM, {-1, 2, nan, 0.5f}, {0, 255, 0, 128}, {255, 255, 255, 255}},
        {VK_FORMAT_R8G8_UNORM, {0.5f, 1.5f, 0, 0}, {128, 255, 0, 0}, {255, 255, 255, 255}},
        {VK_FORMAT_A2B10G10R10_UNORM_PACK32, {0.5f, 0.5f, 0.5f, 0.5f}, {512, 512, 512, 2}, {1023, 1023, 1023, 3}},
        {VK_FORMAT_R16G16B16A16_UNORM, {0.5f, 0.25f, 0.75f, 1.0f}, {32768, 16384, 49151, 65535},
         {65535, 65535, 65535, 65535}},
        {VK_FORMAT_R8G8B8A8_SRGB, {0.2f, 0.7f, 0.5f, 0.5f}, {0, 0, 0, 128}, {0, 0, 0, 255}},  // colour: the GPU's sRGB curve
        {VK_FORMAT_B8G8R8A8_SRGB, {0.5f, 0.25f, 0.75f, 0.499f}, {0, 0, 0, 127}, {0, 0, 0, 255}},
        {VK_FORMAT_R16_UNORM, {0.5f, 0, 0, 0}, {32768, 0, 0, 0}, {65535, 65535, 65535, 65535}},
        // Signed normalised: -1.0 is -127 (-32767), the scale is 127 (32767) and ties go to the even neighbour.
        {VK_FORMAT_R8G8B8A8_SNORM, {-1, -0.5f, 0.5f, 1}, {-127, -64, 64, 127}, {127, 127, 127, 127}},
        {VK_FORMAT_R8G8_SNORM, {nan, inf, -inf, -0.5f}, {0, 127, -127, -64}, {127, 127, 127, 127}},
        {VK_FORMAT_R16G16B16A16_SNORM, {-2, 2, 0.001f, -0.001f}, {-32767, 32767, 33, -33},
         {32767, 32767, 32767, 32767}},
    };
    for (const auto& n : norms) {
        for (const bool rect : {false, true}) {
            if (!replay_color(n.format, n.color, rect)) return 1;
            for (unsigned i = 0; i < 4; ++i) {
                const float expected = n.maximum[i] != 0.0 ? static_cast<float>(double(n.steps[i]) / n.maximum[i]) : n.color[i];
                if (captured.float32[i] != expected) {
                    std::fprintf(stderr, "normalised clear: format %d rect %d channel %u: got %.9g, expected %.9g\n",
                        int(n.format), int(rect), i, double(captured.float32[i]), double(expected));
                    return 1;
                }
            }
            ++checks;
        }
    }
    // The other integer formats clamp to the component range and truncate toward zero (what WARP writes).
    struct Integer {
        VkFormat format;
        leonida::ClearColor color;
        std::array<std::int32_t, 4> expected;
    };
    const Integer integers[] = {
        {VK_FORMAT_R8_SINT, {-1.5f, -2.5f, 2.5f, 3.5f}, {-1, -2, 2, 3}},
        {VK_FORMAT_R8G8B8A8_SINT, {-129, 128, -32769, 32768}, {-128, 127, -128, 127}},
        {VK_FORMAT_R8G8B8A8_SINT, {2147483648.0f, -2147483904.0f, nan, inf}, {127, -128, 0, 127}},
        {VK_FORMAT_R16_SINT, {-129, 128, -32769, 32768}, {-129, 128, -32768, 32767}},
        {VK_FORMAT_R32_SINT, {-129, 128, -32769, 32768}, {-129, 128, -32769, 32768}},
        {VK_FORMAT_R32_SINT, {2147483648.0f, -2147483904.0f, nan, inf}, {2147483647, -2147483647 - 1, 0, 2147483647}},
        {VK_FORMAT_R32G32B32A32_SINT, {-inf, 0.5f, -0.5f, 100000.0f}, {-2147483647 - 1, 0, 0, 100000}},
        {VK_FORMAT_R8_UINT, {256, 1.5f, -1, nan}, {255, 1, 0, 0}},
        {VK_FORMAT_R16_UINT, {65536, 65535.5f, -0.0f, 2.5f}, {65535, 65535, 0, 2}},
    };
    for (const auto& c : integers) {
        for (const bool rect : {false, true}) {
            if (!replay_color(c.format, c.color, rect)) return 1;
            for (unsigned i = 0; i < 4; ++i) {
                if (captured.int32[i] != c.expected[i]) {
                    std::fprintf(stderr, "integer clear: format %d rect %d channel %u: got %d, expected %d\n",
                        int(c.format), int(rect), i, captured.int32[i], c.expected[i]);
                    return 1;
                }
            }
            ++checks;
        }
    }
    std::printf("clear_color_test: %u full/rectangle UINT, SINT, normalised and float replay checks passed\n", checks);
    return 0;
}
