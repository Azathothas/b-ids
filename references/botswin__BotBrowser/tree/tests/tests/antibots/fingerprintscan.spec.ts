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

test('fingerprintscan', async ({ page }) => {
    await page.goto('https://fingerprint-scan.com');
    await page.locator('div#fingerprintScore >> text=Bot Risk Score: 20/100').waitFor({ state: 'visible' });
});
