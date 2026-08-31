/**
 * ⚠️ PRIVACY RESEARCH USE ONLY
 * Run exclusively in authorized anti-tracking labs that comply with all applicable laws.
 * See: https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

/**
 * BotBrowser Per-Context Proxy Example (.NET)
 *
 * This example demonstrates how to use different proxies for each browser context
 * while maintaining automatic geo-detection for timezone, locale, and languages.
 *
 * Key Features:
 * - Multiple contexts with different proxies in a single browser instance
 * - Automatic timezone/locale detection per proxy
 * - Cost-effective alternative to launching multiple browser processes
 */

using Microsoft.Playwright;
using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;

class PerContextProxy
{
    static async Task Main(string[] args)
    {
        string botBrowserExecPath = Environment.GetEnvironmentVariable("BOTBROWSER_EXEC_PATH") ?? "/path/to/chrome"; // Update this path
        string botProfilePath = Environment.GetEnvironmentVariable("BOT_PROFILE_PATH") ?? "/absolute/path/to/profile.enc"; // Update this path

        using var playwright = await Playwright.CreateAsync();

        // Launch BotBrowser with profile but no global proxy
        var browser = await playwright.Chromium.LaunchAsync(new BrowserTypeLaunchOptions
        {
            ExecutablePath = botBrowserExecPath,
            Headless = false, // Set to true for headless mode
            Args = new[]
            {
                $"--bot-profile={botProfilePath}",
                $"--user-data-dir={Path.GetTempPath()}botbrowser-{DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()}"
            }
        });

        Console.WriteLine("🚀 Browser launched successfully");

        try
        {
            // Context 1: US Proxy
            Console.WriteLine("\n📍 Creating Context 1 with US proxy...");
            var context1 = await browser.NewContextAsync(new BrowserNewContextOptions
            {
                Proxy = new ProxySettings
                {
                    Server = "http://username:password@us-proxy.example.com:8080" // Replace with your US proxy
                }
            });

            // Context 2: EU Proxy
            Console.WriteLine("📍 Creating Context 2 with EU proxy...");
            var context2 = await browser.NewContextAsync(new BrowserNewContextOptions
            {
                Proxy = new ProxySettings
                {
                    Server = "socks5://username:password@eu-proxy.example.com:1080" // Replace with your EU proxy
                }
            });

            // Context 3: APAC Proxy
            Console.WriteLine("📍 Creating Context 3 with APAC proxy...");
            var context3 = await browser.NewContextAsync(new BrowserNewContextOptions
            {
                Proxy = new ProxySettings
                {
                    Server = "http://username:password@apac-proxy.example.com:8080" // Replace with your APAC proxy
                }
            });

            // Test each context
            await TestContext(context1, "US Context", "https://httpbin.org/ip");
            await TestContext(context2, "EU Context", "https://httpbin.org/ip");
            await TestContext(context3, "APAC Context", "https://httpbin.org/ip");

            // Demonstrate timezone detection
            Console.WriteLine("\n🌍 Testing automatic timezone detection...");
            await TestTimezone(context1, "US Context");
            await TestTimezone(context2, "EU Context");
            await TestTimezone(context3, "APAC Context");

        }
        catch (Exception error)
        {
            Console.Error.WriteLine($"❌ Error: {error.Message}");
        }
        finally
        {
            Console.WriteLine("\n🔄 Closing browser...");
            await browser.CloseAsync();
            Console.WriteLine("✅ Done!");
        }
    }

    private static async Task TestContext(IBrowserContext context, string label, string testUrl)
    {
        Console.WriteLine($"\n🧪 Testing {label}:");

        var page = await context.NewPageAsync();

        // Remove Playwright bindings to avoid detection
        await page.AddInitScriptAsync(@"
            delete window.__playwright__binding__;
            delete window.__pwInitScripts;
        ");

        try
        {
            await page.GotoAsync(testUrl, new PageGotoOptions { WaitUntil = WaitUntilState.NetworkIdle });

            // Get IP information
            var ipInfo = await page.Locator("pre").TextContentAsync();
            Console.WriteLine($"   📡 IP Response: {ipInfo?.Trim()}");

            // Get browser timezone (automatically set by BotBrowser based on proxy IP)
            var timezone = await page.EvaluateAsync<string>("() => Intl.DateTimeFormat().resolvedOptions().timeZone");
            Console.WriteLine($"   🕐 Detected Timezone: {timezone}");

            // Get browser language (automatically set by BotBrowser based on proxy IP)
            var language = await page.EvaluateAsync<string>("() => navigator.language");
            Console.WriteLine($"   🌐 Detected Language: {language}");

        }
        catch (Exception error)
        {
            Console.Error.WriteLine($"   ❌ Error testing {label}: {error.Message}");
        }
        finally
        {
            await page.CloseAsync();
        }
    }

    private static async Task TestTimezone(IBrowserContext context, string label)
    {
        var page = await context.NewPageAsync();

        try
        {
            // Test timezone consistency
            var timezoneInfo = await page.EvaluateAsync<Dictionary<string, object>>(@"() => {
                const now = new Date();
                return {
                    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
                    offset: now.getTimezoneOffset(),
                    localTime: now.toLocaleString(),
                    utcTime: now.toUTCString()
                };
            }");

            Console.WriteLine($"   🕐 {label} Timezone Details:");
            Console.WriteLine($"      Timezone: {timezoneInfo["timezone"]}");
            Console.WriteLine($"      UTC Offset: {timezoneInfo["offset"]} minutes");
            Console.WriteLine($"      Local Time: {timezoneInfo["localTime"]}");

        }
        catch (Exception error)
        {
            Console.Error.WriteLine($"   ❌ Error getting timezone for {label}: {error.Message}");
        }
        finally
        {
            await page.CloseAsync();
        }
    }
}
