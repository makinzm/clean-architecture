#include "test_helper.h"
#include "infrastructure/jwt.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <sys/stat.h>
#include <unistd.h>
#include <openssl/evp.h>
#include <openssl/pem.h>

#define KEYS_DIR  "/tmp/jwt_rs256_keys"
#define SUB1      "client1"
#define SUB2      "client2"
#define PRIV1     KEYS_DIR "/client1_priv.pem"
#define PUB1      KEYS_DIR "/" SUB1 ".pem"
#define PRIV2     KEYS_DIR "/client2_priv.pem"
#define PUB2      KEYS_DIR "/" SUB2 ".pem"

static EVP_PKEY *g_key1 = NULL;
static EVP_PKEY *g_key2 = NULL;

static EVP_PKEY *gen_rsa_key(void) {
    EVP_PKEY_CTX *ctx = EVP_PKEY_CTX_new_id(EVP_PKEY_RSA, NULL);
    if (!ctx) return NULL;
    EVP_PKEY_keygen_init(ctx);
    EVP_PKEY_CTX_set_rsa_keygen_bits(ctx, 2048);
    EVP_PKEY *key = NULL;
    EVP_PKEY_keygen(ctx, &key);
    EVP_PKEY_CTX_free(ctx);
    return key;
}

static void write_priv(const char *path, EVP_PKEY *key) {
    FILE *fp = fopen(path, "w");
    if (fp) { PEM_write_PrivateKey(fp, key, NULL, NULL, 0, NULL, NULL); fclose(fp); }
}

static void write_pub(const char *path, EVP_PKEY *key) {
    FILE *fp = fopen(path, "w");
    if (fp) { PEM_write_PUBKEY(fp, key); fclose(fp); }
}

static void setup(void) {
    mkdir(KEYS_DIR, 0700);
    g_key1 = gen_rsa_key();
    g_key2 = gen_rsa_key();
    write_priv(PRIV1, g_key1);
    write_pub(PUB1,   g_key1);
    write_priv(PRIV2, g_key2);
    write_pub(PUB2,   g_key2);
}

static void teardown(void) {
    remove(PRIV1); remove(PUB1);
    remove(PRIV2); remove(PUB2);
    rmdir(KEYS_DIR);
    EVP_PKEY_free(g_key1); g_key1 = NULL;
    EVP_PKEY_free(g_key2); g_key2 = NULL;
}

/* {"alg":"RS256","typ":"JWT"} base64url = eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9 */
static void test_header_is_standard_rs256(void) {
    char *token = jwt_generate(SUB1, PRIV1, 3600);
    ASSERT(token != NULL);
    const char *dot = strchr(token, '.');
    ASSERT(dot != NULL);
    size_t hdr_len = (size_t)(dot - token);
    char hdr[128] = {0};
    memcpy(hdr, token, hdr_len);
    ASSERT_STR_EQ(hdr, "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9");
    free(token);
}

static void test_generate_and_verify_valid_token(void) {
    char *token = jwt_generate(SUB1, PRIV1, 3600);
    ASSERT(token != NULL);
    ASSERT_INT_EQ(jwt_verify(token, KEYS_DIR), 1);
    free(token);
}

/* Sign with client2's private key but claim sub=client1.
 * keys_dir/client1.pem is client1's public key → mismatch → fail. */
static void test_verify_fails_with_wrong_key(void) {
    char *token = jwt_generate(SUB1, PRIV2, 3600);
    ASSERT(token != NULL);
    ASSERT_INT_EQ(jwt_verify(token, KEYS_DIR), 0);
    free(token);
}

static void test_verify_fails_with_tampered_signature(void) {
    char *token = jwt_generate(SUB1, PRIV1, 3600);
    ASSERT(token != NULL);
    size_t len = strlen(token);
    token[len - 1] = (token[len - 1] == 'A') ? 'B' : 'A';
    ASSERT_INT_EQ(jwt_verify(token, KEYS_DIR), 0);
    free(token);
}

static void test_verify_fails_empty_token(void) {
    ASSERT_INT_EQ(jwt_verify("", KEYS_DIR), 0);
}

static void test_verify_fails_expired_token(void) {
    char *token = jwt_generate(SUB1, PRIV1, -1);
    ASSERT(token != NULL);
    ASSERT_INT_EQ(jwt_verify(token, KEYS_DIR), 0);
    free(token);
}

int main(void) {
    printf("infrastructure/jwt (RS256)\n");
    setup();
    RUN(test_header_is_standard_rs256);
    RUN(test_generate_and_verify_valid_token);
    RUN(test_verify_fails_with_wrong_key);
    RUN(test_verify_fails_with_tampered_signature);
    RUN(test_verify_fails_empty_token);
    RUN(test_verify_fails_expired_token);
    teardown();
    SUITE_RESULTS();
}
