#include "route_request.h"
#include <string.h>

HttpResponse route_request(const HttpRequest *req) {
    if (strcmp(req->method, "GET") == 0 && strcmp(req->path, "/") == 0) {
        HttpResponse res = { 200, "Hello, World!" };
        return res;
    }
    HttpResponse res = { 404, "Not Found" };
    return res;
}
