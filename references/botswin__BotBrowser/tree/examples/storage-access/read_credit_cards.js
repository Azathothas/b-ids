/**
 * Read saved credit cards from BotBrowser storage (ENT Tier1)
 *
 * Usage:
 *   node read_credit_cards.js "/path/to/User Data"
 *
 * Examples:
 *   node read_credit_cards.js "/tmp/botbrowser-session"
 */

const Database = require('better-sqlite3');
const path = require('path');

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
 * Detect card type from number
 */
function detectCardType(number) {
  if (!number) return 'unknown';
  const n = number.replace(/\D/g, '');
  if (/^4/.test(n)) return 'visa';
  if (/^5[1-5]/.test(n)) return 'mastercard';
  if (/^3[47]/.test(n)) return 'amex';
  if (/^6(?:011|5)/.test(n)) return 'discover';
  if (/^35/.test(n)) return 'jcb';
  if (/^3(?:0[0-5]|[68])/.test(n)) return 'diners';
  return 'unknown';
}

/**
 * Mask card number for display
 */
function maskCardNumber(number) {
  if (!number) return null;
  const n = number.replace(/\D/g, '');
  if (n.length < 8) return number;
  return n.slice(0, 4) + ' **** **** ' + n.slice(-4);
}

/**
 * Read credit cards from Chromium Web Data database
 * @param {string} userDataDir - Path to User Data directory
 * @returns {Array} Array of credit card objects
 */
function readCreditCards(userDataDir) {
  const webDataPath = path.join(userDataDir, 'Default', 'Web Data');
  const db = new Database(webDataPath, { readonly: true });

  // Read basic card info
  const cardsQuery = `
    SELECT
      guid,
      name_on_card,
      expiration_month,
      expiration_year,
      card_number_encrypted,
      date_modified,
      origin,
      use_count,
      use_date,
      billing_address_id,
      nickname
    FROM credit_cards
    ORDER BY use_date DESC
  `;

  const cardsStmt = db.prepare(cardsQuery);
  const cardRows = cardsStmt.all();

  const cards = cardRows.map(row => {
    const cardNumber = stripPrefix(row.card_number_encrypted);
    return {
      guid: row.guid,
      nameOnCard: row.name_on_card,
      cardNumber: cardNumber,
      cardNumberMasked: maskCardNumber(cardNumber),
      cardType: detectCardType(cardNumber),
      expirationMonth: row.expiration_month,
      expirationYear: row.expiration_year,
      expiration: row.expiration_month && row.expiration_year
        ? `${String(row.expiration_month).padStart(2, '0')}/${row.expiration_year}`
        : null,
      nickname: row.nickname,
      origin: row.origin,
      useCount: row.use_count,
      lastUsed: row.use_date ? new Date(row.use_date * 1000).toISOString() : null,
      modified: row.date_modified ? new Date(row.date_modified * 1000).toISOString() : null,
      billingAddressId: row.billing_address_id
    };
  });

  // Try to read CVC if available (stored separately in some Chrome versions)
  try {
    const cvcQuery = `
      SELECT
        guid,
        value_encrypted
      FROM local_stored_cvc
    `;
    const cvcStmt = db.prepare(cvcQuery);
    const cvcRows = cvcStmt.all();

    const cvcMap = new Map();
    for (const row of cvcRows) {
      cvcMap.set(row.guid, stripPrefix(row.value_encrypted));
    }

    for (const card of cards) {
      card.cvc = cvcMap.get(card.guid) || null;
    }
  } catch (e) {
    // local_stored_cvc table may not exist in all Chrome versions
  }

  db.close();
  return cards;
}

/**
 * Read IBAN accounts from Web Data
 */
function readIBANs(userDataDir) {
  const webDataPath = path.join(userDataDir, 'Default', 'Web Data');
  const db = new Database(webDataPath, { readonly: true });

  try {
    const query = `
      SELECT
        guid,
        value_encrypted,
        nickname,
        use_count,
        use_date
      FROM local_ibans
      ORDER BY use_date DESC
    `;

    const stmt = db.prepare(query);
    const rows = stmt.all();

    const ibans = rows.map(row => ({
      guid: row.guid,
      iban: stripPrefix(row.value_encrypted),
      nickname: row.nickname,
      useCount: row.use_count,
      lastUsed: row.use_date ? new Date(row.use_date * 1000).toISOString() : null
    }));

    db.close();
    return ibans;
  } catch (e) {
    db.close();
    return [];
  }
}

// CLI usage
if (require.main === module) {
  const args = process.argv.slice(2);

  if (args.length < 1) {
    console.error('Usage: node read_credit_cards.js <user_data_dir>');
    console.error('');
    console.error('Examples:');
    console.error('  node read_credit_cards.js "/tmp/botbrowser-session"');
    process.exit(1);
  }

  const userDataDir = args[0];

  try {
    // Read credit cards
    const cards = readCreditCards(userDataDir);
    console.log(`Found ${cards.length} saved credit cards:\n`);

    for (const card of cards) {
      console.log(`[${card.cardType.toUpperCase()}] ${card.cardNumberMasked}`);
      console.log(`  Name: ${card.nameOnCard || '(not set)'}`);
      console.log(`  Expires: ${card.expiration || '(not set)'}`);
      if (card.cvc) {
        console.log(`  CVC: ${card.cvc}`);
      }
      if (card.nickname) {
        console.log(`  Nickname: ${card.nickname}`);
      }
      console.log(`  Used: ${card.useCount} times`);
      console.log(`  Last used: ${card.lastUsed || 'never'}`);
      console.log('');
    }

    // Read IBANs
    const ibans = readIBANs(userDataDir);
    if (ibans.length > 0) {
      console.log(`\nFound ${ibans.length} saved IBANs:\n`);
      for (const iban of ibans) {
        console.log(`[IBAN] ${iban.iban}`);
        if (iban.nickname) {
          console.log(`  Nickname: ${iban.nickname}`);
        }
        console.log(`  Used: ${iban.useCount} times`);
        console.log('');
      }
    }

    // Output full card numbers (with warning)
    if (cards.length > 0) {
      console.log('\n--- Full Card Details (SENSITIVE) ---');
      for (const card of cards) {
        console.log(`${card.cardType}: ${card.cardNumber} | ${card.expiration} | CVC: ${card.cvc || 'N/A'} | ${card.nameOnCard}`);
      }
    }
  } catch (err) {
    console.error('Error reading credit cards:', err.message);
    process.exit(1);
  }
}

module.exports = { readCreditCards, readIBANs, maskCardNumber, detectCardType, stripPrefix };
