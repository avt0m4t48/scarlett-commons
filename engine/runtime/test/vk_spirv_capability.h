#pragma once

#include <cstddef>
#include <cstdint>
#include <cstring>

namespace leonida::vk_test {

inline constexpr std::uint32_t spv_sampled_image_array_dynamic_indexing = 29;
inline constexpr std::uint32_t spv_storage_image_array_dynamic_indexing = 31;

// OpCapability (opcode 17, two words) comes first, after the five-word SPIR-V header.
inline bool spirv_declares_capability(const void* code, std::size_t bytes, std::uint32_t capability) {
    const auto* data = static_cast<const char*>(code);
    for (std::size_t i = 20; i + 8 <= bytes; i += 8) {
        std::uint32_t words[2];
        std::memcpy(words, data + i, sizeof(words));
        if (words[0] != (2u << 16 | 17u)) break;
        if (words[1] == capability) return true;
    }
    return false;
}

}  // namespace leonida::vk_test
