import { Game } from '@chess/domain';

export interface NotificationPort {
    notifyGameUpdated(gameId: string, game: Game): Promise<void>;
}
