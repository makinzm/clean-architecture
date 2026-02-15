#ifndef JWT_H
#define JWT_H

/* Returns heap-allocated JWT string (caller must free), or NULL on error. */
char *jwt_generate(const char *subject, const char *secret, long exp_seconds);

/* Returns 1 if valid, 0 if invalid or expired. */
int jwt_verify(const char *token, const char *secret);

#endif
