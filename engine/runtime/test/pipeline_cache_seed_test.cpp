// The cache vk_pipeline_check saves has to hit on the runtime's device:
//   pipeline_cache_seed_test <module.spv> <pipeline_cache.bin>
//
// Creates the runtime's own device (create_vulkan_device_standalone), seeds a VkPipelineCache from the file and
// creates the compute pipeline of <module.spv> the way PipelineState::create_compute does for a shader without set-0
// bindings: no set layouts, one 144-byte compute push-constant range and the subgroup request of src/subgroup_size.h.
// It passes when the driver reports VK_PIPELINE_CREATION_FEEDBACK_APPLICATION_PIPELINE_CACHE_HIT_BIT for that creation.
//
// Exit 77 (skipped): no device, a cache written for another GPU or driver, or a driver that reports no hit even for
// a cache this device wrote itself a moment before. Run by test/check_pipeline_cache_seed.cmake after the checker.
#include "../shims/d3d12_com.h"
#include "../src/shader_library.h"
#include <wrl/client.h>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <iterator>
#include <vector>

namespace {

std::vector<char> read_file(const char* path) {
    std::ifstream file(path, std::ios::binary);
    return {std::istreambuf_iterator<char>(file), std::istreambuf_iterator<char>()};
}

struct Created {
    bool ok = false;
    bool hit = false;  // VALID and APPLICATION_PIPELINE_CACHE_HIT
};

}  // namespace

int main(int argc, char** argv) {
    if (argc != 3) {
        std::fprintf(stderr, "usage: pipeline_cache_seed_test <module.spv> <pipeline_cache.bin>\n");
        return 2;
    }
    const std::vector<char> code = read_file(argv[1]);
    const std::vector<char> seed = read_file(argv[2]);
    if (code.empty() || code.size() % 4) {
        std::fprintf(stderr, "%s: not a SPIR-V module\n", argv[1]);
        return 1;
    }
    if (seed.size() < sizeof(VkPipelineCacheHeaderVersionOne)) {
        std::fprintf(stderr, "%s: not a pipeline cache\n", argv[2]);
        return 1;
    }

    Microsoft::WRL::ComPtr<ID3D12Device> d3d;
    if (FAILED(rt::d3d12::create_vulkan_device_standalone(IID_PPV_ARGS(&d3d))) || !d3d) {
        std::printf("SKIP: no Vulkan device for the runtime\n");
        return 77;
    }
    const rt::d3d12::VulkanDeviceInfo* info = rt::d3d12::vulkan_device_info(d3d.Get());
    if (!info || !info->get_instance_proc) return 1;
    const VkDevice device = info->device;
    const auto device_proc = reinterpret_cast<PFN_vkGetDeviceProcAddr>(
        info->get_instance_proc(info->instance, "vkGetDeviceProcAddr"));
    const auto physical_properties = reinterpret_cast<PFN_vkGetPhysicalDeviceProperties>(
        info->get_instance_proc(info->instance, "vkGetPhysicalDeviceProperties"));
    if (!device_proc || !physical_properties) return 1;
#define LOAD(name) const auto name = reinterpret_cast<PFN_##name>(device_proc(device, #name))
    LOAD(vkCreateShaderModule);
    LOAD(vkDestroyShaderModule);
    LOAD(vkCreatePipelineLayout);
    LOAD(vkDestroyPipelineLayout);
    LOAD(vkCreatePipelineCache);
    LOAD(vkDestroyPipelineCache);
    LOAD(vkGetPipelineCacheData);
    LOAD(vkCreateComputePipelines);
    LOAD(vkDestroyPipeline);
#undef LOAD

    VkPhysicalDeviceProperties properties{};
    physical_properties(info->physical, &properties);
    VkPipelineCacheHeaderVersionOne header{};
    std::memcpy(&header, seed.data(), sizeof(header));
    if (header.vendorID != properties.vendorID || header.deviceID != properties.deviceID ||
        std::memcmp(header.pipelineCacheUUID, properties.pipelineCacheUUID, VK_UUID_SIZE) != 0) {
        std::printf("SKIP: %s was written for another GPU or driver than the runtime's %s\n", argv[2],
                    properties.deviceName);
        return 77;
    }

    VkShaderModuleCreateInfo module_info{VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO};
    module_info.codeSize = code.size();
    module_info.pCode = reinterpret_cast<const std::uint32_t*>(code.data());
    VkShaderModule module = VK_NULL_HANDLE;
    if (vkCreateShaderModule(device, &module_info, nullptr, &module) != VK_SUCCESS) return 1;
    const VkPushConstantRange push{VK_SHADER_STAGE_COMPUTE_BIT, 0, sizeof(leonida::ShaderUserData)};
    VkPipelineLayoutCreateInfo layout_info{VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO};
    layout_info.pushConstantRangeCount = 1;
    layout_info.pPushConstantRanges = &push;
    VkPipelineLayout layout = VK_NULL_HANDLE;
    if (vkCreatePipelineLayout(device, &layout_info, nullptr, &layout) != VK_SUCCESS) return 1;

    const auto make_cache = [&](const void* data, std::size_t size) {
        VkPipelineCacheCreateInfo cache_info{VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO};
        cache_info.initialDataSize = size;
        cache_info.pInitialData = data;
        VkPipelineCache cache = VK_NULL_HANDLE;
        if (vkCreatePipelineCache(device, &cache_info, nullptr, &cache) != VK_SUCCESS) return VkPipelineCache{};
        return cache;
    };
    // The index entry check_pipeline_cache_seed.cmake writes: both programs take the subgroup request from it.
    leonida::ShaderEntry entry;
    entry.stage = "cs";
    entry.local_size = {1, 1, 1};
    const auto create = [&](VkPipelineCache cache) {
        VkPipelineCreationFeedback feedback{};
        const VkPipelineCreationFeedbackCreateInfo feedback_info{
            VK_STRUCTURE_TYPE_PIPELINE_CREATION_FEEDBACK_CREATE_INFO, nullptr, &feedback, 0, nullptr};
        VkComputePipelineCreateInfo pipeline_info{VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO, &feedback_info};
        pipeline_info.stage = {VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO};
        pipeline_info.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT;
        pipeline_info.stage.module = module;
        pipeline_info.stage.pName = "main";
        VkPipelineShaderStageRequiredSubgroupSizeCreateInfo required_size{};
        leonida::apply_subgroup_request(leonida::decide_subgroup_size(entry, info->subgroup), required_size,
                                        pipeline_info.stage);
        pipeline_info.layout = layout;
        VkPipeline pipeline = VK_NULL_HANDLE;
        Created result;
        if (vkCreateComputePipelines(device, cache, 1, &pipeline_info, nullptr, &pipeline) != VK_SUCCESS) return result;
        vkDestroyPipeline(device, pipeline, nullptr);
        result.ok = true;
        result.hit = (feedback.flags & VK_PIPELINE_CREATION_FEEDBACK_VALID_BIT) &&
                     (feedback.flags & VK_PIPELINE_CREATION_FEEDBACK_APPLICATION_PIPELINE_CACHE_HIT_BIT);
        return result;
    };

    int code_out = 1;
    const VkPipelineCache seeded = make_cache(seed.data(), seed.size());
    const Created from_seed = seeded ? create(seeded) : Created{};
    if (!from_seed.ok) {
        std::fprintf(stderr, "FAIL: the pipeline was not created from the seeded cache\n");
    } else if (from_seed.hit) {
        std::printf("ok: %s hits on the runtime's device (%s)\n", argv[2], properties.deviceName);
        code_out = 0;
    } else {
        // Does this driver report a hit at all? Write a cache on this device and read it back.
        const VkPipelineCache first = make_cache(nullptr, 0);
        std::vector<char> own;
        std::size_t size = 0;
        if (first && create(first).ok && vkGetPipelineCacheData(device, first, &size, nullptr) == VK_SUCCESS) {
            own.resize(size);
            if (vkGetPipelineCacheData(device, first, &size, own.data()) != VK_SUCCESS) own.clear();
            own.resize(own.empty() ? 0 : size);
        }
        const VkPipelineCache second = own.empty() ? VkPipelineCache{} : make_cache(own.data(), own.size());
        const bool reports_hits = second && create(second).hit;
        if (first) vkDestroyPipelineCache(device, first, nullptr);
        if (second) vkDestroyPipelineCache(device, second, nullptr);
        if (reports_hits) {
            std::fprintf(stderr,
                         "FAIL: %s does not hit on the runtime's device (%s), which does hit a cache it wrote itself: "
                         "vk_pipeline_check's device differs from the runtime's in something the driver makes part "
                         "of a pipeline's cache key\n",
                         argv[2], properties.deviceName);
        } else {
            std::printf("SKIP: %s reports no cache hit even for a cache it wrote itself\n", properties.deviceName);
            code_out = 77;
        }
    }
    if (seeded) vkDestroyPipelineCache(device, seeded, nullptr);
    vkDestroyPipelineLayout(device, layout, nullptr);
    vkDestroyShaderModule(device, module, nullptr);
    return code_out;
}
