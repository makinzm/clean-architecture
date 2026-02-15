#ifndef ROUTE_REQUEST_H
#define ROUTE_REQUEST_H

#include "../domain/http_request.h"
#include "../domain/http_response.h"

HttpResponse route_request(const HttpRequest *req);

#endif
