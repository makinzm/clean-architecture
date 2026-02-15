#ifndef AUTH_CHECKER_H
#define AUTH_CHECKER_H

typedef int (*verify_fn_t)(const char *token, const char *secret);

typedef struct {
    int          enabled;
    const char  *secret;
    verify_fn_t  verify_fn;
} AuthConfig;

/* Returns 1 (authorized) or 0 (unauthorized). */
int check_auth(const char *token, const AuthConfig *config);

#endif
