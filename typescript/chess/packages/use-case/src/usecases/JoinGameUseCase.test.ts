import { describe, it, expect } from 'vitest';
import { JoinGameUseCase } from './JoinGameUseCase.js';
import { GameRepository } from '../ports/GameRepository.js';
import { Game } from '@chess/domain';

class MockGameRepo implements GameRepository {
    games: Map<string, Game> = new Map();
    async save(game: Game) {
        // Basic mock logic assuming we have some way to track ID
        // Since Game doesn't have an ID property in our current simple domain
        // We'll just store it statically for the test. 
    }
    async findById(id: string) {
        if (id === '123') return Game.create();
        return null;
    }
    async findAll() { return []; }
}

describe('JoinGameUseCase', () => {
    it('should allow joining an existing game', async () => {
        const repo = new MockGameRepo();
        const useCase = new JoinGameUseCase(repo);

        const result = await useCase.execute('123', 'player1');
        expect(result.success).toBe(true);
    });

    it('should throw error if game not found', async () => {
        const repo = new MockGameRepo();
        const useCase = new JoinGameUseCase(repo);

        await expect(useCase.execute('999', 'player1')).rejects.toThrow('Game not found');
    });
});
