#include "infrastructure/signal_handler.h"
#include "infrastructure/server.h"
#include "infrastructure/socket_io.h"
#include "usecase/handle_client.h"
#include <stdio.h>
#include <unistd.h>
#include <signal.h>
#include <stdlib.h>
#include <sys/types.h>
#include <sys/socket.h>

#define PORT 9999

int main(void) {
    // dependency injection: wire infrastructure implementations to usecase ports
    io_operations_t production_io = {
        .read_from_client = real_read,
        .write_to_client  = real_write,
        .sleep_fn         = real_sleep,
    };

    handler_config_t config = {
        .io            = &production_io,
        .delay_seconds = 5,
    };

    if (setup_signal_handlers() < 0) {
        return 1;
    }

    int sockfd = create_and_bind_socket(PORT);
    if (sockfd < 0) {
        return 1;
    }

    printf("Server is listening on port %d...\n", PORT);

    while (!stop_server) {
        int client_fd = accept(sockfd, NULL, NULL);
        if (client_fd < 0) continue;

        pid_t pid = fork();
        if (pid == 0) {
            // child ignores SIGINT/SIGTERM so that sleep always runs to completion
            signal(SIGINT, SIG_IGN);
            signal(SIGTERM, SIG_IGN);
            close(sockfd);
            handle_client_connection(client_fd, &config);
            close(client_fd);
            exit(0);
        }
        close(client_fd);
    }

    cleanup_server(sockfd);
    return 0;
}
