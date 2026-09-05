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
import { generateRandomEmail, generateRandomPassword, generateRandomUsername, getDateFormatted, sleep } from '../utils';

test('southwest', async ({ page }) => {
    const tomorrowDate = getDateFormatted(1);

    const apiResponsePromise = page.waitForResponse((response) =>
        response.url().endsWith('/air-booking/page/air/booking/shopping')
    );
    await page.goto(
        `https://www.southwest.com/air/booking/select-depart.html?int=HOMEQBOMAIR&adultPassengersCount=1&departureDate=${tomorrowDate}&destinationAirportCode=LGA&fareType=USD&originationAirportCode=ATL&passengerType=ADULT&promoCode=&returnDate=&tripType=oneway&from=ATL&to=LGA&adultsCount=1&departureTimeOfDay=ALL_DAY&reset=true&returnTimeOfDay=ALL_DAY`
    );

    const apiResponse = await apiResponsePromise;
    const apiResponseJson = await apiResponse.json();

    expect(apiResponseJson).toHaveProperty('data');
});

test('target', async ({ page }) => {
    await page.goto('https://www.target.com/');
    await page.locator('a#account-sign-in').click();
    await page.locator('button >> text=Sign in or create account').click();
    await page.locator('input#username').pressSequentially(generateRandomEmail(), {
        delay: 20,
    });
    await page.keyboard.press('Enter');
    await page.locator('input#firstname').pressSequentially(
        Math.random()
            .toString(36)
            .substring(2)
            .replace(/[^a-zA-Z]/g, ''),
        { delay: 20 }
    );
    await page.locator('input#lastname').pressSequentially(
        Math.random()
            .toString(36)
            .substring(2)
            .replace(/[^a-zA-Z]/g, ''),
        { delay: 20 }
    );
    await page.locator('input#password-checkbox').click();
    await page.locator('input#password').pressSequentially(generateRandomPassword(), {
        delay: 20,
    });
    await sleep(2000);
    await page.locator('button#createAccount').click();
    await page.locator('h1 >> text=Verification code sent').waitFor();
});

test('nordstrom', async ({ page }) => {
    test.setTimeout(120_000);
    await page.goto('https://www.nordstrom.com/');
    await sleep(3_000);
    await page.goto('https://www.nordstrom.com/signin');
    const email = generateRandomEmail();
    await page.locator('input[name="email"]').pressSequentially(email, { delay: 20 });
    await page.keyboard.press('Enter');
    const firstName = generateRandomUsername();
    await page.locator('input[name="firstName"]').pressSequentially(firstName, { delay: 20 });
    const lastName = generateRandomUsername();
    await page.locator('input[name="lastName"]').pressSequentially(lastName, { delay: 20 });
    const password = generateRandomPassword();
    await page.locator('input[name="password"]').pressSequentially(password, { delay: 20 });
    await page.locator('button[alt="create account button"]').click();
    await page.locator("h1 >> text=Get rewarded—it's free").waitFor();
    await page.goto(
        'https://www.nordstrom.com/s/the-perfect-t-shirt/7031683?origin=category-personalizedsort&breadcrumb=Home%2FNew%20Arrivals%2FWomen%2FClothing&color=019'
    );
});
