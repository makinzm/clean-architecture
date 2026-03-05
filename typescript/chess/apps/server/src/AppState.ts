import { CreateGameUseCase, GetGameStateUseCase, JoinGameUseCase, ListGamesUseCase, MakeMoveUseCase } from '@chess/use-case';
import { InMemoryGameRepository, InMemoryPlayerRepository, LocalWebSocketManager } from '@chess/infrastructure';

export interface AppState {
    createGameUseCase: CreateGameUseCase;
    getGameStateUseCase: GetGameStateUseCase;
    joinGameUseCase: JoinGameUseCase;
    listGamesUseCase: ListGamesUseCase;
    makeMoveUseCase: MakeMoveUseCase;
    wsManager: LocalWebSocketManager;
}

export function buildDependencies(): AppState {
    const gameRepo = new InMemoryGameRepository();
    const playerRepo = new InMemoryPlayerRepository();
    const wsManager = new LocalWebSocketManager();

    return {
        createGameUseCase: new CreateGameUseCase(gameRepo),
        getGameStateUseCase: new GetGameStateUseCase(gameRepo),
        joinGameUseCase: new JoinGameUseCase(gameRepo),
        listGamesUseCase: new ListGamesUseCase(gameRepo),
        makeMoveUseCase: new MakeMoveUseCase(gameRepo, wsManager),
        wsManager,
    };
}
