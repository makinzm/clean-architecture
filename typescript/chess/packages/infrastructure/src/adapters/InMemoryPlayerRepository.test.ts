import { describe, it, expect } from 'vitest';
import { InMemoryPlayerRepository } from './InMemoryPlayerRepository.js';

describe('InMemoryPlayerRepository', () => {
    it('should save and find a player by id', async () => {
        const repo = new InMemoryPlayerRepository();
        const player = { id: 'p1', name: 'Alice' };

        await repo.save(player);

        const retrieved = await repo.findById('p1');
        expect(retrieved).toBeDefined();
        expect(retrieved?.name).toBe('Alice');
    });

    it('should return null for non-existent player', async () => {
        const repo = new InMemoryPlayerRepository();
        const retrieved = await repo.findById('p2');
        expect(retrieved).toBeNull();
    });
});
