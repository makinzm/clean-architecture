#include "http_formatter.h"
#include <stdio.h>

const char *http_format(const HttpResponse *res) {
    static char buf[4096];
    const char *status_text = (res->status_code == 200) ? "OK" : "Not Found";
    snprintf(buf, sizeof(buf),
        "HTTP/1.1 %d %s\r\nContent-Type: text/plain\r\n\r\n%s",
        res->status_code, status_text, res->body);
    return buf;
}
