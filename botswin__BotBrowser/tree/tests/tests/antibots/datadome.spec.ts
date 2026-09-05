/**
 * E-COMMERCE, TICKETING & PAYMENT SERVICES TESTING - AUTHORIZED USE ONLY
 *
 * This test demonstrates DataDome tracking in AUTHORIZED TEST ENVIRONMENTS ONLY.
 *
 * Test Methodology:
 * - Uses publicly accessible pages (search, browse, demo pages)
 * - Submits ONLY synthetic/invalid test data
 * - Does NOT attempt real purchases or payment processing
 * - For academic fingerprint consistency research
 * DEMO USE ONLY: For demonstration and learning purposes. Never use to violate any website Terms of Service.
 *
 * PROHIBITED: Production testing, real transactions, or unauthorized account access.
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

import { expect, test } from '../global-setup';
import { generateRandomEmail, generateRandomPassword, sleep } from '../utils';

test('shutterstock', async ({ page }) => {
    await page.goto('https://www.shutterstock.com/search');

    const apiResponsePromise = page.waitForResponse((response) =>
        response.url().endsWith('/search/nature.json?term=nature')
    );

    await page.locator('input#search-bar').fill('nature');
    await page.locator('button[aria-label="Search"]').click();

    const apiResponse = await apiResponsePromise;
    expect(apiResponse.status()).toBe(200);
    const apiResponseJson = await apiResponse.json();
    expect(
        apiResponseJson.botProps.isGoodBot === false && apiResponseJson.botProps.isSuspectedBadBot === false
    ).toBeTruthy();

    await sleep(5000);
});

test('seatgeek', async ({ page }) => {
    await page.goto(
        'https://seatgeek.com/lady-gaga-tickets/new-york-new-york-madison-square-garden-2025-09-06-8-pm/concert/17460509'
    );
    await expect(page.locator('span[role="heading"]').textContent()).resolves.toMatch(/How many tickets/);
});

test('hermes', async ({ page }) => {
    await page.goto('https://www.hermes.com/us/en/');

    await page.locator('button#onetrust-accept-btn-handler').click();
    await sleep(5000);
    await page.locator('span >> text=Hats and gloves').click();
    await sleep(5000);
    await page.locator('h-image-resizer').nth(0).click();
    await sleep(5000);
    await page.locator('button >> text=Add to cart').click();
    await sleep(5000);

    // id="variant-item-H242034N_0256-size"
    const variantItems = await page.locator('input[id^="variant-item-"]').all();
    if (variantItems.length > 0) {
        await variantItems[0].click();
        await sleep(5000);
        await page.locator('button >> text=Add to cart').click();
        await sleep(5000);
    }

    await page.locator('button >> text=View cart').click();

    await sleep(5_000);
});

test('soundcloud', async ({ page }) => {
    await page.goto('https://soundcloud.com/');
    await page.locator('div.frontHero__signin > button[aria-label="Create a SoundCloud account"]').click();
    await sleep(3_000);
    const authFrameLocator = page.frameLocator('iframe[src*="secure.soundcloud.com/web-auth"]');
    await authFrameLocator.locator('input[name="email"]').pressSequentially(generateRandomEmail(), {
        delay: 20,
    });
    await page.keyboard.press('Enter');
    await sleep(3_000);
    await authFrameLocator.locator('input[name="password"]').pressSequentially(generateRandomPassword(), {
        delay: 20,
    });
    await page.keyboard.press('Enter');
    await sleep(3_000);
    await authFrameLocator.locator('input[name="age"]').pressSequentially('21', {
        delay: 20,
    });
    await authFrameLocator.locator('select[name="gender"]').selectOption({
        value: 'female',
    });
    await sleep(3_000);
    await page.keyboard.press('Enter');
    await authFrameLocator.locator('div.email-verification-page').waitFor({ state: 'visible' });
});

test('paypal', async ({ page }) => {
    await page.goto('https://www.paypal.com/unifiedonboarding/entry?country.x=US&locale.x=en_US');
    await page.locator('h2 >> text=Create a business account').waitFor({ state: 'visible' });
});

test('allegro', async ({ page }) => {
    await page.goto('https://allegro.pl/');
    await sleep(1500);
    for (let i = 0; i < 10; i++) {
        await page.mouse.wheel(0, 200);
        await sleep(800);
    }

    const pageResponsePromise = page.waitForResponse((response) =>
        response
            .url()
            .includes(
                'https://allegro.pl/oferta/filtr-do-odkurzacza-miele-boost-cx1-allergy-cat-dog-11639250-17115428549'
            )
    );
    await page
        .goto('https://allegro.pl/oferta/filtr-do-odkurzacza-miele-boost-cx1-allergy-cat-dog-11639250-17115428549')
        .catch(() => {
            // no-op
        });

    const pageResponse = await pageResponsePromise;
    const pageResponseText = await pageResponse.text();
    expect(/"price":\s*"[\d,]+/.test(pageResponseText)).toBeTruthy();
});

test('fifa', async ({ page }) => {
    await page.goto('https://fifa-fwc26-us.tickets.fifa.com/');
    await page.locator('#titre >> text=Almost there').waitFor({ state: 'visible' });
});
