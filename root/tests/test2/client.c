/*
 * Sentinel Test Client
 *
 * Usage:
 *   gcc -o client client.c
 *   ./client <node_id> [server_ip] [server_port]
 *
 * Examples:
 *   ./client 1                     → connect as Node 1 to 127.0.0.1:8080
 *   ./client 2                     → connect as Node 2 to 127.0.0.1:8080
 *   ./client 1 127.0.0.1 19090    → connect as Node 1 to port 19090 (test config)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>

#define DEFAULT_SERVER_IP   "127.0.0.1"
#define DEFAULT_SERVER_PORT 8080
#define BUFFER_SIZE         512

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Usage: %s <node_id> [server_ip] [server_port]\n", argv[0]);
        printf("  node_id     : your node ID (must match config)\n");
        printf("  server_ip   : default %s\n", DEFAULT_SERVER_IP);
        printf("  server_port : default %d\n", DEFAULT_SERVER_PORT);
        return 1;
    }

    int node_id = atoi(argv[1]);
    const char *server_ip = (argc >= 3) ? argv[2] : DEFAULT_SERVER_IP;
    int server_port = (argc >= 4) ? atoi(argv[3]) : DEFAULT_SERVER_PORT;

    int sock;
    struct sockaddr_in server_addr;
    char buffer[BUFFER_SIZE] = {0};

    /* Create socket */
    sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0) {
        perror("Socket creation failed");
        return 1;
    }

    /* Setup server address */
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(server_port);
    inet_pton(AF_INET, server_ip, &server_addr.sin_addr);

    /* Connect */
    if (connect(sock, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        perror("Connection failed");
        return 1;
    }

    printf("Connected to %s:%d\n", server_ip, server_port);

    /* Send handshake: HELLO <node_id>\n */
    char hello[64];
    snprintf(hello, sizeof(hello), "HELLO %d\n", node_id);
    if (send(sock, hello, strlen(hello), 0) < 0) {
        perror("Handshake send failed");
        close(sock);
        return 1;
    }

    printf("Registered as Node %d\n", node_id);
    printf("Type messages (or Ctrl+C to disconnect):\n\n");

    /* Interactive loop — read from stdin, send to server */
    while (1) {
        printf("node %d > ", node_id);
        fflush(stdout);

        if (fgets(buffer, BUFFER_SIZE, stdin) == NULL) {
            break; /* EOF / Ctrl+D */
        }

        /* Send message to server */
        if (send(sock, buffer, strlen(buffer), 0) < 0) {
            perror("Send failed");
            break;
        }
    }

    printf("\nDisconnecting Node %d...\n", node_id);
    close(sock);
    return 0;
}
