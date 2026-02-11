#include "network_logger.h"
#include "../logger/logger.h"
#include <winsock2.h>
#include <windows.h>
#include <stdio.h>

static int network_up = -1;

int check_connectivity() {

    WSADATA wsa;
    WSAStartup(MAKEWORD(2,2), &wsa);

    SOCKET sock = socket(AF_INET, SOCK_STREAM, 0);

    if (sock == INVALID_SOCKET) {
        WSACleanup();
        return 0;
    }

    struct sockaddr_in server;
    server.sin_addr.s_addr = inet_addr("8.8.8.8");
    server.sin_family = AF_INET;
    server.sin_port = htons(53);

    // Set timeout
    DWORD timeout = 2000; // 2 seconds
    setsockopt(sock, SOL_SOCKET, SO_RCVTIMEO, (char*)&timeout, sizeof(timeout));
    setsockopt(sock, SOL_SOCKET, SO_SNDTIMEO, (char*)&timeout, sizeof(timeout));

    int result = connect(sock,
                         (struct sockaddr *)&server,
                         sizeof(server));

    closesocket(sock);
    WSACleanup();

    if (result == 0)
        return 1;
    else
        return 0;
}

DWORD WINAPI network_thread(LPVOID param) {

    log_event("NETWORK", "monitor_started", "");

    while (1) {

        int current_status = check_connectivity();

        if (network_up == -1) {
            network_up = current_status;
            log_event("NETWORK", "network_status",
                      current_status ? "up" : "down");
        }
        else if (current_status != network_up) {
            network_up = current_status;
            log_event("NETWORK", "network_status",
                      current_status ? "up" : "down");
        }

        Sleep(5000);
    }

    return 0;
}

void network_monitor_start() {
    CreateThread(NULL,
                 0,
                 network_thread,
                 NULL,
                 0,
                 NULL);
}
