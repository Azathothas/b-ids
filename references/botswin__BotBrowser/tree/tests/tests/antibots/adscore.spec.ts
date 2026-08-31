/**
 * PRIVACY RESEARCH USE ONLY
 *
 * Run exclusively inside authorized privacy research labs with synthetic data only.
 * Never target production traffic or violate any Terms of Service.
 *
 * See https://github.com/botswin/BotBrowser/blob/main/tests/README.md
 * and https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */


import { expect, test } from '../global-setup';

test('adscore', async ({ page }) => {
    await page.goto(
        'https://c.adsco.re/r#apikey=Qk9JAwAAAAAAHRNIGwgsoprZU7T52-yq0utL_w&type=3&data=AAIl4FMAOsV08FZ1WZ-jchAo7O_nJGJes_KQ4gykpsfM67g8PevWVrKIg1F7Gi_OXOwRiuTcn5G5jeHKdh096kM2GnPidKm36PYM_Qjb3kntETht9wrO4svi4J8OgDWXYvo'
    );

    expect(await page.waitForRequest('https://thisisgoodtraffic11.com/')).toBeTruthy();
});
