#include "test_helper.h"
#include "interface/http_parser.h"

static void test_parse_get_root(void) {
    HttpRequest req = http_parse("GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");

    ASSERT_STR_EQ(req.method, "GET");
    ASSERT_STR_EQ(req.path,   "/");
}

static void test_parse_post_with_path(void) {
    HttpRequest req = http_parse("POST /api/users HTTP/1.1\r\n\r\n");

    ASSERT_STR_EQ(req.method, "POST");
    ASSERT_STR_EQ(req.path,   "/api/users");
}

static void test_parse_empty_string_yields_empty_fields(void) {
    HttpRequest req = http_parse("");

    ASSERT_STR_EQ(req.method, "");
    ASSERT_STR_EQ(req.path,   "");
}

int main(void) {
    printf("interface/http_parser\n");
    RUN(test_parse_get_root);
    RUN(test_parse_post_with_path);
    RUN(test_parse_empty_string_yields_empty_fields);
    SUITE_RESULTS();
}
