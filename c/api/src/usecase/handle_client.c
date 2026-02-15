#include "handle_client.h"
#include "route_request.h"
#include "../interface/http_parser.h"
#include "../interface/http_formatter.h"
#include <stdio.h>
#include <string.h>

void handle_client_connection(int client_fd, const handler_config_t *config) {
    printf("Accepted a connection.\n");
    config->io->sleep_fn(config->delay_seconds);

    char buffer[1024];
    ssize_t bytes = config->io->read_from_client(client_fd, buffer, sizeof(buffer) - 1);
    if (bytes < 0) {
        perror("recv");
        return;
    }
    buffer[bytes] = '\0';
    printf("Received data: %s\n", buffer);

    HttpRequest req = http_parse(buffer);
    HttpResponse res = route_request(&req);
    const char *formatted = http_format(&res);
    config->io->write_to_client(client_fd, formatted, strlen(formatted));
    printf("Sent response to client.\n");
}
