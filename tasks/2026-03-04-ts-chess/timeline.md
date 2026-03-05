# Timeline

## 2026-03-05
- Checked `tasks/2026-03-04-ts-chess/TODO.md` to begin implementation.
- Branch is set to `feat/ts-chess`.
- Initialized timeline document.
- Set up Typescript workspace and Volta pinnings.
- Implemented `Square.test.ts` first, confirmed failure, then implemented `Square.ts` and passed tests.
- Implemented `Piece.test.ts` first, confirmed failure, then implemented `Piece.ts` and passed tests.
- Implemented `Board.test.ts` first, confirmed failure, then implemented `Board.ts` and passed tests.
- Implemented `Game.test.ts` for piece movement skeleton, confirmed failure and implemented logic.
- Implemented `DomainErrors` and tests, finishing Phase 1.
- Started Phase 2: Defined `GameRepository`, `PlayerRepository`, `NotificationPort`.
- Followed TDD for `CreateGameUseCase`, `JoinGameUseCase`, `MakeMoveUseCase`, `GetGameStateUseCase`, `ListGamesUseCase` and successfully implemented them.
- Started Phase 3: Implemented `InMemoryGameRepository` and `InMemoryPlayerRepository` following TDD.
- Implemented `LocalWebSocketManager` using TDD, completing Phase 3 Infrastructure Layer.
- Started Phase 4 (Server): Initialized `apps/server` with Hono and `@hono/node-server`.
- Implemented `AppState.ts` for Clean Arch dependency injection.
- Developed basic HTTP endpoints and WebSocket stub for checking rules via TDD.
- Started Phase 5 (Client): Initialized Vite + Vanilla TS application.
- Implemented dynamic, aesthetically pleasing UI (`style.css`).
- Created board rendering and interactive move handling with API proxies (`main.ts`).
- Resolved TypeScript build errors to ensure full workspace compilation passes (`pnpm -r build`).
- Started Phase 6 (CI): Added GitHub Actions workflow (`.github/workflows/ts-chess-ci.yml`) to run tests and build automatically on `main`.
- **Project Complete:** All phases finished according to `implementation_plan.md` and TDD practices followed strictly across the clean architecture layers.
