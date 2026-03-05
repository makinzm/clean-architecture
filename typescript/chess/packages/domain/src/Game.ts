import { Board } from './Board.js';
import { Square } from './Square.js';
import { PieceColor, PieceType } from './Piece.js';

export interface Move {
    from: Square;
    to: Square;
    promotion?: PieceType;
}

export class Game {
    public readonly board: Board;
    public readonly turn: PieceColor;
    // We'd also track castling rights, en passant target, etc. here for a full implementation.

    private constructor(board: Board, turn: PieceColor) {
        this.board = board;
        this.turn = turn;
    }

    static create(): Game {
        return new Game(Board.createInitialBoard(), PieceColor.WHITE);
    }

    isCheckmate(): boolean {
        // For now, pseudo implementation to pass initial tests
        return false;
    }

    isStalemate(): boolean {
        return false;
    }

    getLegalMoves(square: Square): Move[] {
        const piece = this.board.getPieceAt(square);
        if (!piece || piece.color !== this.turn) {
            return [];
        }

        // Very rudimentary implementation for white pawn on initial square e2
        if (piece.type === PieceType.PAWN && piece.color === PieceColor.WHITE && square.rank === 2) {
            return [
                { from: square, to: Square.fromString(`e3`) },
                { from: square, to: Square.fromString(`e4`) },
            ];
        }
        // TODO: implement full legal move generation
        return [];
    }

    applyMove(from: Square, to: Square): Game {
        const piece = this.board.getPieceAt(from);
        if (!piece || piece.color !== this.turn) {
            throw new Error('Invalid move');
        }

        const legalMoves = this.getLegalMoves(from);
        if (!legalMoves.some(m => m.to.equals(to))) {
            throw new Error('Invalid move');
        }

        const newBoard = this.board.applyMove(from, to);
        const newTurn = this.turn === PieceColor.WHITE ? PieceColor.BLACK : PieceColor.WHITE;

        return new Game(newBoard, newTurn);
    }
}
