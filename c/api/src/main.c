#include "infrastructure/signal_handler.h"
#include "infrastructure/server.h"
#include "infrastructure/socket_io.h"
#include "infrastructure/jwt.h"
#include "usecase/handle_client.h"
#include <stdio.h>
#include <unistd.h>
#include <signal.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/socket.h>

#define PORT 9999

int main(void) {
    const char *auth_enabled_str = getenv("AUTH_ENABLED");
    const char *keys_dir         = getenv("KEYS_DIR");
    int auth_enabled = (auth_enabled_str && strcmp(auth_enabled_str, "1") == 0);
    if (!keys_dir) keys_dir = "src/infrastructure/keys";

    // dependency injection: wire infrastructure implementations to usecase ports
    io_operations_t production_io = {
        .read_from_client = real_read,
        .write_to_client  = real_write,
        .sleep_fn         = real_sleep,
    };

    handler_config_t config = {
        .io            = &production_io,
        .delay_seconds = 5,
        .auth = {
            .enabled   = auth_enabled,
            .keys_dir  = keys_dir,
            .verify_fn = jwt_verify,
        },
    };

    if (setup_signal_handlers() < 0) {
        return 1;
    }

    int sockfd = create_and_bind_socket(PORT);
    if (sockfd < 0) {
        return 1;
    }

    printf("Server is listening on port %d...\n", PORT);
    printf("Authentication: %s\n", auth_enabled ? "ENABLED" : "DISABLED");

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
