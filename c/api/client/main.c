#include "../src/infrastructure/jwt.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

static int tcp_connect(int port) {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) { perror("socket"); return -1; }

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port   = htons((uint16_t)port);
    inet_pton(AF_INET, "127.0.0.1", &addr.sin_addr);

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        perror("connect");
        close(fd);
        return -1;
    }
    return fd;
}

static void send_request(const char *path, const char *token, int port) {
    int fd = tcp_connect(port);
    if (fd < 0) return;

    char req[2048];
    snprintf(req, sizeof(req),
        "GET %s HTTP/1.1\r\n"
        "Host: localhost\r\n"
        "Authorization: Bearer %s\r\n"
        "Connection: close\r\n"
        "\r\n",
        path, token);

    send(fd, req, strlen(req), 0);

    char buf[4096];
    ssize_t n;
    while ((n = recv(fd, buf, sizeof(buf) - 1, 0)) > 0) {
        buf[n] = '\0';
        printf("%s", buf);
    }
    printf("\n");
    close(fd);
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr,
            "Usage:\n"
            "  %s generate\n"
            "  %s request PATH\n"
            "  %s request-bad PATH\n",
            argv[0], argv[0], argv[0]);
        return 1;
    }

    const char *privkey_path = getenv("PRIVKEY_PATH");
    const char *client_id    = getenv("CLIENT_ID");
    if (!client_id) client_id = "client";

    const char *port_str = getenv("SERVER_PORT");
    int port = port_str ? atoi(port_str) : 9999;

    if (strcmp(argv[1], "generate") == 0) {
        if (!privkey_path) {
            fprintf(stderr, "Error: PRIVKEY_PATH is not set.\n");
            return 1;
        }
        char *token = jwt_generate(client_id, privkey_path, 3600);
        if (!token) { fprintf(stderr, "jwt_generate failed\n"); return 1; }
        printf("%s\n", token);
        free(token);
        return 0;
    }

    const char *path = (argc >= 3) ? argv[2] : "/";

    if (strcmp(argv[1], "request") == 0) {
        if (!privkey_path) {
            fprintf(stderr, "Error: PRIVKEY_PATH is not set.\n");
            return 1;
        }
        char *token = jwt_generate(client_id, privkey_path, 3600);
        if (!token) { fprintf(stderr, "jwt_generate failed\n"); return 1; }
        send_request(path, token, port);
        free(token);

    } else if (strcmp(argv[1], "request-bad") == 0) {
        send_request(path, "invalid.token.here", port);

    } else {
        fprintf(stderr, "Unknown command: %s\n", argv[1]);
        return 1;
    }

    return 0;
}
