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

test('mts', async ({ page }) => {
    await page.goto('https://moskva.mts.ru/personal/vybrat-nomer');
    await page.locator('button >> text=Продолжить').click();
    await page.locator('input[placeholder="Начните вводить название"]').fill('Москва');
    await page.locator('div[class="ScrollbarsCustom-Content"] div >> text=г Москва').click();
    await page.locator('button >> text=ПРОДОЛЖИТЬ С').click();
    await page.locator('h4 >> text=Войдите с номером телефона любого оператора').waitFor({
        state: 'visible',
        timeout: 30000,
    });
});
