import { describe, it, expect } from 'vitest';
import { DomainError, InvalidMoveError, GameAlreadyOverError, NotYourTurnError } from './DomainErrors.js';

describe('Domain Errors', () => {
    it('should create correctly and preserve instanceof', () => {
        const invalidMove = new InvalidMoveError();

        expect(invalidMove).toBeInstanceOf(DomainError);
        expect(invalidMove).toBeInstanceOf(InvalidMoveError);
        expect(invalidMove.name).toBe('InvalidMoveError');
        expect(invalidMove.message).toBe('Invalid move');
    });

    it('should handle custom messages', () => {
        const over = new GameAlreadyOverError('Checkmate!');
        expect(over.message).toBe('Checkmate!');

        const notYourTurn = new NotYourTurnError('Wait for black');
        expect(notYourTurn.message).toBe('Wait for black');
    });
});
