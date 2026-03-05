import { GameRepository } from '../ports/GameRepository.js';
import { Game } from '@chess/domain';

export class ListGamesUseCase {
    constructor(private readonly gameRepository: GameRepository) { }

    async execute(): Promise<Game[]> {
        return this.gameRepository.findAll();
    }
}
