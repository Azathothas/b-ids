import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import puppeteer from 'puppeteer-core';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const BOTBROWSER_EXEC_PATH = process.env.BOTBROWSER_EXEC_PATH || '';
const PROXY_SERVER_TEMPLATE = process.env.PROXY_SERVER || ''; // expects {SID} placeholder when per-context proxy is desired
const PROFILES_DIR = process.env.BOT_PROFILES_DIR || path.join(__dirname, 'profiles');

if (!BOTBROWSER_EXEC_PATH) {
  throw new Error('BOTBROWSER_EXEC_PATH variable must be set');
}

const GRAPHQL_ENDPOINT = 'https://book.virginaustralia.com/api/graphql';
const TARGET_TEMPLATE =
  'https://book.virginaustralia.com/dx/VADX/#/flight-selection?journeyType=one-way&activeMonth={ACTIVE_MONTH}&locale=en-GB&awardBooking=true&class=First&ADT=1&CHD=0&INF=0&origin=JFK&destination=BOS&date={SEARCH_DATE}&execution=e53bd793-4ecf-459e-af2b-5ee810d25114';

const OUTPUT_DIR = path.join(__dirname, 'flight_data');
const MONTH_START = new Date(Date.UTC(2026, 0, 1));
const MONTH_END_EXCLUSIVE = new Date(Date.UTC(2026, 6, 1));
const BROWSER_POOL_SIZE = 5; // how many concurrent browsers run

function pickProfile(profiles) {
  if (!profiles.length) return undefined;
  const idx = Math.floor(Math.random() * profiles.length);
  return profiles[idx];
}

// Worker factory so the pool logic stays outside the main run loop.
function createWorker(dates, profiles) {
  return async function worker(workerId) {
    // Each worker loops until the date queue is empty; once done with one date, it grabs the next.
    while (true) {
      // Use shift() so workers pull tasks from a shared queue; keeps the pool full without extra coordination.
      const currentDate = dates.shift();
      if (!currentDate) break;

      let browser;
      let context;
      let page;
      let graphqlResponsePromise;

      try {
        const activeMonth = new Date(Date.UTC(currentDate.getUTCFullYear(), currentDate.getUTCMonth(), 1));
        const dateLabel = new Date(currentDate)
          .toISOString()
          .slice(0, 10)
          .replace(/^(\d{4})-(\d{2})-(\d{2})$/, '$2-$3-$1');
        const activeMonthLabel = new Date(activeMonth)
          .toISOString()
          .slice(0, 10)
          .replace(/^(\d{4})-(\d{2})-(\d{2})$/, '$2-$3-$1');
        const targetUrl = TARGET_TEMPLATE.replace('{ACTIVE_MONTH}', activeMonthLabel).replace(
          '{SEARCH_DATE}',
          dateLabel,
        );

        // Per-browser proxy: replace {SID} with a random 1–6 digit number so each browser can fan out.
        const sid = Math.floor(Math.random() * 99999) + 1;
        const proxyServer = PROXY_SERVER_TEMPLATE ? PROXY_SERVER_TEMPLATE.replace('{SID}', String(sid)) : undefined;

        // One fresh browser per date to allow distinct bot-profile and proxy.
        const profilePath = pickProfile(profiles);
        const launchArgs = [
          '--headless',
          '--disable-blink-features=AutomationControlled',
          '--disable-audio-output',
          '--bot-webrtc-ice=google',
          '--no-first-run',
          '--password-store=basic',
          '--disable-dev-shm-usage',
          '--disable-extensions',
          '--disable-sync',
          ...(profilePath ? [`--bot-profile=${profilePath}`] : []),
          ...(proxyServer ? [`--proxy-server=${proxyServer}`] : []),
        ];

        browser = await puppeteer.launch({
          executablePath: BOTBROWSER_EXEC_PATH,
          headless: true,
          ignoreDefaultArgs: true, // drop puppeteer defaults that can leak automation flags; we supply our own args
          args: launchArgs,
        });

        context = await browser.createBrowserContext();
        page = await context.newPage();

        console.log(
          `[worker ${workerId}] Processing ${dateLabel} -> ${targetUrl} ${
            proxyServer ? `(proxy: ${proxyServer})` : ''
          }`,
        );

        // Wait for the bookingAirSearch GraphQL response before proceeding.
        // Filter by URL and request body so we only capture the target operation, not other GraphQL chatter.
        graphqlResponsePromise = page.waitForResponse(
          async (resp) => {
            if (!resp.url().startsWith(GRAPHQL_ENDPOINT)) return false;
            try {
              const rawBody = await resp.request().fetchPostData();
              const body = typeof rawBody === 'string' ? rawBody : rawBody ? Buffer.from(rawBody).toString('utf8') : '';
              return body.includes('bookingAirSearch');
            } catch {
              return false;
            }
          },
          { timeout: 30_000 },
        );

        await page.goto(targetUrl);

        const graphqlResponse = await graphqlResponsePromise;
        const responseJson = await graphqlResponse.json();
        const jsonPath = path.join(OUTPUT_DIR, `${dateLabel}.json`);
        await fs.promises.writeFile(jsonPath, JSON.stringify(responseJson, null, 2), 'utf8');
        console.log(`[worker ${workerId}] Saved GraphQL response: ${jsonPath}`);
      } catch (err) {
        console.warn(
          `[worker ${workerId}] Error for date ${currentDate?.toISOString?.() || 'unknown'}, requeueing`,
          err,
        );
        // Requeue the date so another worker (or the same) can retry after a transient failure.
        dates.push(currentDate);
      } finally {
        // Prevent unhandled rejections if the page/browser closed before the waitForResponse settled.
        if (graphqlResponsePromise) {
          graphqlResponsePromise.catch(() => {});
        }
        try {
          if (page && !page.isClosed()) await page.close();
        } catch {}
        try {
          if (context) await context.close();
        } catch {}
        try {
          if (browser) await browser.close();
        } catch {}
      }
    }
  };
}

async function run() {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });

  const dates = [];
  for (
    let cursor = new Date(MONTH_START.getTime());
    cursor < MONTH_END_EXCLUSIVE;
    cursor = new Date(cursor.getTime() + 24 * 60 * 60 * 1000)
  ) {
    dates.push(new Date(cursor.getTime()));
  }

  const profiles = fs
    .readdirSync(PROFILES_DIR, { withFileTypes: true })
    .filter((ent) => ent.isFile())
    .map((ent) => path.join(PROFILES_DIR, ent.name));
  if (!profiles.length) {
    console.warn(`No profiles found in ${PROFILES_DIR}; proceeding without --bot-profile`);
  }

  const workers = [];
  const poolSize = Math.min(BROWSER_POOL_SIZE, dates.length);
  for (let i = 0; i < poolSize; i += 1) {
    // Fixed-size pool: as soon as a worker finishes one date, it immediately grabs the next until none left.
    workers.push(createWorker(dates, profiles)(i + 1));
  }

  await Promise.all(workers);
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
