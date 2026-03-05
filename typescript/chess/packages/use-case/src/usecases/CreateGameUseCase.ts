import { GameRepository } from '../ports/GameRepository.js';
import { Game } from '@chess/domain';

export class CreateGameUseCase {
    constructor(private readonly gameRepository: GameRepository) { }

    async execute(): Promise<Game> {
        const game = Game.create();

        await this.gameRepository.save(game);
        return game;
    }
}
