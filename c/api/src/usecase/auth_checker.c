#include "auth_checker.h"
#include <string.h>

int check_auth(const char *token, const AuthConfig *config) {
    if (!config->enabled) return 1;
    if (!token || strlen(token) == 0) return 0;
    return config->verify_fn(token, config->secret);
}
