import { describe, it, expect, beforeEach } from 'vitest';
import { Hono } from 'hono';
import { app } from './index.js';

describe('Chess HTTP API', () => {
    it('should create a game', async () => {
        const res = await app.request('/api/games', { method: 'POST' });
        expect(res.status).toBe(201);
        const json = await res.json() as any;
        expect(json.game).toBeDefined();
        expect(json.game.turn).toBe('w');
    });

    it('should list games', async () => {
        const res = await app.request('/api/games', { method: 'GET' });
        expect(res.status).toBe(200);
        const json = await res.json() as any;
        expect(Array.isArray(json.games)).toBe(true);
    });
});
