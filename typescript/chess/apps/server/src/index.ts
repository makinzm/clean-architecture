import { Hono } from 'hono';
import { serve } from '@hono/node-server';
import { buildDependencies } from './AppState.js';
import { errorHandler } from './middleware/errorHandler.js';

const app = new Hono();
const deps = buildDependencies();

app.use('*', errorHandler);

app.post('/api/games', async (c) => {
    const game = await deps.createGameUseCase.execute();
    return c.json({ game }, 201);
});

app.get('/api/games', async (c) => {
    const games = await deps.listGamesUseCase.execute();
    return c.json({ games }, 200);
});

app.get('/api/games/:id', async (c) => {
    const id = c.req.param('id');
    const game = await deps.getGameStateUseCase.execute(id);
    const fen = `${game.board.toFenPosition()} ${game.turn} - - 0 1`;
    return c.json({
        game: { id, turn: game.turn, fen }
    }, 200);
});

app.post('/api/games/:id/join', async (c) => {
    const id = c.req.param('id');
    // Usually from auth token or body
    const playerId = 'player-1';
    const result = await deps.joinGameUseCase.execute(id, playerId);
    return c.json(result, 200);
});

app.post('/api/games/:id/moves', async (c) => {
    const id = c.req.param('id');
    const body = await c.req.json();
    // Again, playerId from auth
    const playerId = 'player-1';

    await deps.makeMoveUseCase.execute(id, playerId, body.from, body.to);

    // Return updated game state
    const game = await deps.getGameStateUseCase.execute(id);
    const fen = `${game.board.toFenPosition()} ${game.turn} - - 0 1`;
    return c.json({
        success: true,
        game: { id, turn: game.turn, fen }
    }, 200);
});

// WS route placeholder
app.get('/ws/games/:id', (c) => {
    // In a real app we'd upgrade to WebSocket and register with deps.wsManager.addClient
    return c.text('WebSocket Endpoint Stub');
});

export { app };

if (process.env.NODE_ENV !== 'test') {
    serve({ fetch: app.fetch, port: 3000 }, (info) => {
        console.log(`Listening on http://localhost:${info.port}`);
    });
}
