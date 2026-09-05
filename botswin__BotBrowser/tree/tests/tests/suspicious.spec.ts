/**
 * PRIVACY RESEARCH USE ONLY
 *
 * Run exclusively inside authorized privacy research labs with synthetic data only.
 * Never target production traffic or violate any Terms of Service.
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */


import { expect, test } from './global-setup';

test('is devtools open', async ({ page }) => {
    await page.goto('about:blank');

    const isDevtoolsOpen = await page.evaluate(() => {
        function isDevtoolsOpen() {
            let result = false;
            const error = new Error();

            Object.defineProperty(error, 'stack', {
                configurable: false,
                enumerable: false,
                get: function () {
                    return (result = true), '';
                },
            });
            console.debug(error);
            return result;
        }

        return isDevtoolsOpen();
    });

    expect(isDevtoolsOpen).toBe(false);
});

test('PdfOopif feature turned on', async ({ page }) => {
    await page.goto('https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf');

    const embededPdf = await page.evaluate(() => {
        return document.querySelector('body embed')?.getAttribute('type') === 'application/pdf';
    });

    expect(embededPdf).toBe(true);
});
