#include <sys/socket.h>
#include <stdio.h>
#include <unistd.h>
#include <string.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <errno.h>

#define PORT 9999

volatile sig_atomic_t stop_server = 0;

void handle_signal(int signum) {
    stop_server = 1;
    // printf is not async-signal-safe, so avoid calling it here
}

void handle_sigchld(int signum) {
    // reap finished children to avoid zombies accumulating during normal operation
    while (waitpid(-1, NULL, WNOHANG) > 0); // from sys/wait.h
}

int main() {
    // set up signal handlers for SIGINT and SIGTERM to allow graceful shutdown of the server when these signals are received.
    struct sigaction sa; // from signal.h
    memset(&sa, 0, sizeof(sa)); // from string.h
    sa.sa_handler = handle_signal; // from signal.h
    sigemptyset(&sa.sa_mask); // from signal.h
    sa.sa_flags = 0; // from signal.h

    if (sigaction(SIGINT, &sa, NULL) < 0 || sigaction(SIGTERM, &sa, NULL) < 0) { // from signal.h
        perror("sigaction"); // from stdio.h
        return 1;
    }

    // reap finished children during normal operation (SA_RESTART: don't interrupt accept)
    struct sigaction sa_chld; // from signal.h
    memset(&sa_chld, 0, sizeof(sa_chld)); // from string.h
    sa_chld.sa_handler = handle_sigchld;
    sigemptyset(&sa_chld.sa_mask); // from signal.h
    sa_chld.sa_flags = SA_RESTART; // from signal.h
    if (sigaction(SIGCHLD, &sa_chld, NULL) < 0) { // from signal.h
        perror("sigaction SIGCHLD"); // from stdio.h
        return 1;
    }
    // fd means file descriptor, which is an integer that uniquely identifies an open file (or socket) in the operating system.
    int sockfd = socket(AF_INET6, SOCK_STREAM, 0); // from sys.socket.h
    if (sockfd < 0) {
        perror("socket"); // from stdio.h
        return 1;
    }

    // accept IPv6 and IPv4 connection
    int opt = 0;
    if (setsockopt(sockfd, IPPROTO_IPV6, IPV6_V6ONLY, &opt, sizeof(opt)) < 0) { // from sys.socket.h and netinet/in.h
        perror("setsockopt"); // from stdio.h
        close(sockfd); // from unistd.h
        return 1;
    }
    // easy to re-open the server after shutdown, without waiting for the socket to be released by the operating system.
    opt = 1;
    if (setsockopt(sockfd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt)) < 0) { // from sys.socket.h
        perror("setsockopt"); // from stdio.h
        close(sockfd); // from unistd.h
        return 1;
    } 

    struct sockaddr_in6 server_addr; // from netinet/in.h
    server_addr.sin6_family = AF_INET6; // from sys.socket.h
    server_addr.sin6_addr = in6addr_any; // from netinet/in.h
    server_addr.sin6_port = htons(PORT); // from netintet/in.h (technically from arpa/inet.h, but included by netinet/in.h)
    
    // bind the socket to the specified port and address. This allows the server to listen for incoming connections on that port.
    if (bind(sockfd, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0) { // from sys.socket.h
        perror("bind"); // from stdio.h
        close(sockfd); // from unistd.h
        return 1;
    }
    if (listen(sockfd, 10) < 0) { // from sys.socket.h
        perror("listen"); // from stdio.h
        close(sockfd); // from unistd.h
        return 1;
    }
    printf("Server is listening on port %d...\n", PORT); // from stdio.h

    while (!stop_server) {
        int client_fd = accept(sockfd, NULL, NULL); // from sys.socket.h
        if (client_fd < 0) {
            continue;
        }
        pid_t pid = fork(); // from unistd.h
        if (pid < 0) {
            perror("fork"); // from stdio.h
            close(client_fd); // from unistd.h
            continue;
        }
        if (pid == 0) { // child process
            // child ignores SIGINT/SIGTERM so that sleep(5) always runs to completion
            signal(SIGINT, SIG_IGN); // from signal.h
            signal(SIGTERM, SIG_IGN); // from signal.h
            // child process does not need the listening socket, so close it to free up resources and prevent potential issues with multiple processes trying to accept connections on the same socket.
            close(sockfd); // from unistd.h
            printf("Accepted a connection.\n"); // from stdio.which
            // to check graceful shutdown, sleep for 5 seconds
            sleep(5); // from unistd.h
            char buffer[1024];
            ssize_t bytes_received = recv(client_fd, buffer, sizeof(buffer) - 1, 0); // from sys.socket.h
            if (bytes_received < 0) {
                perror("recv"); // from stdio.h
                close(client_fd); // from unistd.h
                continue;
            }
            buffer[bytes_received] = '\0'; // Null-terminate the received data
            printf("Received data: %s\n", buffer); // from stdio.h
            char method[16], path[256];
            sscanf(buffer, "%s %s", method, path); // from stdio.h
            if (strcmp(method, "GET") == 0 && strcmp(path, "/") == 0) { // from string.h
                const char *response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
                send(client_fd, response, strlen(response), 0); // from sys.socket.h and string.h
            } else {
                const char *response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\nNot Found";
                send(client_fd, response, strlen(response), 0); // from sys.socket.h and string.h
            }
            close(client_fd); // from unistd.h
            printf("Sent response to client.\n"); // from stdio.h
            exit(0); // from stdlib.h
        }
        close(client_fd); // from unistd.h
    }
    printf("Received signal, shutting down server...\n"); // from stdio.h
    close(sockfd); // from unistd.h
    // wait for ALL child processes to finish before exiting
    while (waitpid(-1, NULL, 0) > 0); // from sys/wait.h
    printf("Server has shut down.\n"); // from stdio.h
    return 0;
}

