#include "signal_handler.h"
#include <sys/wait.h>
#include <string.h>
#include <stdio.h>

volatile sig_atomic_t stop_server = 0;

static void handle_signal(int signum) {
    (void)signum;
    stop_server = 1;
}

static void handle_sigchld(int signum) {
    (void)signum;
    while (waitpid(-1, NULL, WNOHANG) > 0);
}

int setup_signal_handlers(void) {
    struct sigaction sa;
    memset(&sa, 0, sizeof(sa));
    sa.sa_handler = handle_signal;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = 0; // no SA_RESTART: SIGINT interrupts accept() to stop the loop

    if (sigaction(SIGINT, &sa, NULL) < 0 || sigaction(SIGTERM, &sa, NULL) < 0) {
        perror("sigaction");
        return -1;
    }

    struct sigaction sa_chld;
    memset(&sa_chld, 0, sizeof(sa_chld));
    sa_chld.sa_handler = handle_sigchld;
    sigemptyset(&sa_chld.sa_mask);
    sa_chld.sa_flags = SA_RESTART; // SA_RESTART: SIGCHLD does not interrupt accept()
    if (sigaction(SIGCHLD, &sa_chld, NULL) < 0) {
        perror("sigaction SIGCHLD");
        return -1;
    }

    return 0;
}
