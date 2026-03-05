export enum PieceColor {
    WHITE = 'w',
    BLACK = 'b',
}

export enum PieceType {
    PAWN = 'p',
    KNIGHT = 'n',
    BISHOP = 'b',
    ROOK = 'r',
    QUEEN = 'q',
    KING = 'k',
}

export class Piece {
    constructor(public readonly color: PieceColor, public readonly type: PieceType) { }

    static fromFenChar(char: string): Piece {
        const isWhite = char === char.toUpperCase();
        const typeStr = char.toLowerCase();

        const color = isWhite ? PieceColor.WHITE : PieceColor.BLACK;
        let type: PieceType;

        switch (typeStr) {
            case 'p': type = PieceType.PAWN; break;
            case 'n': type = PieceType.KNIGHT; break;
            case 'b': type = PieceType.BISHOP; break;
            case 'r': type = PieceType.ROOK; break;
            case 'q': type = PieceType.QUEEN; break;
            case 'k': type = PieceType.KING; break;
            default:
                throw new Error(`Invalid FEN character: ${char}`);
        }

        return new Piece(color, type);
    }

    toFenChar(): string {
        const char = this.type.toString();
        return this.color === PieceColor.WHITE ? char.toUpperCase() : char;
    }

    equals(other: Piece): boolean {
        return this.color === other.color && this.type === other.type;
    }
}
