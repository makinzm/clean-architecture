import { render, screen, fireEvent } from '@testing-library/svelte';
import App from './App.svelte';
import { vi } from 'vitest';

describe('App', () => {
    beforeEach(() => {
        // Reset fetch mock and local storage before each test
        vi.resetAllMocks();
        localStorage.clear();
    });

    it('shows New Game button when no game is active', () => {
        render(App);
        expect(screen.getByText('Ready to start a new match?')).toBeInTheDocument();
        expect(screen.getByRole('button', { name: 'Start New Game' })).toBeInTheDocument();
        expect(screen.queryByRole('button', { name: 'Abort / Resign' })).not.toBeInTheDocument();
    });

    it('shows Game UI when a game ID exists in localStorage', () => {
        localStorage.setItem('currentGameId', 'g-1234');
        render(App);
        expect(screen.queryByText('Ready to start a new match?')).not.toBeInTheDocument();
        expect(screen.getByRole('button', { name: 'Abort / Resign' })).toBeInTheDocument();
    });

    it('creates a new game successfully', async () => {
        const mockFetch = vi.fn().mockResolvedValue({
            ok: true,
            json: async () => ({
                game: {
                    id: 'g-123',
                    turn: 'w',
                    fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
                }
            })
        });
        global.fetch = mockFetch as any;

        render(App);

        const createBtn = screen.getByRole('button', { name: 'Start New Game' });
        await fireEvent.click(createBtn);

        expect(mockFetch).toHaveBeenCalledWith('/api/games', { method: 'POST' });
        // The UI should switch state on success
        expect(await screen.findByRole('button', { name: 'Abort / Resign' })).toBeInTheDocument();
    });
});
