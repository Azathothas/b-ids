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

test('browserscan', async ({ page }) => {
    await page.goto('https://www.browserscan.net/');
    await sleep(20_000);

    expect(await page.locator('text=Browser fingerprint authenticity: 95%')).toBeTruthy();
});
