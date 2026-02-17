#ifndef LOGGER_H
#define LOGGER_H

void logger_init(const char *filename);
void log_event(const char *module,
               const char *event_type,
               const char *metadata);
void logger_shutdown();

#endif
