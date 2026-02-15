#include "jwt.h"
#include <openssl/evp.h>
#include <openssl/pem.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <time.h>

/* ---- base64url (no padding) ---- */

static const char B64URL[] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

static void b64url_encode(const unsigned char *src, size_t src_len, char *dst) {
    size_t i = 0, j = 0;
    while (i + 2 < src_len) {
        dst[j++] = B64URL[(src[i] >> 2) & 0x3F];
        dst[j++] = B64URL[((src[i] & 0x03) << 4) | ((src[i+1] >> 4) & 0x0F)];
        dst[j++] = B64URL[((src[i+1] & 0x0F) << 2) | ((src[i+2] >> 6) & 0x03)];
        dst[j++] = B64URL[src[i+2] & 0x3F];
        i += 3;
    }
    if (src_len - i == 1) {
        dst[j++] = B64URL[(src[i] >> 2) & 0x3F];
        dst[j++] = B64URL[(src[i] & 0x03) << 4];
    } else if (src_len - i == 2) {
        dst[j++] = B64URL[(src[i] >> 2) & 0x3F];
        dst[j++] = B64URL[((src[i] & 0x03) << 4) | ((src[i+1] >> 4) & 0x0F)];
        dst[j++] = B64URL[(src[i+1] & 0x0F) << 2];
    }
    dst[j] = '\0';
}

static int b64val(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '-') return 62;
    if (c == '_') return 63;
    return -1;
}

static size_t b64url_decode(const char *src, unsigned char *dst) {
    size_t src_len = strlen(src);
    size_t full    = src_len / 4;
    size_t rem     = src_len % 4;
    size_t j       = 0;

    for (size_t i = 0; i < full * 4; i += 4) {
        int v0 = b64val(src[i]);
        int v1 = b64val(src[i+1]);
        int v2 = b64val(src[i+2]);
        int v3 = b64val(src[i+3]);
        if (v0 < 0 || v1 < 0 || v2 < 0 || v3 < 0) return 0;
        dst[j++] = (unsigned char)((v0 << 2) | (v1 >> 4));
        dst[j++] = (unsigned char)(((v1 & 0x0F) << 4) | (v2 >> 2));
        dst[j++] = (unsigned char)(((v2 & 0x03) << 6) | v3);
    }

    size_t i = full * 4;
    if (rem == 2) {
        int v0 = b64val(src[i]), v1 = b64val(src[i+1]);
        if (v0 < 0 || v1 < 0) return 0;
        dst[j++] = (unsigned char)((v0 << 2) | (v1 >> 4));
    } else if (rem == 3) {
        int v0 = b64val(src[i]), v1 = b64val(src[i+1]), v2 = b64val(src[i+2]);
        if (v0 < 0 || v1 < 0 || v2 < 0) return 0;
        dst[j++] = (unsigned char)((v0 << 2) | (v1 >> 4));
        dst[j++] = (unsigned char)(((v1 & 0x0F) << 4) | (v2 >> 2));
    }
    return j;
}

/* ---- RSA key helpers ---- */

static EVP_PKEY *load_private_key(const char *path) {
    FILE *fp = fopen(path, "r");
    if (!fp) { perror(path); return NULL; }
    EVP_PKEY *key = PEM_read_PrivateKey(fp, NULL, NULL, NULL);
    fclose(fp);
    return key;
}

static EVP_PKEY *load_public_key(const char *path) {
    FILE *fp = fopen(path, "r");
    if (!fp) { perror(path); return NULL; }
    EVP_PKEY *key = PEM_read_PUBKEY(fp, NULL, NULL, NULL);
    fclose(fp);
    return key;
}

/* ---- public API ---- */

char *jwt_generate(const char *subject, const char *privkey_path, long exp_seconds) {
    /* Fixed header for RS256 */
    const char *header_json = "{\"alg\":\"RS256\",\"typ\":\"JWT\"}";

    char payload_json[512];
    long exp = (long)time(NULL) + exp_seconds;
    snprintf(payload_json, sizeof(payload_json),
             "{\"sub\":\"%s\",\"exp\":%ld}", subject, exp);

    char hdr_b64[128];
    char pay_b64[768];
    b64url_encode((const unsigned char *)header_json, strlen(header_json), hdr_b64);
    b64url_encode((const unsigned char *)payload_json, strlen(payload_json), pay_b64);

    char signing[1024];
    snprintf(signing, sizeof(signing), "%s.%s", hdr_b64, pay_b64);

    EVP_PKEY *pkey = load_private_key(privkey_path);
    if (!pkey) return NULL;

    EVP_MD_CTX *ctx = EVP_MD_CTX_new();
    if (!ctx) { EVP_PKEY_free(pkey); return NULL; }

    if (EVP_DigestSignInit(ctx, NULL, EVP_sha256(), NULL, pkey) != 1 ||
        EVP_DigestSignUpdate(ctx, signing, strlen(signing)) != 1) {
        EVP_MD_CTX_free(ctx); EVP_PKEY_free(pkey); return NULL;
    }

    size_t sig_len = 0;
    EVP_DigestSignFinal(ctx, NULL, &sig_len);
    unsigned char *sig = malloc(sig_len);
    if (!sig) { EVP_MD_CTX_free(ctx); EVP_PKEY_free(pkey); return NULL; }
    EVP_DigestSignFinal(ctx, sig, &sig_len);

    EVP_MD_CTX_free(ctx);
    EVP_PKEY_free(pkey);

    /* base64url encode signature (RSA-2048: 256 bytes → 342 chars) */
    size_t sig_b64_size = (sig_len * 4 / 3) + 4;
    char *sig_b64 = malloc(sig_b64_size);
    if (!sig_b64) { free(sig); return NULL; }
    b64url_encode(sig, sig_len, sig_b64);
    free(sig);

    size_t total = strlen(hdr_b64) + 1 + strlen(pay_b64) + 1 + strlen(sig_b64) + 1;
    char *token = malloc(total);
    if (!token) { free(sig_b64); return NULL; }
    snprintf(token, total, "%s.%s.%s", hdr_b64, pay_b64, sig_b64);
    free(sig_b64);

    return token;
}

/* Extract a JSON string value: returns 1 on success, 0 on failure.
 * e.g. extract_json_str(json, "sub", out, sizeof(out)) */
static int extract_json_str(const char *json, const char *key,
                             char *out, size_t out_size) {
    char needle[64];
    snprintf(needle, sizeof(needle), "\"%s\":\"", key);
    const char *p = strstr(json, needle);
    if (!p) return 0;
    p += strlen(needle);
    const char *end = strchr(p, '"');
    if (!end) return 0;
    size_t len = (size_t)(end - p);
    if (len >= out_size) return 0;
    memcpy(out, p, len);
    out[len] = '\0';
    return 1;
}

int jwt_verify(const char *token, const char *keys_dir) {
    if (!token || !keys_dir || token[0] == '\0') return 0;

    const char *dot1 = strchr(token, '.');
    if (!dot1) return 0;
    const char *dot2 = strchr(dot1 + 1, '.');
    if (!dot2) return 0;

    /* decode payload to extract "sub" */
    size_t pay_b64_len = (size_t)(dot2 - dot1 - 1);
    if (pay_b64_len >= 768) return 0;
    char pay_b64[768];
    memcpy(pay_b64, dot1 + 1, pay_b64_len);
    pay_b64[pay_b64_len] = '\0';

    unsigned char payload[512];
    size_t payload_len = b64url_decode(pay_b64, payload);
    if (payload_len == 0) return 0;
    payload[payload_len] = '\0';

    /* look up public key: <keys_dir>/<sub>.pem */
    char sub[128];
    if (!extract_json_str((char *)payload, "sub", sub, sizeof(sub))) return 0;

    char pubkey_path[512];
    snprintf(pubkey_path, sizeof(pubkey_path), "%s/%s.pem", keys_dir, sub);

    /* signing input = header.payload */
    size_t signing_len = (size_t)(dot2 - token);
    if (signing_len >= 1024) return 0;
    char signing[1024];
    memcpy(signing, token, signing_len);
    signing[signing_len] = '\0';

    /* decode signature */
    const char *sig_b64 = dot2 + 1;
    unsigned char *sig = malloc(strlen(sig_b64) + 1);
    if (!sig) return 0;
    size_t sig_len = b64url_decode(sig_b64, sig);
    if (sig_len == 0) { free(sig); return 0; }

    EVP_PKEY *pkey = load_public_key(pubkey_path);
    if (!pkey) { free(sig); return 0; }

    EVP_MD_CTX *ctx = EVP_MD_CTX_new();
    if (!ctx) { free(sig); EVP_PKEY_free(pkey); return 0; }

    int result = 0;
    if (EVP_DigestVerifyInit(ctx, NULL, EVP_sha256(), NULL, pkey) == 1 &&
        EVP_DigestVerifyUpdate(ctx, signing, signing_len) == 1) {
        result = (EVP_DigestVerifyFinal(ctx, sig, sig_len) == 1) ? 1 : 0;
    }

    EVP_MD_CTX_free(ctx);
    EVP_PKEY_free(pkey);
    free(sig);

    if (!result) return 0;

    /* check expiration */
    const char *exp_str = strstr((char *)payload, "\"exp\":");
    if (exp_str) {
        long exp_val = 0;
        sscanf(exp_str + 6, "%ld", &exp_val);
        if (exp_val > 0 && (long)time(NULL) > exp_val) return 0;
    }

    return 1;
}
