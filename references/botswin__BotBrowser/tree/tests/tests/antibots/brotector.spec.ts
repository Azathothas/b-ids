/**
 * PRIVACY RESEARCH USE ONLY
 *
 * Run exclusively inside authorized privacy research labs with synthetic data only.
 * Never target production traffic or violate any Terms of Service.
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */


import { expect, test } from '../global-setup';
import { sleep } from '../utils';

test('Brotector', async ({ page }) => {
    await page.goto('https://kaliiiiiiiiii.github.io/brotector/?crash=false');

    // Simulate natural mouse movement on the page
    await page.mouse.move(0, 0);

    for (let i = 0; i < 10; i++) {
        await sleep(Math.random() * 2000 + 500);
        await page.mouse.move(Math.random() * 1000, Math.random() * 1000, {
            steps: Math.floor(Math.random() * 100) + 1,
        });
    }

    const tableRowsCount = await page.locator('table').locator('tr').count();
    expect(tableRowsCount).toBe(2);
});
