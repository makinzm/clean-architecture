#include "http_formatter.h"
#include <stdio.h>

const char *http_format(const HttpResponse *res) {
    static char buf[4096];
    const char *status_text;
    if (res->status_code == 200) {
        status_text = "OK";
    } else if (res->status_code == 401) {
        status_text = "Unauthorized";
    } else {
        status_text = "Not Found";
    }
    snprintf(buf, sizeof(buf),
        "HTTP/1.1 %d %s\r\nContent-Type: text/plain\r\n\r\n%s",
        res->status_code, status_text, res->body);
    return buf;
}
