/**
 * PRIVACY RESEARCH USE ONLY
 *
 * Run exclusively inside authorized privacy research labs with synthetic data only.
 * Never target production traffic or violate any Terms of Service.
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */


import { createCursor } from 'ghost-cursor-playwright';
import { expect, test } from '../global-setup';
import { clickWithCursor, enableMouseMovementOverlay, generateRandomEmail, sleep } from '../utils';

test('zillow', async ({ page }) => {
    await page.goto('https://www.zillow.com/');
    await sleep(5_000);
    await page.locator('#register-tab').click();
    await page.locator('input#inputs-newEmail').pressSequentially(generateRandomEmail(), { delay: 20 });
    await sleep(2_000);
    await page.locator('input#password').pressSequentially('Fdf24+dvAc?_13', { delay: 20 });
    await sleep(2_000);
    await page.keyboard.press('Enter');
    await sleep(2_000);
    await page.locator('[[data-za-label="My Zillow"]]').waitFor({ state: 'visible' });
});

test('budget', async ({ page }) => {
    test.setTimeout(3000_000);
    const cursor = await createCursor(page);
    await enableMouseMovementOverlay(page);
    await page.goto('https://www.budget.co.nz/en/home');
    await clickWithCursor(cursor, '#PicLoc');
    await sleep(2000);
    await page.locator('input#PicLoc_value').first().pressSequentially('141', { delay: 200 });
    await clickWithCursor(cursor, 'div#PicLoc_dropdown >> div.angucomplete-results >> text=Banja');
    await sleep(2000);
    await clickWithCursor(cursor, 'button#res-home-select-car');
    expect(await page.waitForNavigation({ url: 'https://www.budget.co.nz/en/reservation#/vehicles' })).toBeTruthy();
});
