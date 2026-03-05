import { describe, it, expect } from 'vitest';
import { GetGameStateUseCase } from './GetGameStateUseCase.js';
import { GameRepository } from '../ports/GameRepository.js';
import { Game } from '@chess/domain';

class MockGameRepo implements GameRepository {
    games: Map<string, Game> = new Map();
    async save(game: Game) { }
    async findById(id: string) {
        if (id === '123') return Game.create();
        return null;
    }
    async findAll() { return []; }
}

describe('GetGameStateUseCase', () => {
    it('should return game state for an existing game', async () => {
        const useCase = new GetGameStateUseCase(new MockGameRepo());

        const state = await useCase.execute('123');
        expect(state).toBeDefined();
        expect(state.turn).toBe('w');
    });

    it('should throw error for non-existent game', async () => {
        const useCase = new GetGameStateUseCase(new MockGameRepo());

        await expect(useCase.execute('999')).rejects.toThrow('Game not found');
    });
});
