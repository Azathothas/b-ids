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

test('oneclick', async ({ page }) => {
    await page.goto('https://www.tencentcloud.com/products/captcha');
    await page.locator('div[report-ext1="type-one-click"]').waitFor({ state: 'visible', timeout: 5000 });

    await page.evaluate(() => {
        const el = document.querySelector('div[report-ext1="type-one-click"]');
        if (el) {
            (el as HTMLElement).scrollIntoView({
                behavior: 'smooth',
                block: 'center',
                inline: 'center',
            });
        }
    });

    await page.locator('#tencent-captcha-dy__robot_checkBox_id').click({ force: true });
    const doneImg = page.locator('img.tencent-captcha-dy__robot-checkBox-img-done');
    await doneImg.waitFor({ state: 'visible', timeout: 5000 });
});
