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

test('smartcaptcha', async ({ page }) => {
    await page.goto('https://smartcaptcha.yandexcloud.net/demo');
    const frame = page.frameLocator('iframe[data-testid="checkbox-iframe"]');
    await frame.locator('input#js-button').waitFor({ state: 'visible', timeout: 5000 });
    await frame.locator('input#js-button').click({ force: true });
    await frame
        .locator('div.CheckboxCaptcha-Checkbox[data-checked="true"]')
        .waitFor({ state: 'visible', timeout: 5000 });
    await page.locator('input#smartcaptcha-demo-submit').click();
    await page.locator('div.greeting >> text=Hello, user!').waitFor({
        state: 'visible',
        timeout: 5000,
    });
});
