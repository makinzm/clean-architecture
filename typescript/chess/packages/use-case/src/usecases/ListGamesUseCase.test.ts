import { describe, it, expect } from 'vitest';
import { ListGamesUseCase } from './ListGamesUseCase.js';
import { GameRepository } from '../ports/GameRepository.js';
import { Game } from '@chess/domain';

class MockGameRepo implements GameRepository {
    games: Game[] = [Game.create(), Game.create()];
    async save(game: Game) { }
    async findById(id: string) { return null; }
    async findAll() { return this.games; }
}

describe('ListGamesUseCase', () => {
    it('should return a list of games', async () => {
        const useCase = new ListGamesUseCase(new MockGameRepo());
        const games = await useCase.execute();

        expect(games.length).toBe(2);
    });
});
