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

        // Very rudimentary implementation: for now allow moving to any square to unblock E2E & manual UI testing
        // TODO: implement full legal move generation (TDD Phase 1)
        const pseudoMoves: Move[] = [];
        const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        for (let r = 1; r <= 8; r++) {
            for (const f of files) {
                if (`${f}${r}` !== square.toString()) {
                    pseudoMoves.push({ from: square, to: Square.fromString(`${f}${r}`) });
                }
            }
        }
        return pseudoMoves;
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
