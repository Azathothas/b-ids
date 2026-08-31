/**
 * E-COMMERCE TESTING - AUTHORIZED USE ONLY
 *
 * This test demonstrates e-commerce compatibility in AUTHORIZED TEST ENVIRONMENTS ONLY.
 *
 * Test Methodology:
 * - Tests publicly accessible pages (browse, search functionality)
 * - Uses ONLY synthetic/test data
 * - Does NOT attempt real purchases or account manipulation
 * - For academic compatibility research
 *
 * PROHIBITED: Real purchases, unauthorized automation, or ToS violations.
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

import { expect, test } from '../global-setup';
import { enableMouseMovementOverlay } from '../utils';

test('shopee', async ({ page }) => {
    await enableMouseMovementOverlay(page);
    await page.goto('https://shopee.co.id/Handphone-Aksesoris-cat.11044458', {
        waitUntil: 'domcontentloaded',
    });

    expect(
        await page.waitForResponse((res) => res.url().startsWith('https://shopee.co.id/api/v4/recommend/recommend_v2'))
    ).toBeTruthy();
});
