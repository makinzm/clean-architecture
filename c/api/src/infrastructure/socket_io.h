#ifndef SOCKET_IO_H
#define SOCKET_IO_H

#include <sys/types.h>

ssize_t real_read(int fd, char *buf, size_t len);
void    real_write(int fd, const char *buf, size_t len);
void    real_sleep(unsigned int seconds);

#endif
