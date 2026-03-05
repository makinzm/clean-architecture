import { describe, it, expect } from 'vitest';
import { Game } from './Game.js';
import { Square } from './Square.js';
import { PieceColor, PieceType } from './Piece.js';

describe('Game Entity', () => {
    it('should initialize a new game properly', () => {
        const game = Game.create();
        expect(game.turn).toBe(PieceColor.WHITE);
        expect(game.isCheckmate()).toBe(false);
        expect(game.isStalemate()).toBe(false);
    });

    it('should allow valid pawn moves for white initially', () => {
        const game = Game.create();
        const legalMoves = game.getLegalMoves(Square.fromString('e2'));

        expect(legalMoves.some(m => m.to.equals(Square.fromString('e3')))).toBe(true);
        expect(legalMoves.some(m => m.to.equals(Square.fromString('e4')))).toBe(true);
    });

    it('should apply a valid move and switch turn', () => {
        let game = Game.create();
        game = game.applyMove(Square.fromString('e2'), Square.fromString('e4'));

        expect(game.turn).toBe(PieceColor.BLACK);
        expect(game.board.getPieceAt(Square.fromString('e4'))?.type).toBe(PieceType.PAWN);
    });

    it('should throw error on invalid move (wrong turn)', () => {
        const game = Game.create();
        expect(() => game.applyMove(Square.fromString('e7'), Square.fromString('e5'))).toThrow('Invalid move');
    });
});
