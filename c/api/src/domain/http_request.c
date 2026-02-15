#include "http_request.h"
#include <string.h>

void http_request_init(HttpRequest *req) {
    memset(req, 0, sizeof(HttpRequest));
}
