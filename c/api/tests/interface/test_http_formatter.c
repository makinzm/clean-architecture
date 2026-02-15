#include "test_helper.h"
#include "interface/http_formatter.h"

static void test_format_200_contains_status_line_and_body(void) {
    HttpResponse res = { 200, "Hello, World!" };
    const char *out  = http_format(&res);

    ASSERT_STR_CONTAINS(out, "HTTP/1.1 200 OK");
    ASSERT_STR_CONTAINS(out, "Hello, World!");
}

static void test_format_404_contains_status_line_and_body(void) {
    HttpResponse res = { 404, "Not Found" };
    const char *out  = http_format(&res);

    ASSERT_STR_CONTAINS(out, "HTTP/1.1 404 Not Found");
    ASSERT_STR_CONTAINS(out, "Not Found");
}

static void test_format_includes_content_type_header(void) {
    HttpResponse res = { 200, "" };
    const char *out  = http_format(&res);

    ASSERT_STR_CONTAINS(out, "Content-Type: text/plain");
}

int main(void) {
    printf("interface/http_formatter\n");
    RUN(test_format_200_contains_status_line_and_body);
    RUN(test_format_404_contains_status_line_and_body);
    RUN(test_format_includes_content_type_header);
    SUITE_RESULTS();
}
