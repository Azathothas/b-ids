/**
 * PRIVACY RESEARCH USE ONLY
 *
 * Run exclusively inside authorized privacy research labs with synthetic data only.
 * Never target production traffic or violate any Terms of Service.
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */


import { test } from '../global-setup';

test('invisiblecaptcha', async ({ page }) => {
    await page.goto('https://www.mtcaptcha.com/');
    await page.locator('#demo-invisible-checkmark1').scrollIntoViewIfNeeded();
    await page.locator('#demo-invisible-checkmark1 + .demo-checkmark').click({ force: true });
    await page.locator('#msg-invisible-verified >> text=verified successfully').waitFor({ state: 'visible' });
});
