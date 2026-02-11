#include "logger.h"
#include <stdio.h>
#include <time.h>
#include <windows.h>

static FILE *log_file = NULL;
static CRITICAL_SECTION log_lock;

static void get_timestamp(char *buffer, size_t size) {
    time_t now = time(NULL);
    struct tm tm_info;
    localtime_s(&tm_info, &now);
    strftime(buffer, size, "%Y-%m-%d %H:%M:%S", &tm_info);
}

void logger_init(const char *filename) {
    InitializeCriticalSection(&log_lock);
    log_file = fopen(filename, "a");
    if (!log_file) {
        printf("Failed to open log file\n");
    }
}

void log_event(const char *module,
               const char *event_type,
               const char *metadata) {

    if (!log_file) return;

    char timestamp[64];
    get_timestamp(timestamp, sizeof(timestamp));

    EnterCriticalSection(&log_lock);

    fprintf(log_file, "%s|%s|%s|%s\n",
            module,
            timestamp,
            event_type,
            metadata ? metadata : "");

    fflush(log_file);

    LeaveCriticalSection(&log_lock);
}

void logger_shutdown(){
    if(log_file){
        fclose(log_file);
    }
    DeleteCriticalSection(&log_lock);
}