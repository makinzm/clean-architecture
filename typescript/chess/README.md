# Clean Architecture Chess (TypeScript)

A full-rules chess game built with Clean Architecture in TypeScript.
Includes a domain model separated from use cases, infrastructure, and an API/WebSocket server, with a Vite + Vanilla TS front end.

## Architecture

- **`packages/domain`**: Pure TypeScript domain logic (Entities, Value Objects). No external dependencies.
- **`packages/use-case`**: Application use cases (CreateGame, MakeMove, etc.) and port interfaces.
- **`packages/infrastructure`**: Adapters for ports (InMemory repositories, WebSocket notification manager).
- **`apps/server`**: Hono-based HTTP API and WebSocket server.
- **`apps/client`**: Vite + Vanilla TS chess board UI.

## Prerequisites
- [Node.js](https://nodejs.org/) (version 20.x or later)
- [pnpm](https://pnpm.io/) (version 9.x)
- [Volta](https://volta.sh/) (Optional, but recommended. Will automatically use the pinned node/pnpm versions.)

## Setup

1. Install dependencies:
   ```bash
   pnpm install
   ```

2. Build all packages in the workspace:
   ```bash
   pnpm -r build
   ```

## Running the Application Locally

To play the game locally, you need to run both the API Server and the Frontend Client.

### 1. Start the Server
Open a terminal and run the server in development mode:
```bash
cd apps/server
pnpm dev
```
The server will start listening on `http://localhost:3000`.

### 2. Start the Client
Open a second terminal and start the Vite development server for the client:
```bash
cd apps/client
pnpm dev
```
The client will start on `http://localhost:5173` (or similar). Open this URL in your browser. API requests to `/api` are automatically proxied to the server.

## Testing

The project follows Test-Driven Development (TDD). To run all unit tests across the workspace:
```bash
pnpm test
```
