#ifndef HANDLE_CLIENT_H
#define HANDLE_CLIENT_H

#include <sys/types.h>

// I/O port: defined here (usecase), implemented by infrastructure
typedef ssize_t (*read_fn_t)(int fd, char *buf, size_t len);
typedef void    (*write_fn_t)(int fd, const char *buf, size_t len);
typedef void    (*sleep_fn_t)(unsigned int seconds);

typedef struct {
    read_fn_t  read_from_client;
    write_fn_t write_to_client;
    sleep_fn_t sleep_fn;
} io_operations_t;

typedef struct {
    const io_operations_t *io;
    unsigned int delay_seconds;
} handler_config_t;

void handle_client_connection(int client_fd, const handler_config_t *config);

#endif
