import { GameRepository } from '@chess/use-case';
import { Game } from '@chess/domain';

export class InMemoryGameRepository implements GameRepository {
    private games: Map<string, Game> = new Map();

    async save(idOrGame: string | Game, maybeGame?: Game): Promise<void> {
        if (typeof idOrGame === 'string' && maybeGame) {
            this.games.set(idOrGame, maybeGame);
        } else {
            // In actual clean arch, the entity Game would likely hold its own ID.
            // But since we didn't add it in Phase 1, we randomly assign one here.
            const id = Date.now().toString();
            this.games.set(id, idOrGame as Game);
        }
    }

    async findById(id: string): Promise<Game | null> {
        return this.games.get(id) || null;
    }

    async findAll(): Promise<Game[]> {
        return Array.from(this.games.values());
    }
}
