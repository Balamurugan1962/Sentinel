#ifndef CONFIG_LOADER_H
#define CONFIG_LOADER_H

#define MAX_IP_LEN 64

typedef struct {
    char client_id[64];
    char root_ip[MAX_IP_LEN];
    int root_port;
    int heartbeat_interval;
} Config;

int load_config(const char *filename, Config *config);

#endif
