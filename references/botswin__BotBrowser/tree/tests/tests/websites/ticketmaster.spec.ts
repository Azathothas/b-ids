/**
 * TICKETING SYSTEM TESTING - AUTHORIZED USE ONLY
 *
 * This test demonstrates ticketing system compatibility in AUTHORIZED TEST ENVIRONMENTS ONLY.
 *
 * Test Methodology:
 * - Tests publicly accessible event pages (does NOT purchase tickets)
 * - Uses ONLY browsing/viewing functionality
 * - Does NOT attempt real ticket purchases or reservations
 * - For academic compatibility research
 *
 * PROHIBITED: Real ticket purchases, scalping, or automated ticket acquisition.
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

import { expect, test } from '../global-setup';

test('checkout', async ({ page }) => {
    await page.goto('https://www.ticketmaster.com/');
    await page.goto(
        'https://www.ticketmaster.com/pinstripe-pass-new-york-yankees-v-bronx-new-york-06-06-2025/event/1D00611CCE275AF8'
    );
    await page.locator('button >> text=i agree').click();
    expect(await page.waitForSelector('h1 >> text=New York Yankees v. Boston Red Sox')).toBeTruthy();

    await page.locator('li[data-bdd="quick-picks-list-item-primary-1"]').click();
    await page.locator('button[data-bdd="offer-card-buy-button"] >> text=Next').click();
    await page.waitForSelector('div.text--primary >> text=Time Remaining');
});
