#include "test_helper.h"
#include "domain/http_response.h"

static void test_init_sets_default_200_with_empty_body(void) {
    HttpResponse res;
    http_response_init(&res);

    ASSERT_INT_EQ(res.status_code, 200);
    ASSERT_STR_EQ(res.body, "");
}

int main(void) {
    printf("domain/http_response\n");
    RUN(test_init_sets_default_200_with_empty_body);
    SUITE_RESULTS();
}
