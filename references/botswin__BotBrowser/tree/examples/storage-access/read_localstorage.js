/**
 * Read LocalStorage from BotBrowser storage (ENT Tier1)
 *
 * LocalStorage uses LevelDB format and is stored as plaintext by default.
 * This script reads and parses the LevelDB data.
 *
 * Usage:
 *   node read_localstorage.js "/path/to/User Data" [origin_filter]
 *
 * Examples:
 *   node read_localstorage.js "/tmp/botbrowser-session"
 *   node read_localstorage.js "/tmp/botbrowser-session" "https://example.com"
 */

const { Level } = require('level');
const path = require('path');
const fs = require('fs');

/**
 * Parse LocalStorage key format
 * Keys are prefixed with metadata: _<origin_length>\x00<origin>
 */
function parseKey(key) {
  try {
    // LocalStorage keys have format: META:<origin>\x00<key> or _<len>\x00<origin>\x00<key>
    const keyStr = key.toString('latin1');

    // Try to extract origin and key name
    const nullIndex = keyStr.indexOf('\x00');
    if (nullIndex === -1) {
      return { origin: null, key: keyStr };
    }

    const prefix = keyStr.slice(0, nullIndex);
    const rest = keyStr.slice(nullIndex + 1);

    // Find second null byte for the actual key
    const secondNull = rest.indexOf('\x00');
    if (secondNull === -1) {
      return { origin: prefix, key: rest };
    }

    return {
      origin: rest.slice(0, secondNull),
      key: rest.slice(secondNull + 1)
    };
  } catch (e) {
    return { origin: null, key: key.toString() };
  }
}

/**
 * Read LocalStorage data from LevelDB
 * @param {string} userDataDir - Path to User Data directory
 * @param {string} [originFilter] - Optional origin filter (e.g., "https://example.com")
 * @returns {Promise<Map>} Map of origin -> {key: value}
 */
async function readLocalStorage(userDataDir, originFilter = null) {
  const lsPath = path.join(userDataDir, 'Default', 'Local Storage', 'leveldb');

  if (!fs.existsSync(lsPath)) {
    throw new Error(`LocalStorage directory not found: ${lsPath}`);
  }

  const db = new Level(lsPath, {
    createIfMissing: false,
    errorIfExists: false,
    keyEncoding: 'buffer',
    valueEncoding: 'buffer'
  });

  const storage = new Map();

  try {
    await db.open();

    for await (const [key, value] of db.iterator()) {
      const parsed = parseKey(key);

      // Skip internal LevelDB keys
      if (!parsed.origin || parsed.origin.startsWith('META:')) {
        continue;
      }

      // Apply origin filter if specified
      if (originFilter && !parsed.origin.includes(originFilter.replace(/^https?:\/\//, ''))) {
        continue;
      }

      if (!storage.has(parsed.origin)) {
        storage.set(parsed.origin, {});
      }

      // Try to decode value as UTF-16LE (Chrome stores strings this way)
      let decodedValue;
      try {
        // Skip first byte (type indicator) and decode as UTF-16LE
        if (value.length > 1) {
          decodedValue = value.slice(1).toString('utf16le');
        } else {
          decodedValue = value.toString();
        }
      } catch (e) {
        decodedValue = value.toString();
      }

      storage.get(parsed.origin)[parsed.key] = decodedValue;
    }
  } finally {
    await db.close();
  }

  return storage;
}

/**
 * Read SessionStorage (if persisted)
 * Note: SessionStorage is typically not persisted to disk
 */
async function readSessionStorage(userDataDir, originFilter = null) {
  const ssPath = path.join(userDataDir, 'Default', 'Session Storage', 'leveldb');

  if (!fs.existsSync(ssPath)) {
    return new Map();
  }

  const db = new Level(ssPath, {
    createIfMissing: false,
    errorIfExists: false,
    keyEncoding: 'buffer',
    valueEncoding: 'buffer'
  });

  const storage = new Map();

  try {
    await db.open();

    for await (const [key, value] of db.iterator()) {
      const parsed = parseKey(key);

      if (!parsed.origin || parsed.origin.startsWith('META:')) {
        continue;
      }

      if (originFilter && !parsed.origin.includes(originFilter.replace(/^https?:\/\//, ''))) {
        continue;
      }

      if (!storage.has(parsed.origin)) {
        storage.set(parsed.origin, {});
      }

      let decodedValue;
      try {
        if (value.length > 1) {
          decodedValue = value.slice(1).toString('utf16le');
        } else {
          decodedValue = value.toString();
        }
      } catch (e) {
        decodedValue = value.toString();
      }

      storage.get(parsed.origin)[parsed.key] = decodedValue;
    }
  } finally {
    await db.close();
  }

  return storage;
}

/**
 * Format storage data as JSON
 */
function toJSON(storage) {
  const obj = {};
  for (const [origin, data] of storage) {
    obj[origin] = data;
  }
  return JSON.stringify(obj, null, 2);
}

// CLI usage
if (require.main === module) {
  const args = process.argv.slice(2);

  if (args.length < 1) {
    console.error('Usage: node read_localstorage.js <user_data_dir> [origin_filter]');
    console.error('');
    console.error('Examples:');
    console.error('  node read_localstorage.js "/tmp/botbrowser-session"');
    console.error('  node read_localstorage.js "/tmp/botbrowser-session" "example.com"');
    process.exit(1);
  }

  const userDataDir = args[0];
  const originFilter = args[1] || null;

  (async () => {
    try {
      const storage = await readLocalStorage(userDataDir, originFilter);

      console.log(`Found LocalStorage data for ${storage.size} origins${originFilter ? ` matching "${originFilter}"` : ''}:\n`);

      for (const [origin, data] of storage) {
        const keys = Object.keys(data);
        console.log(`[${origin}] (${keys.length} keys)`);

        for (const [key, value] of Object.entries(data)) {
          const displayValue = value.length > 100
            ? value.slice(0, 100) + '...'
            : value;

          // Try to detect JSON values
          let formattedValue = displayValue;
          if (value.startsWith('{') || value.startsWith('[')) {
            try {
              JSON.parse(value);
              formattedValue = '[JSON] ' + displayValue;
            } catch (e) {
              // Not valid JSON
            }
          }

          console.log(`  ${key}: ${formattedValue}`);
        }
        console.log('');
      }

      if (storage.size > 0) {
        console.log('\n--- JSON Export ---');
        console.log(toJSON(storage));
      }
    } catch (err) {
      console.error('Error reading LocalStorage:', err.message);
      process.exit(1);
    }
  })();
}

module.exports = { readLocalStorage, readSessionStorage, toJSON };
