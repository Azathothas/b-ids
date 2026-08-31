/**
 * Global Test Configuration
 *
 * Playwright test configuration for BotBrowser test suite.
 * - Configures default timeouts and cleanup behaviors
 * - Removes Playwright artifacts to maintain consistent fingerprint for validation
 * - For fingerprint protection and privacy research purposes
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 */

import { test as base } from '@playwright/test';
import { sleep } from './utils';

base.setTimeout(60_000);

base.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
        // @ts-expect-error - Playwright binding will cause leak
        delete window.__playwright__binding__;
        // @ts-expect-error - Playwright binding will cause leak
        delete window.__pwInitScripts;
        // @ts-expect-error - Playwright builtins will cause leak
        delete window.__playwright_builtins__;
    });
});

base.afterEach(async () => {
    await sleep(8_000); // sleep for 8 seconds for better video recording
});

export * from '@playwright/test';
export const test = base;
