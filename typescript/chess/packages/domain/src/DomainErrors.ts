export class DomainError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'DomainError';
    }
}

export class InvalidMoveError extends DomainError {
    constructor(message: string = 'Invalid move') {
        super(message);
        this.name = 'InvalidMoveError';
    }
}

export class GameAlreadyOverError extends DomainError {
    constructor(message: string = 'Game is already over') {
        super(message);
        this.name = 'GameAlreadyOverError';
    }
}

export class NotYourTurnError extends DomainError {
    constructor(message: string = 'Not your turn') {
        super(message);
        this.name = 'NotYourTurnError';
    }
}
