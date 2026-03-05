import { test, expect } from '@playwright/test';

test('create game and make a basic move', async ({ page }) => {
    // Go to client URL
    await page.goto('/');

    // Should have title
    await expect(page).toHaveTitle(/Clean Architecture Chess/i);

    // Click on "New Game" button
    await page.getByRole('button', { name: 'New Game' }).click();

    // Verify game status appears
    const status = page.locator('#game-status');
    await expect(status).toContainText('Game active');

    // Verify squares are rendered (64 squares)
    const squares = page.locator('.square');
    await expect(squares).toHaveCount(64);

    // Find a pawn, let's say pawn at e2 (file 4, rank 1 in 0-indexed terms)
    // According to our main.ts logic: "e2" -> file=4, rank=1 
    // Wait, in standard chess e2 is file 'e' (4th index: a0,b1,c2,d3,e4), rank 2 (index 1)
    const e2 = page.locator('.square[data-file="4"][data-rank="1"]');
    await e2.click();

    // Verify it gets highlighted (selected class added)
    // In `main.ts` we added `selected` class instead of highlight
    await expect(e2).toHaveClass(/selected/);

    // Move it to e4 (file 4, rank 3)
    const e4 = page.locator('.square[data-file="4"][data-rank="3"]');
    await e4.click();

    // The pawn should now be at e4. We need to verify that.
    // We'll just verify the game state updates properly and e4 has the piece '♙'
    await expect(e4).toHaveText('♙');
});
