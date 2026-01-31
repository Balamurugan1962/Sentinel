#include <stdio.h>
#include <windows.h>
#include "config_loader.h"
#include "heartbeat.h"

int main() {
    Config config;

    if (load_config("config.toml", &config) != 0) {
        printf("Failed to load config\n");
        return 1;
    }

    printf("Sentinel Client Started\n");
    printf("Client ID: %s\n", config.client_id);

    while (1) {
        send_heartbeat(config.root_ip, config.root_port, config.client_id);
        Sleep(config.heartbeat_interval * 1000);
    }

    return 0;
}
