/**
 * Read saved passwords from BotBrowser storage (ENT Tier1)
 *
 * Usage:
 *   node read_passwords.js "/path/to/User Data" [url_filter]
 *
 * Examples:
 *   node read_passwords.js "/tmp/botbrowser-session"
 *   node read_passwords.js "/tmp/botbrowser-session" "github.com"
 */

const Database = require('better-sqlite3');
const path = require('path');

/**
 * Convert WebKit timestamp (microseconds since 1601-01-01) to JavaScript Date
 */
function webkitToDate(webkitMicros) {
  if (!webkitMicros) return null;
  const WEBKIT_EPOCH_DIFF = 11644473600000000n;
  try {
    const unixMicros = BigInt(webkitMicros) - WEBKIT_EPOCH_DIFF;
    return new Date(Number(unixMicros / 1000n));
  } catch (e) {
    return null;
  }
}

/**
 * Strip v00 prefix from plaintext storage value
 */
function stripPrefix(buffer) {
  if (!buffer) return null;
  const str = buffer.toString();
  if (str.startsWith('v00')) {
    return str.slice(3);
  }
  return str;
}

/**
 * Read passwords from Chromium Login Data database
 * @param {string} userDataDir - Path to User Data directory
 * @param {string} [urlFilter] - Optional URL filter
 * @returns {Array} Array of credential objects
 */
function readPasswords(userDataDir, urlFilter = null) {
  const loginDataPath = path.join(userDataDir, 'Default', 'Login Data');
  const db = new Database(loginDataPath, { readonly: true });

  let query = `
    SELECT
      origin_url,
      action_url,
      username_element,
      username_value,
      password_element,
      password_value,
      signon_realm,
      date_created,
      date_last_used,
      times_used
    FROM logins
  `;

  if (urlFilter) {
    query += ` WHERE origin_url LIKE ? OR signon_realm LIKE ?`;
  }

  query += ` ORDER BY date_last_used DESC`;

  const stmt = db.prepare(query);
  const rows = urlFilter
    ? stmt.all(`%${urlFilter}%`, `%${urlFilter}%`)
    : stmt.all();

  const credentials = rows.map(row => {
    const createdDate = webkitToDate(row.date_created);
    const lastUsedDate = webkitToDate(row.date_last_used);
    return {
      url: row.origin_url,
      actionUrl: row.action_url,
      username: row.username_value,
      password: stripPrefix(row.password_value),
      usernameField: row.username_element,
      passwordField: row.password_element,
      realm: row.signon_realm,
      created: createdDate ? createdDate.toISOString() : null,
      lastUsed: lastUsedDate ? lastUsedDate.toISOString() : null,
      timesUsed: row.times_used
    };
  });

  db.close();
  return credentials;
}

/**
 * Format credentials as CSV
 */
function toCSV(credentials) {
  const header = 'url,username,password,realm,last_used,times_used';
  const rows = credentials.map(c => {
    const escape = (s) => `"${(s || '').replace(/"/g, '""')}"`;
    return [
      escape(c.url),
      escape(c.username),
      escape(c.password),
      escape(c.realm),
      escape(c.lastUsed),
      c.timesUsed
    ].join(',');
  });
  return [header, ...rows].join('\n');
}

/**
 * Format as Chrome password export format (for import to other browsers)
 */
function toChromeExportFormat(credentials) {
  const header = 'name,url,username,password';
  const rows = credentials.map(c => {
    const escape = (s) => `"${(s || '').replace(/"/g, '""')}"`;
    let name = c.url;
    try {
      name = new URL(c.url).hostname;
    } catch (e) {
      // Use URL as-is if parsing fails
    }
    return [
      escape(name),
      escape(c.url),
      escape(c.username),
      escape(c.password)
    ].join(',');
  });
  return [header, ...rows].join('\n');
}

// CLI usage
if (require.main === module) {
  const args = process.argv.slice(2);

  if (args.length < 1) {
    console.error('Usage: node read_passwords.js <user_data_dir> [url_filter]');
    console.error('');
    console.error('Examples:');
    console.error('  node read_passwords.js "/tmp/botbrowser-session"');
    console.error('  node read_passwords.js "/tmp/botbrowser-session" "github.com"');
    process.exit(1);
  }

  const userDataDir = args[0];
  const urlFilter = args[1] || null;

  try {
    const credentials = readPasswords(userDataDir, urlFilter);

    console.log(`Found ${credentials.length} saved passwords${urlFilter ? ` matching "${urlFilter}"` : ''}:\n`);

    for (const cred of credentials) {
      const maskedPassword = cred.password
        ? cred.password.slice(0, 2) + '*'.repeat(Math.max(0, cred.password.length - 2))
        : '(empty)';
      console.log(`[${cred.realm}]`);
      console.log(`  Username: ${cred.username || '(empty)'}`);
      console.log(`  Password: ${maskedPassword}`);
      console.log(`  Last used: ${cred.lastUsed || 'never'}`);
      console.log('');
    }

    if (credentials.length > 0) {
      console.log('--- Chrome Export Format (CSV) ---');
      console.log(toChromeExportFormat(credentials));
    }
  } catch (err) {
    console.error('Error reading passwords:', err.message);
    process.exit(1);
  }
}

module.exports = { readPasswords, toCSV, toChromeExportFormat, stripPrefix, webkitToDate };
