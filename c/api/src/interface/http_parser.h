#ifndef HTTP_PARSER_H
#define HTTP_PARSER_H

#include "../domain/http_request.h"

HttpRequest http_parse(const char *raw);

#endif
