#include "http_response.h"

void http_response_init(HttpResponse *res) {
    res->status_code = 200;
    res->body = "";
}
