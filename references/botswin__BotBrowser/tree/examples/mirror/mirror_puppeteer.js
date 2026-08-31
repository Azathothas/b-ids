#!/usr/bin/env node

const fs = require('fs');
const puppeteer = require('puppeteer-core');

const CHROMIUM = process.env.BOTBROWSER_EXEC_PATH || 'chromium';
const PROFILE = process.env.BOT_PROFILE_PATH;
const URL = process.env.BOT_URL || 'google.com';

function parseEndpoint(endpoint) {
  const [host, port] = endpoint.split(':');
  return { host, port: parseInt(port, 10) };
}

const CONTROLLER = parseEndpoint(process.env.BOT_MIRROR_CONTROLLER_ENDPOINT || '127.0.0.1:9990');
const CLIENT = parseEndpoint(process.env.BOT_MIRROR_CLIENT_ENDPOINT || '127.0.0.1:9990');

if (!PROFILE || !fs.existsSync(PROFILE)) {
  console.error('BOT_PROFILE_PATH environment variable must point to a valid .enc file');
  process.exit(1);
}

async function launch(x) {
  return puppeteer.launch({
    executablePath: CHROMIUM,
    headless: false,
    args: [
      `--bot-profile=${PROFILE}`,
      `--window-position=${x},50`,
      '--window-size=400,600',
      URL,
    ],
    defaultViewport: null,
  });
}

async function main() {
  const browsers = await Promise.all([
    launch(10),
    launch(430),
    launch(850),
  ]);

  const sessions = await Promise.all(browsers.map(b => b.target().createCDPSession()));

  await new Promise(r => setTimeout(r, 1000));

  await sessions[0].send('BotBrowser.startMirrorController', {
    bindHost: CONTROLLER.host,
    port: CONTROLLER.port,
  });

  await Promise.all([
    sessions[1].send('BotBrowser.startMirrorClient', { host: CLIENT.host, port: CLIENT.port }),
    sessions[2].send('BotBrowser.startMirrorClient', { host: CLIENT.host, port: CLIENT.port }),
  ]);

  console.log('Mirror active. Interact with controller (left). Clients sync automatically.\n');

  process.on('SIGINT', shutdown);
  await new Promise(() => {});

  async function shutdown() {
    try {
      await Promise.all(sessions.map(s => s.send('BotBrowser.stopMirror')));
    } catch (err) {
      // ignore shutdown errors
    }
    await Promise.all(browsers.map(b => b.close()));
    process.exit(0);
  }
}

main().catch(err => {
  console.error(err.message);
  process.exit(1);
});
