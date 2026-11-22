// Link-time stand-in for the console's d3d12_xs.dll, so tiny_guest_draw.exe gets a real import of
// D3D12XboxCreateDevice. Under `gta6rt --run` the shim registry answers it instead. The stub is built into
// fixtures/stubs, away from the guest; if anything ever loaded it, it returns E_FAIL and no device, so the guest fails
// loudly. Test code, not game content.
extern "C" __declspec(dllexport) long __stdcall D3D12XboxCreateDevice(void*, const void*, const void*, void** out) {
    if (out) *out = nullptr;
    return -2147467259L;  // E_FAIL (0x80004005)
}
