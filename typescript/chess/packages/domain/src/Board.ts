import { Square } from './Square.js';
import { Piece, PieceColor, PieceType } from './Piece.js';

export class Board {
    // We'll store the grid as an array of 64 squares or use a map
    private squares: Map<string, Piece>;

    private constructor(squares: Map<string, Piece>) {
        this.squares = new Map(squares);
    }

    static createInitialBoard(): Board {
        const squares = new Map<string, Piece>();

        const setupRank = (rank: number, color: PieceColor, isPawns: boolean) => {
            const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
            const majorPieces = [
                PieceType.ROOK, PieceType.KNIGHT, PieceType.BISHOP, PieceType.QUEEN,
                PieceType.KING, PieceType.BISHOP, PieceType.KNIGHT, PieceType.ROOK
            ];

            for (let i = 0; i < files.length; i++) {
                const file = files[i];
                const type = isPawns ? PieceType.PAWN : majorPieces[i];
                squares.set(`${file}${rank}`, new Piece(color, type));
            }
        };

        setupRank(1, PieceColor.WHITE, false);
        setupRank(2, PieceColor.WHITE, true);
        setupRank(7, PieceColor.BLACK, true);
        setupRank(8, PieceColor.BLACK, false);

        return new Board(squares);
    }

    getPieceAt(square: Square): Piece | undefined {
        return this.squares.get(square.toString());
    }

    applyMove(from: Square, to: Square): Board {
        const piece = this.getPieceAt(from);
        if (!piece) {
            throw new Error(`No piece at ${from.toString()}`);
        }

        const newSquares = new Map(this.squares);
        newSquares.delete(from.toString());
        newSquares.set(to.toString(), piece);

        return new Board(newSquares);
    }

    toFenPosition(): string {
        let fen = '';
        const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

        for (let rank = 8; rank >= 1; rank--) {
            let emptyCount = 0;
            for (let i = 0; i < files.length; i++) {
                const file = files[i];
                const piece = this.getPieceAt(Square.fromString(`${file}${rank}`));
                if (piece) {
                    if (emptyCount > 0) {
                        fen += emptyCount;
                        emptyCount = 0;
                    }
                    fen += piece.toFenChar();
                } else {
                    emptyCount++;
                }
            }
            if (emptyCount > 0) {
                fen += emptyCount;
            }
            if (rank > 1) {
                fen += '/';
            }
        }
        return fen;
    }
}
