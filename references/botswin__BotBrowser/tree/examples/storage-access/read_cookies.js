/**
 * Read cookies from BotBrowser storage (ENT Tier1)
 *
 * Usage:
 *   node read_cookies.js "/path/to/User Data" [domain_filter]
 *
 * Examples:
 *   node read_cookies.js "/tmp/botbrowser-session"
 *   node read_cookies.js "/tmp/botbrowser-session" ".github.com"
 */

const Database = require('better-sqlite3');
const path = require('path');

/**
 * Convert WebKit timestamp (microseconds since 1601-01-01) to JavaScript Date
 * Chromium stores cookie timestamps in this format
 */
function webkitToDate(webkitMicros) {
  if (!webkitMicros) return null;
  // WebKit epoch is Jan 1, 1601. Unix epoch is Jan 1, 1970.
  // Difference: 11644473600 seconds = 11644473600000000 microseconds
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
 * Read cookies from Chromium Cookies database
 * @param {string} userDataDir - Path to User Data directory
 * @param {string} [domainFilter] - Optional domain filter (e.g., ".example.com")
 * @returns {Array} Array of cookie objects
 */
function readCookies(userDataDir, domainFilter = null) {
  const cookiesPath = path.join(userDataDir, 'Default', 'Cookies');
  const db = new Database(cookiesPath, { readonly: true });

  let query = `
    SELECT
      host_key,
      name,
      encrypted_value,
      path,
      expires_utc,
      is_secure,
      is_httponly,
      samesite,
      creation_utc
    FROM cookies
  `;

  if (domainFilter) {
    query += ` WHERE host_key LIKE ?`;
  }

  query += ` ORDER BY host_key, name`;

  const stmt = db.prepare(query);
  const rows = domainFilter ? stmt.all(`%${domainFilter}%`) : stmt.all();

  const cookies = rows.map(row => {
    const expiresDate = webkitToDate(row.expires_utc);
    const createdDate = webkitToDate(row.creation_utc);
    return {
      domain: row.host_key,
      name: row.name,
      value: stripPrefix(row.encrypted_value),
      path: row.path,
      expires: expiresDate ? expiresDate.toISOString() : null,
      secure: !!row.is_secure,
      httpOnly: !!row.is_httponly,
      sameSite: ['unspecified', 'no_restriction', 'lax', 'strict'][row.samesite] || 'unspecified',
      created: createdDate ? createdDate.toISOString() : null
    };
  });

  db.close();
  return cookies;
}

/**
 * Format cookies as Netscape cookie file format
 */
function toNetscapeFormat(cookies) {
  const lines = ['# Netscape HTTP Cookie File'];
  for (const c of cookies) {
    const httpOnly = c.httpOnly ? '#HttpOnly_' : '';
    const secure = c.secure ? 'TRUE' : 'FALSE';
    const expires = c.expires ? Math.floor(new Date(c.expires).getTime() / 1000) : '0';
    lines.push(`${httpOnly}${c.domain}\tTRUE\t${c.path}\t${secure}\t${expires}\t${c.name}\t${c.value}`);
  }
  return lines.join('\n');
}

/**
 * Format cookies as JSON for fetch/axios
 */
function toHeaderString(cookies, domain) {
  return cookies
    .filter(c => domain.endsWith(c.domain.replace(/^\./, '')))
    .map(c => `${c.name}=${c.value}`)
    .join('; ');
}

// CLI usage
if (require.main === module) {
  const args = process.argv.slice(2);

  if (args.length < 1) {
    console.error('Usage: node read_cookies.js <user_data_dir> [domain_filter]');
    console.error('');
    console.error('Examples:');
    console.error('  node read_cookies.js "/tmp/botbrowser-session"');
    console.error('  node read_cookies.js "/tmp/botbrowser-session" ".github.com"');
    process.exit(1);
  }

  const userDataDir = args[0];
  const domainFilter = args[1] || null;

  try {
    const cookies = readCookies(userDataDir, domainFilter);

    console.log(`Found ${cookies.length} cookies${domainFilter ? ` matching "${domainFilter}"` : ''}:\n`);

    for (const cookie of cookies) {
      console.log(`[${cookie.domain}] ${cookie.name}=${cookie.value?.slice(0, 50)}${cookie.value?.length > 50 ? '...' : ''}`);
    }

    if (cookies.length > 0) {
      console.log('\n--- Netscape Format ---');
      console.log(toNetscapeFormat(cookies));

      if (domainFilter) {
        console.log('\n--- Cookie Header ---');
        console.log(toHeaderString(cookies, domainFilter.replace(/^\./, '')));
      }
    }
  } catch (err) {
    console.error('Error reading cookies:', err.message);
    process.exit(1);
  }
}

module.exports = { readCookies, toNetscapeFormat, toHeaderString, stripPrefix, webkitToDate };
