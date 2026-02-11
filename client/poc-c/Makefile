CC=gcc.exe
CFLAGS=-Wall
LIBS=-lws2_32

all:
	$(CC) main.c config_loader.c heartbeat.c \
	logger/logger.c \
	-o sentinel-client.exe $(LIBS)
