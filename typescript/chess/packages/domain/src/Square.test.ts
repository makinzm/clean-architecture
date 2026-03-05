import { describe, it, expect } from 'vitest';
import { Square } from './Square.js';

describe('Square Value Object', () => {
    it('should create a valid square from algebraic notation', () => {
        const square = Square.fromString('e4');
        expect(square.file).toBe('e');
        expect(square.rank).toBe(4);
        expect(square.toString()).toBe('e4');
    });

    it('should throw error for invalid squares', () => {
        expect(() => Square.fromString('e9')).toThrow('Invalid square: e9');
        expect(() => Square.fromString('i4')).toThrow('Invalid square: i4');
        expect(() => Square.fromString('e')).toThrow('Invalid square: e');
    });

    it('should check equality correctly', () => {
        const sq1 = Square.fromString('e4');
        const sq2 = Square.fromString('e4');
        const sq3 = Square.fromString('d4');

        expect(sq1.equals(sq2)).toBe(true);
        expect(sq1.equals(sq3)).toBe(false);
    });
});
