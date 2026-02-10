#include "config_loader.h"
#include <stdio.h>
#include <string.h>

int load_config(const char *filename, Config *config) {
    FILE *file = fopen(filename, "r");
    if (!file) return -1;

    char line[256];
    while (fgets(line, sizeof(line), file)) {
        if (strstr(line, "id")) {
            sscanf(line, "id = \"%[^\"]\"", config->client_id);
        } else if (strstr(line, "ip")) {
            sscanf(line, "ip = \"%[^\"]\"", config->root_ip);
        } else if (strstr(line, "port")) {
            sscanf(line, "port = %d", &config->root_port);
        } else if (strstr(line, "interval_seconds")) {
            sscanf(line, "interval_seconds = %d", &config->heartbeat_interval);
        }
    }

    fclose(file);
    return 0;
}
