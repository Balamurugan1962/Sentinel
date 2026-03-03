#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <pthread.h>

#define SERVER_IP "127.0.0.1"
#define SERVER_PORT 8080
#define BUFFER_SIZE 1024

int sockfd;

void *read_from_server(void *arg) {
    char buffer[BUFFER_SIZE];

    while (1) {
        ssize_t bytes = recv(sockfd, buffer, BUFFER_SIZE - 1, 0);
        if (bytes <= 0) {
            printf("Disconnected from server\n");
            close(sockfd);
            exit(0);
        }

        buffer[bytes] = '\0';
        printf("Server: %s", buffer);
        fflush(stdout);
    }

    return NULL;
}

int main() {
    struct sockaddr_in server_addr;
    pthread_t reader_thread;

    sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) {
        perror("Socket creation failed");
        return 1;
    }

    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(SERVER_PORT);

    if (inet_pton(AF_INET, SERVER_IP, &server_addr.sin_addr) <= 0) {
        perror("Invalid address");
        return 1;
    }

    if (connect(sockfd, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        perror("Connection failed");
        return 1;
    }

    printf("Connected to TCP daemon\n");

    // Create thread to read from server
    pthread_create(&reader_thread, NULL, read_from_server, NULL);

    // Main thread writes user input
    char input[BUFFER_SIZE];
    while (1) {
        if (fgets(input, BUFFER_SIZE, stdin) == NULL) {
            break;
        }

        send(sockfd, input, strlen(input), 0);
    }

    close(sockfd);
    return 0;
}
