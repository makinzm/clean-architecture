#include "test_helper.h"
#include "usecase/auth_checker.h"

/* ---- stub verify functions ---- */

static int stub_verify_ok(const char *token, const char *secret) {
    (void)token; (void)secret;
    return 1;
}

static int stub_verify_fail(const char *token, const char *secret) {
    (void)token; (void)secret;
    return 0;
}

/* ---- tests ---- */

static void test_auth_disabled_always_authorized(void) {
    AuthConfig cfg = { .enabled = 0, .keys_dir = "/tmp", .verify_fn = stub_verify_fail };
    ASSERT_INT_EQ(check_auth("", &cfg), 1);
    ASSERT_INT_EQ(check_auth(NULL, &cfg), 1);
}

static void test_auth_enabled_valid_token_authorized(void) {
    AuthConfig cfg = { .enabled = 1, .keys_dir = "/tmp", .verify_fn = stub_verify_ok };
    ASSERT_INT_EQ(check_auth("sometoken", &cfg), 1);
}

static void test_auth_enabled_empty_token_unauthorized(void) {
    AuthConfig cfg = { .enabled = 1, .keys_dir = "/tmp", .verify_fn = stub_verify_ok };
    ASSERT_INT_EQ(check_auth("", &cfg), 0);
}

static void test_auth_enabled_invalid_token_unauthorized(void) {
    AuthConfig cfg = { .enabled = 1, .keys_dir = "/tmp", .verify_fn = stub_verify_fail };
    ASSERT_INT_EQ(check_auth("badtoken", &cfg), 0);
}

int main(void) {
    printf("usecase/auth_checker\n");
    RUN(test_auth_disabled_always_authorized);
    RUN(test_auth_enabled_valid_token_authorized);
    RUN(test_auth_enabled_empty_token_unauthorized);
    RUN(test_auth_enabled_invalid_token_unauthorized);
    SUITE_RESULTS();
}
