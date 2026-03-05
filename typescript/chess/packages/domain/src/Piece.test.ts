import { describe, it, expect } from 'vitest';
import { Piece, PieceColor, PieceType } from './Piece.js';

describe('Piece Value Object', () => {
    it('should create a valid piece', () => {
        const piece = new Piece(PieceColor.WHITE, PieceType.PAWN);
        expect(piece.color).toBe(PieceColor.WHITE);
        expect(piece.type).toBe(PieceType.PAWN);
    });

    it('should create from FEN character', () => {
        expect(Piece.fromFenChar('P').equals(new Piece(PieceColor.WHITE, PieceType.PAWN))).toBe(true);
        expect(Piece.fromFenChar('p').equals(new Piece(PieceColor.BLACK, PieceType.PAWN))).toBe(true);
        expect(Piece.fromFenChar('K').equals(new Piece(PieceColor.WHITE, PieceType.KING))).toBe(true);
        expect(Piece.fromFenChar('q').equals(new Piece(PieceColor.BLACK, PieceType.QUEEN))).toBe(true);
    });

    it('should convert to FEN character', () => {
        expect(new Piece(PieceColor.WHITE, PieceType.KNIGHT).toFenChar()).toBe('N');
        expect(new Piece(PieceColor.BLACK, PieceType.ROOK).toFenChar()).toBe('r');
    });

    it('should throw error for invalid FEN character', () => {
        expect(() => Piece.fromFenChar('X')).toThrow('Invalid FEN character: X');
    });
});
