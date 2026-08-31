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

test('deviceandbrowserinfo', async ({ page }) => {
    await page.goto(`https://deviceandbrowserinfo.com/are_you_a_bot`);
    expect(page.locator('text=You are human').isVisible()).toBeTruthy();
});
