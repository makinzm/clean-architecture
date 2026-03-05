import { describe, it, expect, vi } from 'vitest';
import { LocalWebSocketManager } from './LocalWebSocketManager.js';
import { Game } from '@chess/domain';

// A mock WebSocket-like interface for testing
class MockWebSocket {
    messages: string[] = [];
    send(data: string) {
        this.messages.push(data);
    }
}

describe('LocalWebSocketManager', () => {
    it('should broadcast game updates to all connected clients for a specific game', async () => {
        const manager = new LocalWebSocketManager();
        const ws1 = new MockWebSocket() as any;
        const ws2 = new MockWebSocket() as any;

        manager.addClient('game-1', ws1);
        manager.addClient('game-1', ws2);

        const game = Game.create();
        await manager.notifyGameUpdated('game-1', game);

        expect(ws1.messages.length).toBe(1);
        expect(ws2.messages.length).toBe(1);

        const msg = JSON.parse(ws1.messages[0]);
        expect(msg.type).toBe('GAME_UPDATED');
        expect(msg.payload.turn).toBe('w');
    });
});
