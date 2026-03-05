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

let currentGameId: string | null = localStorage.getItem('currentGameId');
let selectedSquare: { file: number, rank: number } | null = null;
let currentFen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

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

    // Set the base class and optionally the selected class
    if (selectedSquare?.file === file && selectedSquare?.rank === rank) {
        square.className = `square ${isLight ? 'light' : 'dark'} selected`;
    } else {
        square.className = `square ${isLight ? 'light' : 'dark'}`;
    }

    const pieceMap: Record<string, string> = {
        'P': '♙', 'N': '♘', 'B': '♗', 'R': '♖', 'Q': '♕', 'K': '♔',
        'p': '♟', 'n': '♞', 'b': '♝', 'r': '♜', 'q': '♛', 'k': '♚'
    };

    square.textContent = pieceMap[piece] || '';

    // No need for individual event listener, using delegation on boardEl

    boardEl.appendChild(square);
}

// Event delegation for square clicks
boardEl.addEventListener('click', (e) => {
    const target = e.target as HTMLElement;
    const square = target.closest('.square');
    if (square) {
        const file = parseInt(square.getAttribute('data-file') || '0', 10);
        const rank = parseInt(square.getAttribute('data-rank') || '0', 10);
        handleSquareClick(file, rank);
    }
});

async function handleSquareClick(file: number, rank: number) {
    if (!currentGameId) {
        console.log('No current game ID, ignoring click', { file, rank });
        return;
    }

    console.log(`Square clicked: file=${file}, rank=${rank}`);

    if (!selectedSquare) {
        selectedSquare = { file, rank };
        const sq = boardEl.querySelector(`.square[data-file="${file}"][data-rank="${rank}"]`);

        console.log('Found square element: ', !!sq);

        // Remove old selection visually
        const selectedEls = boardEl.querySelectorAll('.selected');
        selectedEls.forEach(el => el.classList.remove('selected'));

        if (sq) {
            sq.classList.add('selected');
            console.log('Added selected class. Current classes:', sq.className);
        }
    } else {
        // Attempt move
        const fromAlgebraic = toAlgebraic(selectedSquare.file, selectedSquare.rank);
        const toAlgebraicStr = toAlgebraic(file, rank);

        selectedSquare = null; // deselect

        // Remove selection visually before network request
        const selectedEls = boardEl.querySelectorAll('.selected');
        selectedEls.forEach(el => el.classList.remove('selected'));

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

        if (game.fen) {
            renderBoard(game.fen);
        }

        console.log('Got game state', game);
    }
}

// Initial load check
if (currentGameId) {
    fetchGameState();
} else {
    renderBoard('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1');
}

createBtn.addEventListener('click', async () => {
    const res = await fetch('/api/games', { method: 'POST' });
    if (res.ok) {
        const createData = await res.json();

        // We get ID from POST directly
        currentGameId = createData.game.id;
        localStorage.setItem('currentGameId', currentGameId!);

        statusEl.textContent = 'Game active. Turn: w';
        boardEl.dataset.gameReady = 'true'; // Add marker for E2E tests

        if (createData.game.fen) {
            renderBoard(createData.game.fen);
        } else {
            renderBoard('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1');
        }
    }
});
