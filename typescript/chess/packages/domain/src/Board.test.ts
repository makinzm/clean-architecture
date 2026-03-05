import { describe, it, expect } from 'vitest';
import { Board } from './Board.js';
import { Square } from './Square.js';
import { Piece, PieceColor, PieceType } from './Piece.js';

describe('Board Entity', () => {
    it('should create initial board with correct piece placement', () => {
        const board = Board.createInitialBoard();

        // Check white pawns
        expect(board.getPieceAt(Square.fromString('e2'))?.equals(new Piece(PieceColor.WHITE, PieceType.PAWN))).toBe(true);
        // Check black king
        expect(board.getPieceAt(Square.fromString('e8'))?.equals(new Piece(PieceColor.BLACK, PieceType.KING))).toBe(true);
        // Check empty square
        expect(board.getPieceAt(Square.fromString('e4'))).toBeUndefined();
    });

    it('should apply a move correctly', () => {
        const board = Board.createInitialBoard();
        const e2 = Square.fromString('e2');
        const e4 = Square.fromString('e4');

        // Move e2 pawn to e4
        const newBoard = board.applyMove(e2, e4);

        // e2 should be empty
        expect(newBoard.getPieceAt(e2)).toBeUndefined();
        // e4 should have white pawn
        expect(newBoard.getPieceAt(e4)?.equals(new Piece(PieceColor.WHITE, PieceType.PAWN))).toBe(true);

        // original board should be unmodified
        expect(board.getPieceAt(e2)?.equals(new Piece(PieceColor.WHITE, PieceType.PAWN))).toBe(true);
        expect(board.getPieceAt(e4)).toBeUndefined();
    });

    it('should handle FEN string generation for a given board', () => {
        const board = Board.createInitialBoard();
        // Initially, Board should map out to the starting FEN layout for pieces
        const position = board.toFenPosition();
        expect(position).toBe('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR');
    });
});
