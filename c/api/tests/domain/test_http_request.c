#include "test_helper.h"
#include "domain/http_request.h"

static void test_init_zeroes_all_fields(void) {
    HttpRequest req;
    req.method[0] = 'X';
    req.path[0]   = 'Y';

    http_request_init(&req);

    ASSERT_STR_EQ(req.method, "");
    ASSERT_STR_EQ(req.path,   "");
}

int main(void) {
    printf("domain/http_request\n");
    RUN(test_init_zeroes_all_fields);
    SUITE_RESULTS();
}
