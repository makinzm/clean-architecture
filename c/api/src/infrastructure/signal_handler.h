#ifndef SIGNAL_HANDLER_H
#define SIGNAL_HANDLER_H

#include <signal.h>

extern volatile sig_atomic_t stop_server;

int setup_signal_handlers(void);

#endif
