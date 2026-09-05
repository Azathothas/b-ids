# Virgin Australia Capture with Docker

For Fingerprint Protection and Privacy Research. See the project [Legal Disclaimer](../../DISCLAIMER.md) and [Responsible Use Guidelines](../../RESPONSIBLE_USE.md) for usage boundaries.

## Project Overview

This setup runs a Node.js `puppeteer-core` script inside Docker that:

1. Launches BotBrowser headless (no XDISPLAY or GPU required), with optional per-launch proxy.
2. Iterates JFK → BOS dates across **Jan–Jun 2026**.
3. Waits on `https://book.virginiaustralia.com/api/graphql` (operation `bookingAirSearch`) and writes JSON to `flight_data/` as `mm-dd-yyyy.json`.
4. Picks a random profile from `profiles/` per launch; `PROXY_SERVER` supports `{SID}` placeholder per launch.
5. The target site is protected by **Imperva**, and the workflow demonstrates that BotBrowser prevents fingerprint collection in complex tracking scenarios.

---

## Repo Layout

```
virginaustralia_docker/
├── Dockerfile
├── docker-compose.yml
├── main.js
├── package.json
├── package-lock.json
├── profiles/          # Place multiple profile files here; one is chosen at random per browser launch
└── flight_data/       # Captured GraphQL responses
```

> **Tip:** Review `Dockerfile` and `docker-compose.yml` for mounts, env vars, and proxy templates.

---

## Getting Started

> **Reminder:** run this workflow only against environments where you have explicit authorization.

1. **Add Files**
   * Place `botbrowser_*.deb` in the repo root.
   * Put your profile files (e.g., `*.enc`) into `profiles/`.

2. **Launch with Docker Compose** (proxy optional; `{SID}` is replaced per launch):

   ```bash
   PROXY_SERVER="socks5h://user_sid_{SID}:pass@gate.example.com:7000" docker compose up --build -d
   ```

3. **Monitor Logs**

   ```bash
   docker-compose logs -f virginaustralia-botbrowser-service
   ```

4. **View Results**
   * GraphQL JSON per day: `flight_data/mm-dd-yyyy.json`

This capture script is not intended for bulk data collection and must only be run in test setups where you have explicit authorization.

---

## main.js at a Glance

* Iterates dates Jan–Jun 2026.
* Launches one browser per date with optional proxy and random profile selection.
* Waits for `bookingAirSearch` GraphQL response and saves it to `flight_data/`.
* Retries a date if the target response is not captured.

---

## Troubleshooting

| Issue                         | Fix                                                                   |
| ----------------------------- | --------------------------------------------------------------------- |
| Missing `.deb`                | Add `botbrowser_*.deb` to the repo root                               |
| Profiles not found            | Ensure files exist under `profiles/`                                  |
| Permission errors             | Make sure `profiles/` and `flight_data/` are writable                 |
| Proxy issues                  | Verify `PROXY_SERVER` format; `{SID}` will be replaced per launch     |
| Docker startup failure        | Run without `-d` to see errors: `docker compose up`                   |

---

## Configuration

Environment variables (see `docker-compose.yml`):

- `BOTBROWSER_EXEC_PATH` (required): path to the BotBrowser Chromium binary inside the container.
- `BOT_PROFILES_DIR` (optional): directory holding profile files; one is chosen at random per browser launch.
- `PROXY_SERVER` (optional): template supporting `{SID}` for per-launch proxy values.

Adjust URLs or timeouts inside `main.js` if the Virgin Australia flow changes.

---

**[Legal Disclaimer & Terms of Use](https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md) • [Responsible Use Guidelines](https://github.com/botswin/BotBrowser/blob/main/RESPONSIBLE_USE.md)**. BotBrowser is for authorized fingerprint protection and privacy research only.
