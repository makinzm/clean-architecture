#include "test_helper.h"
#include "usecase/handle_client.h"
#include <string.h>

// ----- stubs -----

static char         stub_write_buf[4096];
static const char  *stub_read_data;
static unsigned int stub_sleep_called_with;

static ssize_t stub_read(int fd, char *buf, size_t len) {
    (void)fd;
    size_t n = strlen(stub_read_data);
    if (n >= len) n = len - 1;
    memcpy(buf, stub_read_data, n);
    return (ssize_t)n;
}

static void stub_write(int fd, const char *buf, size_t len) {
    (void)fd;
    if (len >= sizeof(stub_write_buf)) len = sizeof(stub_write_buf) - 1;
    memcpy(stub_write_buf, buf, len);
    stub_write_buf[len] = '\0';
}

static void stub_sleep(unsigned int seconds) {
    stub_sleep_called_with = seconds;
}

static io_operations_t stub_io = {
    .read_from_client = stub_read,
    .write_to_client  = stub_write,
    .sleep_fn         = stub_sleep,
};

// ----- tests -----

static void test_get_root_responds_200(void) {
    stub_read_data = "GET / HTTP/1.1\r\n\r\n";
    memset(stub_write_buf, 0, sizeof(stub_write_buf));

    handler_config_t config = { .io = &stub_io, .delay_seconds = 0 };
    handle_client_connection(0, &config);

    ASSERT_STR_CONTAINS(stub_write_buf, "200 OK");
    ASSERT_STR_CONTAINS(stub_write_buf, "Hello, World!");
}

static void test_unknown_path_responds_404(void) {
    stub_read_data = "GET /unknown HTTP/1.1\r\n\r\n";
    memset(stub_write_buf, 0, sizeof(stub_write_buf));

    handler_config_t config = { .io = &stub_io, .delay_seconds = 0 };
    handle_client_connection(0, &config);

    ASSERT_STR_CONTAINS(stub_write_buf, "404 Not Found");
}

static void test_post_root_responds_404(void) {
    stub_read_data = "POST / HTTP/1.1\r\n\r\n";
    memset(stub_write_buf, 0, sizeof(stub_write_buf));

    handler_config_t config = { .io = &stub_io, .delay_seconds = 0 };
    handle_client_connection(0, &config);

    ASSERT_STR_CONTAINS(stub_write_buf, "404 Not Found");
}

static void test_sleep_called_with_configured_delay(void) {
    stub_read_data        = "GET / HTTP/1.1\r\n\r\n";
    stub_sleep_called_with = 0;

    handler_config_t config = { .io = &stub_io, .delay_seconds = 3 };
    handle_client_connection(0, &config);

    ASSERT_INT_EQ(stub_sleep_called_with, 3);
}

int main(void) {
    printf("usecase/handle_client\n");
    RUN(test_get_root_responds_200);
    RUN(test_unknown_path_responds_404);
    RUN(test_post_root_responds_404);
    RUN(test_sleep_called_with_configured_delay);
    SUITE_RESULTS();
}
