#include "http_parser.h"
#include <stdio.h>
#include <string.h>

HttpRequest http_parse(const char *raw) {
    HttpRequest req;
    memset(&req, 0, sizeof(req));
    sscanf(raw, "%15s %255s", req.method, req.path);
    return req;
}
