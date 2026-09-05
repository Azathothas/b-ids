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

test('naver', async ({ page }) => {
    const productResponsePromise = page.waitForResponse((res) =>
        res.url().startsWith('https://brand.naver.com/cheiljedang/products/10678165717')
    );

    await page.goto('https://brand.naver.com/cheiljedang/products/10678165717', {
        waitUntil: 'domcontentloaded',
    });

    const productResponse = await productResponsePromise;
    const productData = await productResponse.text();

    // discountedSalePrice should be present in the response
    expect(productData).toContain('discountedSalePrice');
});
