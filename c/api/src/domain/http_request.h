#ifndef HTTP_REQUEST_H
#define HTTP_REQUEST_H

typedef struct {
    char method[16];
    char path[256];
} HttpRequest;

void http_request_init(HttpRequest *req);

#endif
