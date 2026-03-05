import './style.css';


document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <div>
    <h1>Clean Architecture Chess</h1>
    <div id="controls">
      <button id="btn-create">New Game</button>
      <div id="game-status"></div>
    </div>
    <div id="board" class="chess-board"></div>
  </div>
`;

const boardEl = document.getElementById('board')!;
const createBtn = document.getElementById('btn-create')!;
const statusEl = document.getElementById('game-status')!;

let currentGameId: string | null = null;
let selectedSquare: { file: number, rank: number } | null = null;
let currentFen = '8/8/8/8/8/8/8/8 w - - 0 1';

function renderBoard(fen: string) {
    currentFen = fen;
    boardEl.innerHTML = '';
    const rows = fen.split(' ')[0].split('/');

    for (let rank = 0; rank < 8; rank++) {
        let file = 0;
        for (const char of rows[rank]) {
            if (isNaN(parseInt(char))) {
                createSquare(file, 7 - rank, char);
                file++;
            } else {
                const spaces = parseInt(char);
                for (let i = 0; i < spaces; i++) {
                    createSquare(file, 7 - rank, '');
                    file++;
                }
            }
        }
    }
}

function createSquare(file: number, rank: number, piece: string) {
    const isLight = (file + rank) % 2 !== 0;
    const square = document.createElement('div');
    square.className = 'square ' + (isLight ? 'light' : 'dark');
    square.dataset.file = file.toString();
    square.dataset.rank = rank.toString();

    if (selectedSquare?.file === file && selectedSquare?.rank === rank) {
        square.classList.add('selected');
    }

    const pieceMap: Record<string, string> = {
        'P': '♙', 'N': '♘', 'B': '♗', 'R': '♖', 'Q': '♕', 'K': '♔',
        'p': '♟', 'n': '♞', 'b': '♝', 'r': '♜', 'q': '♛', 'k': '♚'
    };

    square.textContent = pieceMap[piece] || '';

    square.addEventListener('click', () => handleSquareClick(file, rank));
    boardEl.appendChild(square);
}

async function handleSquareClick(file: number, rank: number) {
    if (!currentGameId) return;

    if (!selectedSquare) {
        selectedSquare = { file, rank };
        renderBoard(currentFen);
    } else {
        // Attempt move
        const fromAlgebraic = toAlgebraic(selectedSquare.file, selectedSquare.rank);
        const toAlgebraicStr = toAlgebraic(file, rank);

        selectedSquare = null; // deselect

        if (fromAlgebraic !== toAlgebraicStr) {
            try {
                const res = await fetch('/api/games/' + currentGameId + '/moves', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ from: fromAlgebraic, to: toAlgebraicStr })
                });
                if (!res.ok) {
                    const err = await res.json();
                    alert('Move failed: ' + err.message);
                } else {
                    // fetch updated game state
                    await fetchGameState();
                }
            } catch (e) {
                alert('Error communicating with server');
            }
        } else {
            renderBoard(currentFen);
        }
    }
}

function toAlgebraic(file: number, rank: number): string {
    const fileChar = String.fromCharCode('a'.charCodeAt(0) + file);
    return fileChar + (rank + 1).toString();
}

async function fetchGameState() {
    if (!currentGameId) return;
    const res = await fetch('/api/games/' + currentGameId);
    if (res.ok) {
        const data = await res.json();
        const game = data.game;
        statusEl.textContent = 'Game active. Turn: ' + game.turn;
        // For now we get full board array or something since the domain entity is complex
        // If backend serialized properly, it has fen or board. Let's just mock FEN update 
        // depending on what the backend gives. Since the real implementation of MakeMoveUseCase
        // just returns success, we rely on GetGameStateUseCase which returns the Game.
        // However, Game hasn't implemented FEN fully, but we added a basic toFenPosition in Phase 1.
        if (game && game.board && typeof game.board.toFenPosition === 'function') {
            // Since we don't have Game.restore(), let's hope the backend sends state in an easy format
            // Since we don't have Game.restore(), let's hope the backend sends state in an easy format
        }
        // Stub
        console.log('Got game state', game);
    }
}

renderBoard('8/8/8/8/8/8/8/8 w - - 0 1');

createBtn.addEventListener('click', async () => {
    const res = await fetch('/api/games', { method: 'POST' });
    if (res.ok) {
        // We assume ID is available or we fetch list
        const listRes = await fetch('/api/games');
        const listData = await listRes.json();
        if (listData.games && listData.games.length > 0) {
            // Just take latest or stub
            currentGameId = 'test-id'; // Using a placeholder as our repo adapter hasn't fully integrated IDs
            statusEl.textContent = 'Game active. Turn: w';
            renderBoard('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1');
        }
    }
});
