<script lang="ts">
  import { onMount } from 'svelte';

  type AppState = 'MENU' | 'PLAYING';
  let appState: AppState = $state(localStorage.getItem('currentGameId') ? 'PLAYING' : 'MENU');
  
  let currentGameId: string | null = $state(localStorage.getItem('currentGameId'));
  let currentFen: string = $state('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1');
  let turn: string = $state('w');
  
  let selectedFile: number | null = $state(null);
  let selectedRank: number | null = $state(null);
  let errorMessage: string = $state('');
  let isLoading: boolean = $state(false);

  async function fetchGameState() {
    if (!currentGameId) return;
    try {
      const res = await fetch('/api/games/' + currentGameId);
      if (res.ok) {
        const data = await res.json();
        const game = data.game;
        turn = game.turn;
        if (game.fen) {
          currentFen = game.fen;
        }
      }
    } catch(e) {
      console.error(e);
    }
  }

  onMount(() => {
    if (currentGameId) {
      fetchGameState();
    }
  });

  async function createGame() {
    errorMessage = '';
    isLoading = true;
    try {
        const res = await fetch('/api/games', { method: 'POST' });
        if (res.ok) {
            const createData = await res.json();
            currentGameId = createData.game.id;
            localStorage.setItem('currentGameId', currentGameId!);
            turn = createData.game.turn;
            if (createData.game.fen) {
                currentFen = createData.game.fen;
            } else {
                currentFen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';
            }
            appState = 'PLAYING';
        } else {
            errorMessage = 'Failed to create game';
        }
    } catch(e) {
        errorMessage = 'Network error while creating game';
    } finally {
        isLoading = false;
    }
  }

  function abortGame() {
    if (confirm("Are you sure you want to abort/resign?")) {
        currentGameId = null;
        localStorage.removeItem('currentGameId');
        currentFen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';
        errorMessage = '';
        appState = 'MENU';
        selectedFile = null;
        selectedRank = null;
    }
  }

  function toAlgebraic(file: number, rank: number): string {
    const fileChar = String.fromCharCode('a'.charCodeAt(0) + file);
    return fileChar + (rank + 1).toString();
  }

  async function handleSquareClick(file: number, rank: number) {
    if (!currentGameId) return;

    if (selectedFile === null || selectedRank === null) {
        selectedFile = file;
        selectedRank = rank;
    } else {
        // Attempt move
        const fromAlg = toAlgebraic(selectedFile, selectedRank);
        const toAlg = toAlgebraic(file, rank);

        selectedFile = null;
        selectedRank = null;
        errorMessage = '';

        if (fromAlg !== toAlg) {
            try {
                const res = await fetch('/api/games/' + currentGameId + '/moves', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ from: fromAlg, to: toAlg })
                });
                if (!res.ok) {
                    const err = await res.json();
                    errorMessage = 'Move failed: ' + err.message;
                } else {
                    await fetchGameState();
                }
            } catch (e) {
                errorMessage = 'Error communicating with server';
            }
        }
    }
  }

  let boardRanks = $derived(() => {
        const rows = currentFen.split(' ')[0].split('/');
        const ranks = [];
        for (let r = 0; r < 8; r++) {
            const rankArr = [];
            let file = 0;
            for (const char of rows[r]) {
                if (isNaN(parseInt(char))) {
                    rankArr.push({ piece: char, file, rank: 7 - r });
                    file++;
                } else {
                    const spaces = parseInt(char);
                    for (let i = 0; i < spaces; i++) {
                        rankArr.push({ piece: '', file, rank: 7 - r });
                        file++;
                    }
                }
            }
            ranks.push(rankArr);
        }
        return ranks();
  });

  const getBoardRows = () => {
        const rows = currentFen.split(' ')[0].split('/');
        const ranks = [];
        for (let r = 0; r < 8; r++) {
            const rankArr = [];
            let file = 0;
            for (const char of rows[r]) {
                if (isNaN(parseInt(char))) {
                    rankArr.push({ piece: char, file, rank: 7 - r, id: `${file}-${7-r}` });
                    file++;
                } else {
                    const spaces = parseInt(char);
                    for (let i = 0; i < spaces; i++) {
                        rankArr.push({ piece: '', file, rank: 7 - r, id: `${file}-${7-r}` });
                        file++;
                    }
                }
            }
            ranks.push(rankArr);
        }
        return ranks;
  };

  function getPieceSymbol(pieceStr: string) {
    const isWhite = pieceStr === pieceStr.toUpperCase() && pieceStr !== '';
    const pieceMap: Record<string, string> = {
        'P': '♙', 'N': '♘', 'B': '♗', 'R': '♖', 'Q': '♕', 'K': '♔',
        'p': '♟', 'n': '♞', 'b': '♝', 'r': '♜', 'q': '♛', 'k': '♚'
    };
    const symbol = pieceMap[pieceStr] || '';
    if (!symbol) return '';
    // Use data attributes or inline styles to enforce high contrast
    return { symbol, isWhite };
  }
</script>

<!-- Modal Overlay for Game Menu -->
{#if appState === 'MENU'}
<div class="modal-overlay">
  <div class="modal">
    <h2>Welcome to Chess</h2>
    <p>Ready to start a new match?</p>
    {#if errorMessage}
        <div class="error">{errorMessage}</div>
    {/if}
    <button id="btn-create" onclick={createGame} disabled={isLoading}>
        {isLoading ? 'Creating Room...' : 'Start New Game'}
    </button>
  </div>
</div>
{/if}

<div class="app-container" class:blurred={appState === 'MENU'}>
  <h1>Clean Architecture Chess</h1>
  
  <div id="controls">
    {#if appState === 'PLAYING'}
        <button id="btn-abort" class="btn-danger" onclick={abortGame}>Abort / Resign</button>
        <div id="game-status">Game active. Turn: {turn === 'w' ? 'White' : 'Black'}</div>
    {/if}
  </div>

  {#if errorMessage}
    <div style="color: red; margin: 10px 0;">{errorMessage}</div>
  {/if}

  <!-- using data-game-ready to interop with E2E test -->
  <div id="board" class="chess-board" data-game-ready={currentGameId ? "true" : null}>
    <!-- Using reactive block instead of $derived -->
    {#each getBoardRows() as row}
      {#each row as cell (cell.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div 
          class="square {(cell.file + cell.rank) % 2 !== 0 ? 'light' : 'dark'} {selectedFile === cell.file && selectedRank === cell.rank ? 'selected' : ''}"
          data-file={cell.file}
          data-rank={cell.rank}
          onclick={() => handleSquareClick(cell.file, cell.rank)}
        >
          {#if cell.piece}
            {@const pieceInfo = getPieceSymbol(cell.piece)}
            <span class="piece {pieceInfo.isWhite ? 'piece-white' : 'piece-black'}">
                {pieceInfo.symbol}
            </span>
          {/if}
        </div>
      {/each}
    {/each}
  </div>
</div>
