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
make client

# run tests
make test

# remove all build artefacts
make clean
```

---

## Registering a Client (RS256)

Authentication uses RS256 (asymmetric RSA). Each client has its own key pair.
The server stores the **public key**; the client keeps the **private key**.

### 1. Generate a key pair for a client (e.g. `alice`)

```bash
# generate 2048-bit RSA private key
openssl genrsa -out alice_priv.pem 2048

# extract public key
openssl rsa -in alice_priv.pem -pubout -out src/infrastructure/keys/alice.pem
```

- `alice_priv.pem` — kept by the client (never committed; ignored by `.gitignore`)
- `src/infrastructure/keys/alice.pem` — registered on the server

### 2. Verify the key pair is correct

```bash
openssl rsa -in alice_priv.pem -check -noout
# → RSA key ok
```

---

## Run

### Server

```bash
# auth disabled (default)
./main.out

# auth enabled — keys_dir defaults to src/infrastructure/keys
AUTH_ENABLED=1 ./main.out

# auth enabled with a custom keys directory
AUTH_ENABLED=1 KEYS_DIR=/path/to/keys ./main.out
```

### Client

```bash
# generate a JWT token for CLIENT_ID (default: "client")
PRIVKEY_PATH=alice_priv.pem CLIENT_ID=alice ./client/client.out generate

# send a valid request → 200 OK
PRIVKEY_PATH=alice_priv.pem CLIENT_ID=alice ./client/client.out request /

# send a request with an invalid token → 401 Unauthorized
./client/client.out request-bad /
```

### curl (auth disabled)

```bash
curl http://localhost:9999
curl http://localhost:9999/hello
```

### curl (auth enabled)

```bash
TOKEN=$(PRIVKEY_PATH=alice_priv.pem CLIENT_ID=alice ./client/client.out generate)
curl -H "Authorization: Bearer $TOKEN" http://localhost:9999
```

---

## Environment Variables

### Server

| Variable       | Description                                                  | Default                      |
|----------------|--------------------------------------------------------------|------------------------------|
| `AUTH_ENABLED` | Set to `1` to enable JWT auth                                | disabled                     |
| `KEYS_DIR`     | Directory containing `<client_id>.pem` public key files      | `src/infrastructure/keys`    |

### Client

| Variable       | Description                                                  | Default    |
|----------------|--------------------------------------------------------------|------------|
| `PRIVKEY_PATH` | Path to the client's RSA private key PEM file (**required** for `generate`/`request`) | — |
| `CLIENT_ID`    | The `sub` claim in the JWT; must match a `<CLIENT_ID>.pem` file in `KEYS_DIR` | `client` |
| `SERVER_PORT`  | Port to connect to                                           | `9999`     |
