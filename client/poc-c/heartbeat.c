#include "heartbeat.h"
#include <stdio.h>
#include <winsock2.h>

#pragma comment(lib, "ws2_32.lib")

void send_heartbeat(const char *ip, int port, const char *client_id) {
    WSADATA wsa;
    SOCKET sock;
    struct sockaddr_in server;

    WSAStartup(MAKEWORD(2,2), &wsa);
    sock = socket(AF_INET, SOCK_STREAM, 0);

    server.sin_addr.s_addr = inet_addr(ip);
    server.sin_family = AF_INET;
    server.sin_port = htons(port);

    if (connect(sock, (struct sockaddr *)&server, sizeof(server)) == 0) {
        send(sock, client_id, strlen(client_id), 0);
        printf("[HEARTBEAT] Sent from %s\n", client_id);
    } else {
        printf("[HEARTBEAT] Root unreachable\n");
    }

    closesocket(sock);
    WSACleanup();
}
