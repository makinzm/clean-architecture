import { GameRepository } from '../ports/GameRepository.js';

export interface JoinGameResult {
    success: boolean;
    gameId: string;
}

export class JoinGameUseCase {
    constructor(private readonly gameRepository: GameRepository) { }

    async execute(gameId: string, playerId: string): Promise<JoinGameResult> {
        const game = await this.gameRepository.findById(gameId);
        if (!game) {
            throw new Error('Game not found');
        }

        // In a real implementation we'd probably add the player to the game logic
        // For now we just check if the game exists and "succeed".

        return {
            success: true,
            gameId
        };
    }
}
