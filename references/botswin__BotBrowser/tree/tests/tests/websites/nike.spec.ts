/**
 * E-COMMERCE CHECKOUT TESTING - AUTHORIZED USE ONLY
 *
 * This test demonstrates e-commerce checkout flow in AUTHORIZED TEST ENVIRONMENTS ONLY.
 *
 * Test Methodology:
 * - Tests publicly accessible checkout pages (does NOT complete purchases)
 * - Uses ONLY synthetic/test data
 * - Does NOT process real payments or create real orders
 * - For academic compatibility research
 *
 * PROHIBITED: Real purchases, payment processing, or production system testing.
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

import { expect, test } from '../global-setup';
import { clickLocatorWithMouse, sleep } from '../utils';

test('checkout', async ({ page }) => {
    await page.goto(`https://www.nike.com/t/field-general-womens-shoes-SDTBm0k1/FZ5593-103`);

    const removeRipple = async () => {
        await page.evaluate(() => {
            document.querySelectorAll('span[class="ripple"]').forEach((el) => {
                el.remove();
            });
        });
    };

    await removeRipple();
    await clickLocatorWithMouse(page, 'input[id="grid-selector-input-11.5"]');
    await sleep(5000);

    // wheel to make sure add to bag button is visible
    await page.mouse.wheel(0, 150);
    await clickLocatorWithMouse(page, 'button[aria-label="Add to Bag"]');
    await sleep(5000);
    await clickLocatorWithMouse(page, 'button >> text=Checkout');
    await sleep(5000);

    await removeRipple();
    await clickLocatorWithMouse(page, 'button[data-attr="qa-guest-checkout"]');

    expect(await page.locator('input#search-address-input').first()).toBeTruthy();

    await sleep(10000);
});
