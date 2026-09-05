# Mirror Examples

## mirror_puppeteer.js

Launch controller + 2 clients and activate Mirror via Chrome DevTools Protocol (CDP).

**Usage (local verification):**
```bash
BOTBROWSER_EXEC_PATH=/path/to/chromium \
BOT_PROFILE_PATH=/path/to/profile.enc \
node mirror_puppeteer.js
```

**Usage (distributed verification):**
```bash
BOTBROWSER_EXEC_PATH=/path/to/chromium \
BOT_PROFILE_PATH=/path/to/profile.enc \
BOT_MIRROR_CONTROLLER_ENDPOINT=0.0.0.0:9990 \
node mirror_puppeteer.js
```

**Environment variables:**
- `BOTBROWSER_EXEC_PATH` - Path to BotBrowser executable (default: chromium)
- `BOT_PROFILE_PATH` - Path to .enc profile file (required)
- `BOT_URL` - Target URL (default: google.com)
- `BOT_MIRROR_CONTROLLER_ENDPOINT` - Controller bind address (default: 127.0.0.1:9990, use 0.0.0.0:9990 for remote access)
- `BOT_MIRROR_CLIENT_ENDPOINT` - Remote controller address for clients (default: 127.0.0.1:9990, e.g., 192.168.1.100:9990 for cross-network)

## cli_launch.sh

Bash script to launch controller + 2 clients using command-line flags.

**Usage (local verification):**
```bash
BOTBROWSER_EXEC_PATH=/path/to/chromium \
BOT_PROFILE_PATH=/path/to/profile.enc \
./cli_launch.sh
```

**Usage (distributed cross-network verification):**
```bash
BOTBROWSER_EXEC_PATH=/path/to/chromium \
BOT_PROFILE_PATH=/path/to/profile.enc \
BOT_MIRROR_CONTROLLER_ENDPOINT=0.0.0.0:9990 \
BOT_MIRROR_CLIENT_ENDPOINT=192.168.1.100:9990 \
./cli_launch.sh
```

**Environment variables:**
- `BOTBROWSER_EXEC_PATH` - Path to BotBrowser executable (default: chromium)
- `BOT_PROFILE_PATH` - Path to .enc profile file (required)
- `BOT_URL` - Target URL (default: google.com)
- `BOT_MIRROR_CONTROLLER_ENDPOINT` - Controller bind address (default: 127.0.0.1:9990, use 0.0.0.0:9990 for network accessibility)
- `BOT_MIRROR_CLIENT_ENDPOINT` - Remote controller address (default: 127.0.0.1:9990, format: host:port)

---

See [Mirror Documentation](../../tools/mirror/) for complete usage guide. Guide: [Mirror Distributed](https://botbrowser.io/docs/deployment/mirror-distributed/)

---

**[Legal Disclaimer & Terms of Use](../../DISCLAIMER.md) • [Responsible Use Guidelines](../../RESPONSIBLE_USE.md)**. BotBrowser is for authorized fingerprint protection and privacy research only.
