#include "jwt.h"
#include <openssl/hmac.h>
#include <openssl/evp.h>
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

/* ---- HMAC-SHA256 helper ---- */

static void hmac_sha256(const char *key, const char *msg, size_t msg_len,
                        unsigned char out[32]) {
    unsigned int out_len = 32;
    HMAC(EVP_sha256(),
         key, (int)strlen(key),
         (const unsigned char *)msg, msg_len,
         out, &out_len);
}

/* ---- public API ---- */

char *jwt_generate(const char *subject, const char *secret, long exp_seconds) {
    /* Fixed header for HS256 */
    const char *header_json = "{\"alg\":\"HS256\",\"typ\":\"JWT\"}";

    char payload_json[512];
    long exp = (long)time(NULL) + exp_seconds;
    snprintf(payload_json, sizeof(payload_json),
             "{\"sub\":\"%s\",\"exp\":%ld}", subject, exp);

    /* base64url-encode header and payload */
    char hdr_b64[128];
    char pay_b64[768];
    b64url_encode((const unsigned char *)header_json, strlen(header_json), hdr_b64);
    b64url_encode((const unsigned char *)payload_json, strlen(payload_json), pay_b64);

    /* signing input = header_b64 "." payload_b64 */
    char signing[1024];
    snprintf(signing, sizeof(signing), "%s.%s", hdr_b64, pay_b64);

    /* compute signature */
    unsigned char sig_raw[32];
    hmac_sha256(secret, signing, strlen(signing), sig_raw);

    char sig_b64[64];
    b64url_encode(sig_raw, 32, sig_b64);

    /* assemble token */
    size_t total = strlen(hdr_b64) + 1 + strlen(pay_b64) + 1 + strlen(sig_b64) + 1;
    char *token = malloc(total);
    if (!token) return NULL;
    snprintf(token, total, "%s.%s.%s", hdr_b64, pay_b64, sig_b64);
    return token;
}

int jwt_verify(const char *token, const char *secret) {
    if (!token || !secret || token[0] == '\0') return 0;

    /* locate the two '.' separators */
    const char *dot1 = strchr(token, '.');
    if (!dot1) return 0;
    const char *dot2 = strchr(dot1 + 1, '.');
    if (!dot2) return 0;

    /* signing input = everything before the last dot */
    size_t signing_len = (size_t)(dot2 - token);
    if (signing_len >= 1024) return 0;
    char signing[1024];
    memcpy(signing, token, signing_len);
    signing[signing_len] = '\0';

    /* re-compute expected signature */
    unsigned char expected_raw[32];
    hmac_sha256(secret, signing, signing_len, expected_raw);
    char expected_b64[64];
    b64url_encode(expected_raw, 32, expected_b64);

    /* compare with actual signature */
    const char *actual_sig = dot2 + 1;
    if (strcmp(expected_b64, actual_sig) != 0) return 0;

    /* check expiration by decoding payload */
    size_t pay_b64_len = (size_t)(dot2 - dot1 - 1);
    if (pay_b64_len >= 768) return 0;
    char pay_b64[768];
    memcpy(pay_b64, dot1 + 1, pay_b64_len);
    pay_b64[pay_b64_len] = '\0';

    unsigned char payload[512];
    size_t payload_len = b64url_decode(pay_b64, payload);
    if (payload_len == 0) return 0;
    payload[payload_len] = '\0';

    const char *exp_str = strstr((char *)payload, "\"exp\":");
    if (exp_str) {
        long exp = 0;
        sscanf(exp_str + 6, "%ld", &exp);
        if (exp > 0 && (long)time(NULL) > exp) return 0;
    }

    return 1;
}
