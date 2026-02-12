#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>

#define SERVER_IP "127.0.0.1"
#define SERVER_PORT 8080
#define BUFFER_SIZE 512

int main(int argc, char *argv[]) {
    if (argc != 2) {
        printf("Usage: %s <client_port>\n", argv[0]);
        return 1;
    }

    int client_port = atoi(argv[1]);

    if (client_port <= 0 || client_port > 65535) {
        printf("Invalid client port\n");
        return 1;
    }

    int sock;
    struct sockaddr_in server_addr, client_addr;
    char buffer[BUFFER_SIZE] = {0};

    // Create socket
    sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0) {
        perror("Socket creation failed");
        return 1;
    }

    // -----------------------
    // Bind client to specific port
    // -----------------------
    client_addr.sin_family = AF_INET;
    client_addr.sin_port = htons(client_port);
    client_addr.sin_addr.s_addr = inet_addr("127.0.0.1");

    if (bind(sock, (struct sockaddr *)&client_addr, sizeof(client_addr)) < 0) {
        perror("Client bind failed");
        return 1;
    }

    // -----------------------
    // Setup server address
    // -----------------------
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(SERVER_PORT);
    inet_pton(AF_INET, SERVER_IP, &server_addr.sin_addr);

    // Connect to server
    if (connect(sock, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        perror("Connection failed");
        return 1;
    }

    printf("Client bound to port %d\n", client_port);
    printf("Connected to root at %s:%d\n", SERVER_IP, SERVER_PORT);

    while (1) {

    }
    // Send message
    char *message = "Hello from C client";
    send(sock, message, strlen(message), 0);

    // Receive reply
    int bytes_read = read(sock, buffer, BUFFER_SIZE);
    printf("Server replied: %.*s\n", bytes_read, buffer);

    close(sock);
    return 0;
}
