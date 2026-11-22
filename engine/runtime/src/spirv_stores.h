#pragma once

// Does a SPIR-V module declare a storage image, storage texel buffer or storage buffer it may write?
//
// Vulkan lets a shader stage other than compute write such a variable only when the device enabled a feature for
// it: fragmentStoresAndAtomics for the fragment stage (VUID-RuntimeSpirv-NonWritable-06340), and
// vertexPipelineStoresAndAtomics for the vertex, tessellation and geometry stages (-06341). Without the feature,
// every such variable of the stage must carry the NonWritable decoration. The translator declares the storage
// images of a shader that stores or does an image atomic without it, so the device has to know which modules need
// the feature before it builds a pipeline from them.
//
// The scan is conservative the way the rule is: it looks at declarations, not at whether an instruction writes.
// No Vulkan headers; the caller has already checked the binary's framing (spirv_words.h).

#include <cstddef>
#include <cstdint>
#include <span>
#include <unordered_map>
#include <unordered_set>
#include <vector>

namespace leonida {

inline bool spirv_declares_writable_storage(std::span<const std::uint32_t> words) {
    constexpr std::uint32_t magic = 0x07230203;
    constexpr std::uint16_t op_type_image = 25, op_type_array = 28, op_type_runtime_array = 29, op_type_struct = 30,
                            op_type_pointer = 32, op_variable = 59, op_decorate = 71, op_member_decorate = 72;
    constexpr std::uint32_t decoration_buffer_block = 3, decoration_non_writable = 24;
    constexpr std::uint32_t class_uniform_constant = 0, class_uniform = 2, class_storage_buffer = 12;
    if (words.size() < 5 || words[0] != magic) return false;

    struct Pointer {
        std::uint32_t storage_class, pointee;
    };
    struct Variable {
        std::uint32_t pointer_type, id, storage_class;
    };
    std::unordered_set<std::uint32_t> storage_images;                  // OpTypeImage with Sampled == 2
    std::unordered_map<std::uint32_t, std::uint32_t> element_of;       // array type -> element type
    std::unordered_map<std::uint32_t, std::uint32_t> member_count;     // struct type -> members
    std::unordered_map<std::uint32_t, Pointer> pointers;
    std::unordered_set<std::uint32_t> non_writable, buffer_blocks;     // decorated ids
    std::unordered_map<std::uint32_t, std::unordered_set<std::uint32_t>> non_writable_members;
    std::vector<Variable> variables;

    for (std::size_t at = 5; at < words.size();) {
        const std::uint32_t count = words[at] >> 16;
        const std::uint16_t op = static_cast<std::uint16_t>(words[at] & 0xffff);
        if (count == 0 || at + count > words.size()) return false;  // not a module the driver would take either
        const std::uint32_t* w = &words[at];
        switch (op) {
        case op_type_image:  // result, sampled type, dim, depth, arrayed, ms, sampled, format
            if (count >= 9 && w[7] == 2) storage_images.insert(w[1]);
            break;
        case op_type_array:
        case op_type_runtime_array:
            if (count >= 3) element_of[w[1]] = w[2];
            break;
        case op_type_struct:
            if (count >= 2) member_count[w[1]] = count - 2;
            break;
        case op_type_pointer:
            if (count >= 4) pointers[w[1]] = {w[2], w[3]};
            break;
        case op_variable:
            if (count >= 4) variables.push_back({w[1], w[2], w[3]});
            break;
        case op_decorate:
            if (count >= 3 && w[2] == decoration_non_writable) non_writable.insert(w[1]);
            if (count >= 3 && w[2] == decoration_buffer_block) buffer_blocks.insert(w[1]);
            break;
        case op_member_decorate:
            if (count >= 4 && w[3] == decoration_non_writable) non_writable_members[w[1]].insert(w[2]);
            break;
        default:
            break;
        }
        at += count;
    }

    for (const Variable& variable : variables) {
        const auto pointer = pointers.find(variable.pointer_type);
        if (pointer == pointers.end()) continue;
        std::uint32_t type = pointer->second.pointee;
        for (auto element = element_of.find(type); element != element_of.end(); element = element_of.find(type))
            type = element->second;  // an array of images or of buffer blocks: the element decides
        if (non_writable.count(variable.id)) continue;
        if (variable.storage_class == class_uniform_constant) {
            if (storage_images.count(type)) return true;  // a storage image or a storage texel buffer
            continue;
        }
        const bool storage_buffer = variable.storage_class == class_storage_buffer ||
                                    (variable.storage_class == class_uniform && buffer_blocks.count(type));
        if (!storage_buffer) continue;
        // A block is read-only when every member is.
        const auto members = member_count.find(type);
        const auto read_only = non_writable_members.find(type);
        const std::uint32_t total = members == member_count.end() ? 0 : members->second;
        const std::size_t marked = read_only == non_writable_members.end() ? 0 : read_only->second.size();
        if (total == 0 || marked < total) return true;
    }
    return false;
}

}  // namespace leonida
