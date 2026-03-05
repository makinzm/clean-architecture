import { GameRepository } from '../ports/GameRepository.js';
import { Game } from '@chess/domain';

export class GetGameStateUseCase {
    constructor(private readonly gameRepository: GameRepository) { }

    async execute(gameId: string): Promise<Game> {
        const game = await this.gameRepository.findById(gameId);
        if (!game) {
            throw new Error('Game not found');
        }
        return game;
    }
}
