# C API

## Prerequisites

Install [devbox](https://www.jetify.com/devbox) and enter the shell from the repo root:

```bash
devbox shell   # installs gcc, openssl, pkg-config automatically
```

All `make` commands below must be run **inside `devbox shell`**.

---

## Build

```bash
# server binary → main.out
make

# client binary → client/client.out
make client/client.out

# run tests
make test

# remove all build artefacts
make clean
```

---

## Run

### Server

```bash
# auth disabled (default)
./main.out

# auth enabled  (secret must be ≥ 32 bytes)
AUTH_ENABLED=1 JWT_SECRET=a-string-secret-at-least-256-bits-long ./main.out
```

### Client

```bash
# generate a JWT token  (secret must be ≥ 32 bytes)
JWT_SECRET=a-string-secret-at-least-256-bits-long ./client/client.out generate

# send a valid request  → 200 OK
JWT_SECRET=a-string-secret-at-least-256-bits-long ./client/client.out request /

# send a request with an invalid token → 401 Unauthorized
JWT_SECRET=a-string-secret-at-least-256-bits-long ./client/client.out request-bad /
```

### curl (auth disabled)

```bash
curl http://localhost:9999
curl http://localhost:9999/hello
```

### curl (auth enabled)

```bash
SECRET=a-string-secret-at-least-256-bits-long
TOKEN=$(JWT_SECRET=$SECRET ./client/client.out generate)
curl -H "Authorization: Bearer $TOKEN" http://localhost:9999
```

---

## Environment Variables

| Variable       | Description                                          | Default    |
|----------------|------------------------------------------------------|------------|
| `AUTH_ENABLED` | Set to `1` to enable JWT auth                        | disabled   |
| `JWT_SECRET`   | Secret key for signing/verifying (**≥ 32 bytes required** when auth is enabled) | `"secret"` |
| `SERVER_PORT`  | Port used by the client                              | `9999`     |

> **Note**: `JWT_SECRET` must be at least 32 bytes (256 bits) for HS256.
> Both server and client will exit with an error if the secret is too short.
>
> ```bash
> # generate a strong secret
> openssl rand -base64 32
> ```
