/**
 * ANTI-BOT RESEARCH USE ONLY
 *
 * This test demonstrates Castle bot-risk evaluation in AUTHORIZED TEST ENVIRONMENTS ONLY.
 * - Exercises the public X.com sign-up flow without completing enrollment
 * - Submits only synthetic/invalid information (no real accounts or credentials)
 * - For academic fingerprint and behavioral signal analysis
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

import { test } from '../global-setup';
import { generateRandomEmail, generateRandomUsername } from '../utils';

test('x', async ({ page }) => {
    await page.goto('https://x.com/i/flow/signup');
    await page.locator('button >> text=Create account').click();

    const username = generateRandomUsername();
    await page.locator('input[name="name"]').pressSequentially(username, { delay: 20 });
    await page.locator('button >> text="Use email instead"').click();

    const email = generateRandomEmail();
    await page.locator('input[name="email"]').pressSequentially(email, { delay: 20 });

    await page.locator('select#SELECTOR_1').selectOption('10'); // Month
    await page.locator('select#SELECTOR_2').selectOption('15'); // Day
    await page.locator('select#SELECTOR_3').selectOption('1995'); // Year

    await page.locator('button[data-testid="ocfSignupNextLink"]').click();
    await page.locator('input[name="verfication_code"]').pressSequentially('123456', { delay: 20 });
});
