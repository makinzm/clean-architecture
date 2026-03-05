import { NotificationPort } from '@chess/use-case';
import { Game } from '@chess/domain';

export class LocalWebSocketManager implements NotificationPort {
    // Map of gameId -> Set of WebSockets
    private clients: Map<string, Set<any>> = new Map();

    addClient(gameId: string, ws: any) {
        if (!this.clients.has(gameId)) {
            this.clients.set(gameId, new Set());
        }
        this.clients.get(gameId)!.add(ws);
    }

    removeClient(gameId: string, ws: any) {
        this.clients.get(gameId)?.delete(ws);
    }

    async notifyGameUpdated(gameId: string, game: Game): Promise<void> {
        const gameClients = this.clients.get(gameId);
        if (!gameClients) return;

        const message = JSON.stringify({
            type: 'GAME_UPDATED',
            payload: {
                turn: game.turn,
                isCheckmate: game.isCheckmate(),
                isStalemate: game.isStalemate(),
                fen: game.board.toFenPosition(),
            }
        });

        for (const client of gameClients) {
            if (client.readyState === undefined || client.readyState === 1) { // 1 = OPEN
                client.send(message);
            }
        }
    }
}
