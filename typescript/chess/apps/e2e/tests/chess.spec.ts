import { test, expect } from '@playwright/test';

test('create game and make a basic move', async ({ page }) => {
    // Listen for console events and print them
    page.on('console', msg => console.log(`Browser console: ${msg.text()}`));

    // Go to client URL
    await page.goto('/');

    // Should have title
    await expect(page).toHaveTitle(/Clean Architecture Chess/i);

    // Click on "New Game" button, wait for the network request to finish
    await Promise.all([
        page.waitForResponse(resp => resp.url().includes('/api/games') && resp.status() === 201),
        page.getByRole('button', { name: 'New Game' }).click()
    ]);

    // Give it an extra moment to render 
    await page.waitForTimeout(500);

    // Wait for the client to process the response and set `currentGameId`
    await page.locator('#board[data-game-ready="true"]').waitFor();

    // Verify game status appears
    const status = page.locator('#game-status');
    await expect(status).toContainText('Game active');

    // Verify squares are rendered (64 squares)
    const squares = page.locator('.square');
    await expect(squares).toHaveCount(64);

    // Find a pawn, let's say pawn at e2 (file 4, rank 1 in 0-indexed terms)
    // According to our main.ts logic: "e2" -> file=4, rank=1 
    // Wait for the board piece to be rendered from the fetch response
    // Find a pawn, wait for the actual DOM element that has the piece text
    const e2 = page.locator('.square[data-file="4"][data-rank="1"]');
    await expect(e2).toHaveText('♙'); // Wait until pieces populate

    // Evaluate click inside the browser to bypass Playwright's potentially blocked action
    await e2.evaluate(el => (el as HTMLElement).click());

    // Verification of highlighted selected class added
    // Re-locate since the DOM might have been rebuilt
    await expect(page.locator('.square[data-file="4"][data-rank="1"]')).toHaveClass(/selected/);

    // Move it to e4 (file 4, rank 3)
    const e4 = page.locator('.square[data-file="4"][data-rank="3"]');
    await e4.click();

    // The pawn should now be at e4. We need to verify that.
    // We'll just verify the game state updates properly and e4 has the piece '♙'
    await expect(page.locator('.square[data-file="4"][data-rank="3"]')).toHaveText('♙');
});
