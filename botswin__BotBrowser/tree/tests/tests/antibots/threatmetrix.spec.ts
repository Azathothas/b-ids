/**
 * FINANCIAL SERVICES TESTING - AUTHORIZED USE ONLY
 *
 * This test demonstrates ThreatMetrix tracking in AUTHORIZED TEST ENVIRONMENTS ONLY.
 *
 * Test Methodology:
 * - Uses publicly accessible "forgot password" pages (NOT login/account access)
 * - Submits ONLY invalid/synthetic data (fake names, invalid SSN "6789")
 * - Does NOT attempt authentication or account access
 * - For academic fingerprint consistency research
 *
 * PROHIBITED: Production testing without explicit written authorization.
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

import { expect, test } from '../global-setup';
import { generateRandomUsername } from '../utils';

test('vanguard', async ({ page }) => {
    await page.goto('https://personal1.vanguard.com/lmi-forgotusernamepassword/home');
    await page.locator('#firstName').pressSequentially(generateRandomUsername(), { delay: 40 });
    await page.locator('#lastName').pressSequentially(generateRandomUsername(), { delay: 40 });
    await page.locator('#ssn').pressSequentially('6789', { delay: 40 });
    await page.locator('#dob').pressSequentially('11111960', { delay: 40 });
    await page.locator('#us').click({ force: true });
    await page.locator('#zipCode').pressSequentially('75052', { delay: 40 });

    const responsePromise = page.waitForResponse(
        (response) =>
            response.url().endsWith('/lmi-forgotusernamepassword/api/find-client') && response.status() === 200
    );

    await page.locator('button[type="submit"]').click();
    expect((await (await responsePromise).json()).type).toBe('NotFound');
});

test('fidelity', async ({ page }) => {
    await page.goto(
        'https://nb.fidelity.com/public/nbpreloginnav/app/forgotlogindomestic#/forgotLoginDomestic/verifyIdentity'
    );
    await page.locator('#onetrust-accept-btn-handler').click({ force: true });
    await page.locator('#personFirstName').pressSequentially(generateRandomUsername(), { delay: 40 });
    await page.locator('#personLastName').pressSequentially(generateRandomUsername(), { delay: 40 });
    await page.locator('#dob-month-input').selectOption('01');
    await page.locator('#dob-day-input').pressSequentially('10', { delay: 40 });
    await page.locator('#dob-year-input').pressSequentially('1960', { delay: 40 });
    await page.locator('#lastFourOfSSN').pressSequentially('6789', { delay: 40 });

    const responsePromise = page.waitForResponse(
        (response) => response.url().endsWith('/user/identity/attributes/.search') && response.status() === 200
    );

    await page.locator('#nb-prelogin-submit-button').click();
    expect((await (await responsePromise).json()).responseBaseInfo.status.code).toBe(1404);
});
