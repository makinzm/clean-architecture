export interface Player {
    id: string;
    name: string;
}

export interface PlayerRepository {
    save(player: Player): Promise<void>;
    findById(id: string): Promise<Player | null>;
}
