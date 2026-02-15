#include "http_parser.h"
#include <stdio.h>
#include <string.h>

HttpRequest http_parse(const char *raw) {
    HttpRequest req;
    memset(&req, 0, sizeof(req));
    sscanf(raw, "%15s %255s", req.method, req.path);

    const char *bearer = strstr(raw, "Authorization: Bearer ");
    if (bearer) {
        bearer += strlen("Authorization: Bearer ");
        const char *end = strstr(bearer, "\r\n");
        if (end) {
            size_t len = (size_t)(end - bearer);
            if (len >= sizeof(req.auth_token)) len = sizeof(req.auth_token) - 1;
            memcpy(req.auth_token, bearer, len);
            req.auth_token[len] = '\0';
        } else {
            strncpy(req.auth_token, bearer, sizeof(req.auth_token) - 1);
        }
    }

    return req;
}
