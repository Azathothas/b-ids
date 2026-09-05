# Storage Access Examples (ENT Tier1)

Read browser storage data (cookies, passwords, credit cards, LocalStorage) directly from disk after a BotBrowser session.

## Overview

ENT Tier1 profiles enable plaintext storage mode, allowing direct programmatic access to browser data without decryption. This is useful for:

- Extracting session cookies for API replay
- Migrating credentials across machines
- Debugging authentication flows
- Auditing stored payment methods

## Storage Locations

| Data Type | File Path | Format |
|-----------|-----------|--------|
| Cookies | `User Data/Default/Cookies` | SQLite |
| Passwords | `User Data/Default/Login Data` | SQLite |
| Credit Cards | `User Data/Default/Web Data` | SQLite |
| LocalStorage | `User Data/Default/Local Storage/leveldb/` | LevelDB |
| IndexedDB | `User Data/Default/IndexedDB/` | LevelDB |

## Examples

| File | Description |
|------|-------------|
| [read_cookies.js](read_cookies.js) | Extract cookies with domain filtering |
| [read_passwords.js](read_passwords.js) | Read saved login credentials |
| [read_credit_cards.js](read_credit_cards.js) | Access stored payment methods |
| [read_localstorage.js](read_localstorage.js) | Parse LocalStorage LevelDB data |

## Prerequisites

```bash
npm install better-sqlite3 level
```

## Usage

```bash
# Read all cookies
node read_cookies.js "/path/to/User Data"

# Filter by domain
node read_cookies.js "/path/to/User Data" ".example.com"

# Read passwords
node read_passwords.js "/path/to/User Data"

# Read credit cards
node read_credit_cards.js "/path/to/User Data"

# Read LocalStorage for a specific origin
node read_localstorage.js "/path/to/User Data" "https://example.com"
```

## Data Format

### Cookies

Cookies are stored with a `v00` prefix followed by the plaintext value:

```
v00<cookie_value>
```

The example scripts automatically strip this prefix.

### Passwords and Credit Cards

Similarly stored with `v00` prefix. The scripts handle the prefix removal automatically.

### LocalStorage and IndexedDB

These use LevelDB format and are stored as plaintext by default in Chromium. No special prefix handling needed.

## Security Notes

⚠️ **Handle extracted data responsibly.** These scripts access sensitive information:

- Store credentials securely
- Do not commit extracted data to version control
- Use for authorized testing and debugging only
- Delete extracted data when no longer needed

## Integration Example

Combine with Puppeteer to automate cookie extraction:

```javascript
const puppeteer = require('puppeteer-core');
const { readCookies } = require('./read_cookies');

async function extractSessionCookies() {
  const userDataDir = '/tmp/botbrowser-session';

  const browser = await puppeteer.launch({
    executablePath: process.env.BOTBROWSER_EXEC_PATH,
    args: [
      `--bot-profile=${process.env.BOT_PROFILE_PATH}`,
      `--user-data-dir=${userDataDir}`
    ]
  });

  const page = await browser.newPage();
  await page.goto('https://example.com/login');
  // ... perform login
  await browser.close();

  // Extract cookies after browser closes
  const cookies = readCookies(userDataDir, '.example.com');
  console.log('Session cookies:', cookies);
}
```

---

**[Legal Disclaimer & Terms of Use](https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md) • [Responsible Use Guidelines](https://github.com/botswin/BotBrowser/blob/main/RESPONSIBLE_USE.md)**. BotBrowser is for authorized fingerprint protection and privacy research only.
