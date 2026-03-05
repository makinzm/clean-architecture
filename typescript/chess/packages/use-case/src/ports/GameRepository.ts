import { Game } from '@chess/domain';

export interface GameRepository {
    save(game: Game): Promise<void>;
    findById(id: string): Promise<Game | null>;
    findAll(): Promise<Game[]>;
}
