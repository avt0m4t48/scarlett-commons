// A DLL that does what a title's input code does first: GameInputCreate, a timestamp, a gamepad reading. It uses the
// Windows SDK's public GameInput.h and links against the SDK's gameinput.lib, so GameInputCreate is imported from
// GameInput.dll (by ordinal) like any title's import. Built without the CRT: DllMain does nothing and the exports call
// only GameInput. gameinput_host_test maps it with the loader. No game data.
#include <windows.h>

#include <GameInput.h>

extern "C" BOOL WINAPI gameinput_host_main(HINSTANCE, DWORD, LPVOID) { return TRUE; }

// Returns 1 when the library was created. reading_hr is S_OK with a gamepad attached and
// GAMEINPUT_E_READING_NOT_FOUND without one; state_ok is 1 when GetGamepadState succeeded on a reading.
extern "C" __declspec(dllexport) int gameinput_host_probe(long* create_hr, unsigned long long* timestamp,
                                                          long* reading_hr, int* state_ok) {
    IGameInput* input = nullptr;
    IGameInputReading* reading = nullptr;
    const HRESULT hr = GameInputCreate(&input);
    *create_hr = hr;
    *timestamp = 0;
    *reading_hr = 0;
    *state_ok = 0;
    if (FAILED(hr) || !input) return 0;
    *timestamp = input->GetCurrentTimestamp();
    *reading_hr = input->GetCurrentReading(GameInputKindGamepad, nullptr, &reading);
    if (SUCCEEDED(*reading_hr) && reading) {
        GameInputGamepadState state;
        *state_ok = reading->GetGamepadState(&state) ? 1 : 0;
        reading->Release();
    }
    input->Release();
    return 1;
}
