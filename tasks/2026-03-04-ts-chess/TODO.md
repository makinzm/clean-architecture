# Task: Chess Clean Architecture (TypeScript)

## Overview
`typescript/chess/` ディレクトリにフルルール対応のオンラインチェスを Clean Architecture で実装する。

## Goals
- ローカル起動を目的とした in-memory インフラ層のスタブ実装
- 最終ターゲットは Cloudflare (Workers + Durable Objects + KV)
- TDD (test-first) で実装する
- 将来的にAtomic ChessやChess960などのバリエーションもサポートできる設計

## Directory Structure
```
typescript/chess/
├── pnpm-workspace.yaml
├── package.json            # volta pins + root scripts
├── tsconfig.base.json
├── vitest.workspace.ts
├── packages/
│   ├── domain/             # @chess/domain — 外部依存ゼロ
│   ├── use-case/           # @chess/use-case — domain に依存
│   ├── infrastructure/     # @chess/infrastructure — domain + use-case に依存
│   └── shared/             # @chess/shared — DTOs & mappers
├── apps/
│   ├── server/             # @chess/server — Hono (ローカル: node-server)
│   └── client/             # @chess/client — Vite + Vanilla TS
└── tasks/
    └── 20260304-chess/
        ├── TODO.md
        └── timeline.md
```

## Implementation Phases

### Phase 1: Domain Layer (`@chess/domain`)
- [x] Square value object + tests
- [x] Piece value object + tests
- [x] Board (createInitialBoard, applyMove) + tests
- [x] Game logic (isInCheck, getLegalMoves, getCastlingMoves, getEnPassantMoves, isCheckmate, isStalemate, applyGameMove) + tests
- [x] DomainErrors

### Phase 2: Use Case Layer (`@chess/use-case`)
- [x] CreateGameUseCase + tests
- [x] JoinGameUseCase + tests
- [x] MakeMoveUseCase + tests
- [x] GetGameStateUseCase + tests
- [x] ListGamesUseCase + tests
- [x] NotificationPort interface

### Phase 3: Infrastructure Layer (`@chess/infrastructure`)
- [x] InMemoryGameRepository + tests
- [x] InMemoryPlayerRepository + tests
- [x] LocalWebSocketManager (NotificationPort impl)
- [x] Cloudflare stubs (TODO comments only)

### Phase 4: Server (`apps/server`)
- [x] Hono HTTP API handlers + tests
- [x] WebSocket endpoint
- [x] Error mapping middleware
- [x] AppState composition root

### Phase 5: Client (`apps/client`)
- [x] Vite + Vanilla TS chess board UI
- [x] Render board squares and pieces
- [x] Handle piece selection and movement UI
### Phase 6: CI
- [x] GitHub Actions workflow for TypeScript chess

## API Endpoints
```
POST   /api/games              → createGame
POST   /api/games/:id/join     → joinGame
POST   /api/games/:id/moves    → makeMove
GET    /api/games/:id          → getGameState
GET    /api/games              → listGames
WS     /ws/games/:id           → WebSocket upgrade
```

## Key Design Decisions
- Clean Architecture: domain ← use-case ← infrastructure ← server
- NotificationPort in use-case layer (outbound port)
- TransactionManager not needed (in-memory, no transactions)
- AppState holds use case instances, composed in server layer
