import { PlayerRepository, Player } from '@chess/use-case';

export class InMemoryPlayerRepository implements PlayerRepository {
    private players: Map<string, Player> = new Map();

    async save(player: Player): Promise<void> {
        this.players.set(player.id, player);
    }

    async findById(id: string): Promise<Player | null> {
        return this.players.get(id) || null;
    }
}
