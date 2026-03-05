import { GameRepository } from '../ports/GameRepository.js';
import { NotificationPort } from '../ports/NotificationPort.js';
import { Square } from '@chess/domain';

export interface MakeMoveResult {
    success: boolean;
}

export class MakeMoveUseCase {
    constructor(
        private readonly gameRepository: GameRepository,
        private readonly notificationPort: NotificationPort
    ) { }

    async execute(gameId: string, playerId: string, fromAlgebraic: string, toAlgebraic: string): Promise<MakeMoveResult> {
        const game = await this.gameRepository.findById(gameId);
        if (!game) {
            throw new Error('Game not found');
        }

        const fromSquare = Square.fromString(fromAlgebraic);
        const toSquare = Square.fromString(toAlgebraic);

        const updatedGame = game.applyMove(fromSquare, toSquare);
        await this.gameRepository.save(updatedGame);
        await this.notificationPort.notifyGameUpdated(gameId, updatedGame);

        return { success: true };
    }
}
