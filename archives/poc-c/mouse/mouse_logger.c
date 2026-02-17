#include "mouse_logger.h"
#include "../logger/logger.h"
#include <windows.h>

static HHOOK mouse_hook = NULL;
static HANDLE mouse_thread_handle = NULL;

LRESULT CALLBACK mouse_proc(int nCode, WPARAM wParam, LPARAM lParam) {

    if (nCode == HC_ACTION) {

        switch (wParam) {

            case WM_LBUTTONDOWN:
                log_event("MOUSE", "click", "button=left");
                break;

            case WM_RBUTTONDOWN:
                log_event("MOUSE", "click", "button=right");
                break;

            case WM_MBUTTONDOWN:
                log_event("MOUSE", "click", "button=middle");
                break;

            case WM_MOUSEWHEEL:
                log_event("MOUSE", "scroll", "wheel");
                break;

            case WM_MOUSEMOVE:
                log_event("MOUSE", "movement", "detected");
                break;
        }
    }

    return CallNextHookEx(mouse_hook, nCode, wParam, lParam);
}

DWORD WINAPI mouse_thread(LPVOID param) {

    mouse_hook = SetWindowsHookEx(WH_MOUSE_LL,
                                  mouse_proc,
                                  NULL,
                                  0);

    if (!mouse_hook) {
        log_event("MOUSE", "hook_failed", "install_error");
        return 1;
    }

    log_event("MOUSE", "hook_installed", "success");

    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    UnhookWindowsHookEx(mouse_hook);
    return 0;
}

void mouse_logger_start() {
    mouse_thread_handle = CreateThread(NULL,
                                       0,
                                       mouse_thread,
                                       NULL,
                                       0,
                                       NULL);
}

void mouse_logger_stop() {
    if (mouse_hook)
        UnhookWindowsHookEx(mouse_hook);
}
