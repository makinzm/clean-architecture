CC     = gcc
CFLAGS = -Wall -Wextra -Isrc
TARGET = main.out

SRCS = \
	src/main.c \
	src/domain/http_request.c \
	src/domain/http_response.c \
	src/usecase/handle_client.c \
	src/usecase/route_request.c \
	src/interface/http_parser.c \
	src/interface/http_formatter.c \
	src/infrastructure/socket_io.c \
	src/infrastructure/signal_handler.c \
	src/infrastructure/server.c

OBJS = $(SRCS:.c=.o)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

# ===== tests =====

TEST_FLAGS = $(CFLAGS) -Itests

DOMAIN_SRCS = src/domain/http_request.c src/domain/http_response.c
IFACE_SRCS  = src/interface/http_parser.c src/interface/http_formatter.c

TEST_BINS = \
	tests/domain/test_http_request.out \
	tests/domain/test_http_response.out \
	tests/interface/test_http_parser.out \
	tests/interface/test_http_formatter.out \
	tests/usecase/test_route_request.out \
	tests/usecase/test_handle_client.out

tests/domain/test_http_request.out: tests/domain/test_http_request.c $(DOMAIN_SRCS)
	$(CC) $(TEST_FLAGS) -o $@ $^

tests/domain/test_http_response.out: tests/domain/test_http_response.c $(DOMAIN_SRCS)
	$(CC) $(TEST_FLAGS) -o $@ $^

tests/interface/test_http_parser.out: tests/interface/test_http_parser.c \
		src/interface/http_parser.c $(DOMAIN_SRCS)
	$(CC) $(TEST_FLAGS) -o $@ $^

tests/interface/test_http_formatter.out: tests/interface/test_http_formatter.c \
		src/interface/http_formatter.c $(DOMAIN_SRCS)
	$(CC) $(TEST_FLAGS) -o $@ $^

tests/usecase/test_route_request.out: tests/usecase/test_route_request.c \
		src/usecase/route_request.c $(DOMAIN_SRCS)
	$(CC) $(TEST_FLAGS) -o $@ $^

tests/usecase/test_handle_client.out: tests/usecase/test_handle_client.c \
		src/usecase/handle_client.c src/usecase/route_request.c \
		$(IFACE_SRCS) $(DOMAIN_SRCS)
	$(CC) $(TEST_FLAGS) -o $@ $^

test: $(TEST_BINS)
	@status=0; \
	for t in $(TEST_BINS); do \
		echo; ./$$t || status=1; \
	done; \
	exit $$status

clean:
	rm -f $(OBJS) $(TARGET) $(TEST_BINS)

.PHONY: clean test
