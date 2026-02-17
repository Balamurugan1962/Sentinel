#include "keyboard_logger.h"
#include "../logger/logger.h"
#include <windows.h>
#include <stdio.h>

static HHOOK keyboard_hook = NULL;
static HANDLE keyboard_thread_handle = NULL;

LRESULT CALLBACK keyboard_proc(int nCode, WPARAM wParam, LPARAM lParam) {

    if (nCode == HC_ACTION) {
        if (wParam == WM_KEYDOWN) {
            log_event("KEYBOARD", "key_event", "pressed");
        }
    }

    return CallNextHookEx(keyboard_hook, nCode, wParam, lParam);
}

DWORD WINAPI keyboard_thread(LPVOID param) {

    keyboard_hook = SetWindowsHookEx(WH_KEYBOARD_LL,
                                     keyboard_proc,
                                     NULL,
                                     0);

    if (!keyboard_hook) {
        log_event("KEYBOARD", "hook_failed", "install_error");
        return 1;
    }

    log_event("KEYBOARD", "hook_installed", "success");

    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    UnhookWindowsHookEx(keyboard_hook);
    return 0;
}

void keyboard_logger_start() {
    keyboard_thread_handle = CreateThread(NULL,
                                          0,
                                          keyboard_thread,
                                          NULL,
                                          0,
                                          NULL);
}

void keyboard_logger_stop() {
    if (keyboard_hook)
        UnhookWindowsHookEx(keyboard_hook);
}
