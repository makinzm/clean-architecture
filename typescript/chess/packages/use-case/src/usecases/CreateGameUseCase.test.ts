import { describe, it, expect } from 'vitest';
import { CreateGameUseCase } from './CreateGameUseCase.js';
import { GameRepository } from '../ports/GameRepository.js';
import { Game } from '@chess/domain';

class MockGameRepo implements GameRepository {
    games: Game[] = [];
    async save(game: Game) {
        this.games.push(game);
    }
    async findById() { return null; }
    async findAll() { return this.games; }
}

describe('CreateGameUseCase', () => {
    it('should create a new game and save it to the repository', async () => {
        const repo = new MockGameRepo();
        const useCase = new CreateGameUseCase(repo);

        const game = await useCase.execute();

        expect(game).toBeDefined();
        expect(repo.games.length).toBe(1);
        expect(game.turn).toBe('w'); // WHITE
    });
});
