export type File = 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h';
export type Rank = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;

export class Square {
    constructor(public readonly file: File, public readonly rank: Rank) { }

    static fromString(algebraic: string): Square {
        if (algebraic.length !== 2) {
            throw new Error(`Invalid square: ${algebraic}`);
        }

        const fileChar = algebraic[0];
        const rankChar = algebraic[1];

        if (!/^[a-h]$/.test(fileChar) || !/^[1-8]$/.test(rankChar)) {
            throw new Error(`Invalid square: ${algebraic}`);
        }

        return new Square(fileChar as File, parseInt(rankChar, 10) as Rank);
    }

    toString(): string {
        return `${this.file}${this.rank}`;
    }

    equals(other: Square): boolean {
        return this.file === other.file && this.rank === other.rank;
    }
}
