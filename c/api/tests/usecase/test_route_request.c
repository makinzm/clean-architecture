#include "test_helper.h"
#include "usecase/route_request.h"
#include <string.h>

static void test_get_root_returns_200_with_body(void) {
    HttpRequest req;
    strcpy(req.method, "GET");
    strcpy(req.path,   "/");

    HttpResponse res = route_request(&req);

    ASSERT_INT_EQ(res.status_code, 200);
    ASSERT_STR_EQ(res.body, "Hello, World!");
}

static void test_get_unknown_path_returns_404(void) {
    HttpRequest req;
    strcpy(req.method, "GET");
    strcpy(req.path,   "/unknown");

    HttpResponse res = route_request(&req);

    ASSERT_INT_EQ(res.status_code, 404);
}

static void test_post_root_returns_404(void) {
    HttpRequest req;
    strcpy(req.method, "POST");
    strcpy(req.path,   "/");

    HttpResponse res = route_request(&req);

    ASSERT_INT_EQ(res.status_code, 404);
}

int main(void) {
    printf("usecase/route_request\n");
    RUN(test_get_root_returns_200_with_body);
    RUN(test_get_unknown_path_returns_404);
    RUN(test_post_root_returns_404);
    SUITE_RESULTS();
}
