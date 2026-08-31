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
import { getDateFormatted } from '../utils';

test('wizzair', async ({ page }) => {
    const tomorrowDate = getDateFormatted(1);

    const apiResponsePromise = page.waitForResponse((response) => response.url().endsWith('/Api/search/search'));
    await page.goto(`https://wizzair.com/en-gb/booking/select-flight/TIA/BRI/${tomorrowDate}/null/1/0/0/null`);

    const apiResponse = await apiResponsePromise;

    // If it's 429 it means it was blocked by Kasada
    expect(apiResponse.status()).toBe(200);
});
