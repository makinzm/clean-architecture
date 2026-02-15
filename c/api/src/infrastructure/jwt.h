#ifndef JWT_H
#define JWT_H

/* RS256: sign JWT with RSA private key PEM file.
 * Returns heap-allocated token (caller must free), or NULL on error. */
char *jwt_generate(const char *subject, const char *privkey_path, long exp_seconds);

/* RS256: verify JWT with RSA public key PEM file.
 * Returns 1 if valid and not expired, 0 otherwise. */
int jwt_verify(const char *token, const char *pubkey_path);

#endif
