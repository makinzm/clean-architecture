import { describe, it, expect, vi } from 'vitest';
import { MakeMoveUseCase } from './MakeMoveUseCase.js';
import { GameRepository } from '../ports/GameRepository.js';
import { NotificationPort } from '../ports/NotificationPort.js';
import { Game, Square } from '@chess/domain';

class MockGameRepo implements GameRepository {
    games: Map<string, Game> = new Map();
    async save(game: Game) {
        this.games.set('123', game);
    }
    async findById(id: string) {
        if (id === '123') return this.games.get(id) || Game.create();
        return null;
    }
    async findAll() { return Array.from(this.games.values()); }
}

class MockNotificationPort implements NotificationPort {
    async notifyGameUpdated() { }
}

describe('MakeMoveUseCase', () => {
    it('should make a valid move, save state, and notify', async () => {
        const repo = new MockGameRepo();
        await repo.save(Game.create()); // Ensure a game exists
        const notifier = new MockNotificationPort();
        const notifySpy = vi.spyOn(notifier, 'notifyGameUpdated');
        const useCase = new MakeMoveUseCase(repo, notifier);

        const result = await useCase.execute('123', 'player1', 'e2', 'e4');
        expect(result.success).toBe(true);
        expect(notifySpy).toHaveBeenCalledWith('123', expect.any(Object));

        const updatedGame = await repo.findById('123');
        expect(updatedGame?.turn).toBe('b'); // Turn should have switched to black
    });

    it('should fail on invalid move string', async () => {
        const repo = new MockGameRepo();
        const useCase = new MakeMoveUseCase(repo, new MockNotificationPort());

        await expect(useCase.execute('123', 'player1', 'e9', 'e4')).rejects.toThrow();
    });
});
