/**
 * ⚠️ PRIVACY RESEARCH USE ONLY
 * Run exclusively in authorized anti-tracking labs that comply with all applicable laws.
 * See: https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

/**
 * BotBrowser Per-Context Proxy Example (Java)
 *
 * This example demonstrates how to use different proxies for each browser context
 * while maintaining automatic geo-detection for timezone, locale, and languages.
 *
 * Key Features:
 * - Multiple contexts with different proxies in a single browser instance
 * - Automatic timezone/locale detection per proxy
 * - Cost-effective alternative to launching multiple browser processes
 */

import com.microsoft.playwright.*;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.Map;

public class PerContextProxy {
    public static void main(String[] args) {
        String botBrowserExecPath = System.getenv("BOTBROWSER_EXEC_PATH");
        String botProfilePath = System.getenv("BOT_PROFILE_PATH");

        if (botBrowserExecPath == null || botBrowserExecPath.isEmpty()) {
            botBrowserExecPath = "/path/to/chrome"; // Update this path
        }
        if (botProfilePath == null || botProfilePath.isEmpty()) {
            botProfilePath = "/absolute/path/to/profile.enc"; // Update this path
        }

        try (Playwright playwright = Playwright.create()) {
            // Launch BotBrowser with profile but no global proxy
            BrowserType.LaunchOptions launchOptions = new BrowserType.LaunchOptions()
                .setExecutablePath(Paths.get(botBrowserExecPath))
                .setHeadless(false) // Set to true for headless mode
                .setArgs(Arrays.asList(
                    "--bot-profile=" + botProfilePath,
                    "--user-data-dir=" + System.getProperty("java.io.tmpdir") + "/botbrowser-" + System.currentTimeMillis()
                ));

            Browser browser = playwright.chromium().launch(launchOptions);
            System.out.println("🚀 Browser launched successfully");

            try {
                // Context 1: US Proxy
                System.out.println("\n📍 Creating Context 1 with US proxy...");
                BrowserContext context1 = browser.newContext(new Browser.NewContextOptions()
                    .setProxy(new Proxy("http://username:password@us-proxy.example.com:8080"))); // Replace with your US proxy

                // Context 2: EU Proxy
                System.out.println("📍 Creating Context 2 with EU proxy...");
                BrowserContext context2 = browser.newContext(new Browser.NewContextOptions()
                    .setProxy(new Proxy("socks5://username:password@eu-proxy.example.com:1080"))); // Replace with your EU proxy

                // Context 3: APAC Proxy
                System.out.println("📍 Creating Context 3 with APAC proxy...");
                BrowserContext context3 = browser.newContext(new Browser.NewContextOptions()
                    .setProxy(new Proxy("http://username:password@apac-proxy.example.com:8080"))); // Replace with your APAC proxy

                // Test each context
                testContext(context1, "US Context", "https://httpbin.org/ip");
                testContext(context2, "EU Context", "https://httpbin.org/ip");
                testContext(context3, "APAC Context", "https://httpbin.org/ip");

                // Demonstrate timezone detection
                System.out.println("\n🌍 Testing automatic timezone detection...");
                testTimezone(context1, "US Context");
                testTimezone(context2, "EU Context");
                testTimezone(context3, "APAC Context");

            } catch (Exception error) {
                System.err.println("❌ Error: " + error.getMessage());
            } finally {
                System.out.println("\n🔄 Closing browser...");
                browser.close();
                System.out.println("✅ Done!");
            }
        }
    }

    private static void testContext(BrowserContext context, String label, String testUrl) {
        System.out.println("\n🧪 Testing " + label + ":");

        Page page = context.newPage();

        // Remove Playwright bindings to avoid detection
        page.addInitScript(
            "delete window.__playwright__binding__;\n" +
            "delete window.__pwInitScripts;"
        );

        try {
            page.navigate(testUrl, new Page.NavigateOptions().setWaitUntil(WaitUntilState.NETWORKIDLE));

            // Get IP information
            String ipInfo = page.locator("pre").textContent();
            System.out.println("   📡 IP Response: " + ipInfo.trim());

            // Get browser timezone (automatically set by BotBrowser based on proxy IP)
            String timezone = (String) page.evaluate("() => Intl.DateTimeFormat().resolvedOptions().timeZone");
            System.out.println("   🕐 Detected Timezone: " + timezone);

            // Get browser language (automatically set by BotBrowser based on proxy IP)
            String language = (String) page.evaluate("() => navigator.language");
            System.out.println("   🌐 Detected Language: " + language);

        } catch (Exception error) {
            System.err.println("   ❌ Error testing " + label + ": " + error.getMessage());
        } finally {
            page.close();
        }
    }

    private static void testTimezone(BrowserContext context, String label) {
        Page page = context.newPage();

        try {
            // Test timezone consistency
            @SuppressWarnings("unchecked")
            Map<String, Object> timezoneInfo = (Map<String, Object>) page.evaluate(
                "() => {\n" +
                "  const now = new Date();\n" +
                "  return {\n" +
                "    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,\n" +
                "    offset: now.getTimezoneOffset(),\n" +
                "    localTime: now.toLocaleString(),\n" +
                "    utcTime: now.toUTCString()\n" +
                "  };\n" +
                "}"
            );

            System.out.println("   🕐 " + label + " Timezone Details:");
            System.out.println("      Timezone: " + timezoneInfo.get("timezone"));
            System.out.println("      UTC Offset: " + timezoneInfo.get("offset") + " minutes");
            System.out.println("      Local Time: " + timezoneInfo.get("localTime"));

        } catch (Exception error) {
            System.err.println("   ❌ Error getting timezone for " + label + ": " + error.getMessage());
        } finally {
            page.close();
        }
    }
}
