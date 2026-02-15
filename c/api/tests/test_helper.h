#ifndef TEST_HELPER_H
#define TEST_HELPER_H

#include <stdio.h>
#include <string.h>

static int _tests_run    = 0;
static int _tests_failed = 0;
static int _test_failed  = 0;

#define ASSERT(cond)                                                           \
    do {                                                                       \
        if (!(cond)) {                                                         \
            fprintf(stderr, "      %s:%d  (%s)\n",                            \
                    __FILE__, __LINE__, #cond);                                \
            _test_failed = 1;                                                  \
        }                                                                      \
    } while (0)

#define ASSERT_INT_EQ(got, expected)                                           \
    do {                                                                       \
        int _g = (int)(got), _e = (int)(expected);                            \
        if (_g != _e) {                                                        \
            fprintf(stderr, "      %s:%d  expected %d, got %d\n",             \
                    __FILE__, __LINE__, _e, _g);                               \
            _test_failed = 1;                                                  \
        }                                                                      \
    } while (0)

#define ASSERT_STR_EQ(got, expected)                                           \
    do {                                                                       \
        if (strcmp((got), (expected)) != 0) {                                  \
            fprintf(stderr, "      %s:%d  expected \"%s\", got \"%s\"\n",     \
                    __FILE__, __LINE__, (expected), (got));                    \
            _test_failed = 1;                                                  \
        }                                                                      \
    } while (0)

#define ASSERT_STR_CONTAINS(haystack, needle)                                  \
    do {                                                                       \
        if (strstr((haystack), (needle)) == NULL) {                            \
            fprintf(stderr, "      %s:%d  \"%s\" not in \"%s\"\n",            \
                    __FILE__, __LINE__, (needle), (haystack));                 \
            _test_failed = 1;                                                  \
        }                                                                      \
    } while (0)

#define RUN(fn)                                                                \
    do {                                                                       \
        _test_failed = 0;                                                      \
        (fn)();                                                                \
        _tests_run++;                                                          \
        if (_test_failed) {                                                    \
            _tests_failed++;                                                   \
            printf("  [FAIL] " #fn "\n");                                      \
        } else {                                                               \
            printf("  [PASS] " #fn "\n");                                      \
        }                                                                      \
    } while (0)

#define SUITE_RESULTS()                                                        \
    do {                                                                       \
        printf("\n%d/%d passed\n", _tests_run - _tests_failed, _tests_run);   \
        return (_tests_failed == 0) ? 0 : 1;                                  \
    } while (0)

#endif
