CC      = gcc
TARGET  = main.out

OPENSSL_CFLAGS = $(shell pkg-config --cflags openssl 2>/dev/null)
OPENSSL_LIBS   = $(shell pkg-config --libs openssl 2>/dev/null || echo "-lssl -lcrypto")

CFLAGS  = -Wall -Wextra -Isrc $(OPENSSL_CFLAGS)

SRCS = \
	src/main.c \
	src/domain/http_request.c \
	src/domain/http_response.c \
	src/usecase/handle_client.c \
	src/usecase/route_request.c \
	src/usecase/auth_checker.c \
	src/interface/http_parser.c \
	src/interface/http_formatter.c \
	src/infrastructure/socket_io.c \
	src/infrastructure/signal_handler.c \
	src/infrastructure/server.c \
	src/infrastructure/jwt.c

OBJS = $(SRCS:.c=.o)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^ $(OPENSSL_LIBS)

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

# ===== client =====

client: client/client.out

client/client.out: client/main.c src/infrastructure/jwt.c
	$(CC) $(CFLAGS) -o $@ $^ $(OPENSSL_LIBS)

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
	tests/usecase/test_handle_client.out \
	tests/infrastructure/test_jwt.out \
	tests/usecase/test_auth_checker.out

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
		src/usecase/auth_checker.c \
		$(IFACE_SRCS) $(DOMAIN_SRCS)
	$(CC) $(TEST_FLAGS) -o $@ $^

tests/infrastructure/test_jwt.out: tests/infrastructure/test_jwt.c \
		src/infrastructure/jwt.c
	$(CC) $(TEST_FLAGS) -o $@ $^ $(OPENSSL_LIBS)

tests/usecase/test_auth_checker.out: tests/usecase/test_auth_checker.c \
		src/usecase/auth_checker.c
	$(CC) $(TEST_FLAGS) -o $@ $^

test: $(TEST_BINS)
	@status=0; \
	for t in $(TEST_BINS); do \
		echo; ./$$t || status=1; \
	done; \
	exit $$status

clean:
	rm -f $(OBJS) $(TARGET) $(TEST_BINS) client/client.out

.PHONY: clean test client
