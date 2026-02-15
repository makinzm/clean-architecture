#ifndef HTTP_RESPONSE_H
#define HTTP_RESPONSE_H

typedef struct {
    int status_code;
    const char *body;
} HttpResponse;

void http_response_init(HttpResponse *res);

#endif
