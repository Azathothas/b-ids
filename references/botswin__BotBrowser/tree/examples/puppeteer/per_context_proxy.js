/**
 * ⚠️ PRIVACY RESEARCH USE ONLY
 * Run exclusively in authorized privacy research labs that comply with all applicable laws.
 * See: https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

/**
 * BotBrowser Per-Context Proxy Example (Puppeteer)
 *
 * This example demonstrates how to use different proxies for each browser context
 * while maintaining automatic geo-detection for timezone, locale, and languages.
 *
 * Key Features:
 * - Multiple contexts with different proxies in a single browser instance
 * - Automatic timezone/locale detection per proxy
 * - Cost-effective alternative to launching multiple browser processes
 */

const puppeteer = require('puppeteer-core');
const os = require('os');

(async () => {
  // Launch BotBrowser with profile but no global proxy
  const browser = await puppeteer.launch({
    headless: false, // Set to true for headless mode
    executablePath: process.env.BOTBROWSER_EXEC_PATH || '/path/to/chrome', // Update this path
    args: [
      `--bot-profile=${process.env.BOT_PROFILE_PATH || '/absolute/path/to/profile.enc'}`, // Update this path
      '--user-data-dir=' + os.tmpdir() + '/botbrowser-' + Date.now(),
    ],
  });

  console.log('🚀 Browser launched successfully');

  try {
    // Context 1: US Proxy
    console.log('\n📍 Creating Context 1 with US proxy...');
    const context1 = await browser.createBrowserContext({
      proxyServer: 'http://username:password@us-proxy.example.com:8080', // Replace with your US proxy
    });

    // Context 2: EU Proxy
    console.log('📍 Creating Context 2 with EU proxy...');
    const context2 = await browser.createBrowserContext({
      proxyServer: 'socks5://username:password@eu-proxy.example.com:1080', // Replace with your EU proxy
    });

    // Context 3: APAC Proxy
    console.log('📍 Creating Context 3 with APAC proxy...');
    const context3 = await browser.createBrowserContext({
      proxyServer: 'http://username:password@apac-proxy.example.com:8080', // Replace with your APAC proxy
    });

    // Test each context
    await testContext(context1, 'US Context', 'https://httpbin.org/ip');
    await testContext(context2, 'EU Context', 'https://httpbin.org/ip');
    await testContext(context3, 'APAC Context', 'https://httpbin.org/ip');

    // Demonstrate timezone detection
    console.log('\n🌍 Testing automatic timezone detection...');
    await testTimezone(context1, 'US Context');
    await testTimezone(context2, 'EU Context');
    await testTimezone(context3, 'APAC Context');

  } catch (error) {
    console.error('❌ Error:', error.message);
  } finally {
    console.log('\n🔄 Closing browser...');
    await browser.close();
    console.log('✅ Done!');
  }
})();

async function testContext(context, label, testUrl) {
  console.log(`\n🧪 Testing ${label}:`);

  const page = await context.newPage();
  try {
    await page.goto(testUrl, { waitUntil: 'networkidle2' });

    // Get IP information
    const ipInfo = await page.$eval('pre', el => el.textContent);
    console.log(`   📡 IP Response: ${ipInfo.trim()}`);

    // Get browser timezone (automatically set by BotBrowser based on proxy IP)
    const timezone = await page.evaluate(() => Intl.DateTimeFormat().resolvedOptions().timeZone);
    console.log(`   🕐 Detected Timezone: ${timezone}`);

    // Get browser language (automatically set by BotBrowser based on proxy IP)
    const language = await page.evaluate(() => navigator.language);
    console.log(`   🌐 Detected Language: ${language}`);

  } catch (error) {
    console.log(`   ❌ Error testing ${label}: ${error.message}`);
  } finally {
    await page.close();
  }
}

async function testTimezone(context, label) {
  const page = await context.newPage();

  try {
    // Test timezone consistency
    const timezoneInfo = await page.evaluate(() => {
      const now = new Date();
      return {
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        offset: now.getTimezoneOffset(),
        localTime: now.toLocaleString(),
        utcTime: now.toUTCString()
      };
    });

    console.log(`   🕐 ${label} Timezone Details:`);
    console.log(`      Timezone: ${timezoneInfo.timezone}`);
    console.log(`      UTC Offset: ${timezoneInfo.offset} minutes`);
    console.log(`      Local Time: ${timezoneInfo.localTime}`);

  } catch (error) {
    console.log(`   ❌ Error getting timezone for ${label}: ${error.message}`);
  } finally {
    await page.close();
  }
}
