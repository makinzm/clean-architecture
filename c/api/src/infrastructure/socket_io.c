#include "socket_io.h"
#include <sys/socket.h>
#include <unistd.h>

ssize_t real_read(int fd, char *buf, size_t len) {
    return recv(fd, buf, len, 0);
}

void real_write(int fd, const char *buf, size_t len) {
    send(fd, buf, len, 0);
}

void real_sleep(unsigned int seconds) {
    sleep(seconds);
}
