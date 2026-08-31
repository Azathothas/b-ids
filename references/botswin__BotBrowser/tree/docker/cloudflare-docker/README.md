# Cloudflare Compatibility Testing with Docker

For Fingerprint Protection and Privacy Research. See the project [Legal Disclaimer](../../DISCLAIMER.md) and [Responsible Use Guidelines](../../RESPONSIBLE_USE.md) for usage boundaries.

## Project Overview

This repository contains the Docker setup for validating fingerprint protection with Cloudflare using BotBrowser and Playwright, which runs a Python script (`main.py`) inside a Docker container that:

1. Launches BotBrowser in headless mode: **no XDISPLAY or GPU required**: while simulating a full desktop environment.
2. Performs compatibility validation on Cloudflare's demo interface.
3. Saves screenshots to `screenshots/` for analysis.

---

## Repo Layout

```
.
├── Dockerfile
├── docker-compose.yml
├── main.py
├── requirements.txt
├── profile/        # Place your .enc profile here
└── screenshots/    # Output screenshots per date
```

> **Tip:** Review `Dockerfile` and `docker-compose.yml` for configuration options.

---

## Getting Started

> **Reminder:** run this workflow only with explicit authorization from the environment owner.

1. **Add Files**

   * Place `botbrowser_*.deb` in the root directory.
   * Put your profile file (`your-profile.enc`) into `profile/`.

2. **Launch with Docker Compose**

   ```bash
   docker-compose up --build -d
   ```

3. **Monitor Logs**

   ```bash
   docker-compose logs -f cloudflare-botbrowser-service
   ```

4. **View Outputs**

   * Screenshots in `screenshots/`

This compatibility testing script is not intended for bulk data collection and must only be run in explicitly authorized test setups.

---

## Troubleshooting

| Issue                  | Fix                                                                   |
| ---------------------- | --------------------------------------------------------------------- |
| Missing `.deb`         | Add `botbrowser_*.deb` to the repo root                               |
| Profile not found      | Ensure `profile/your-profile.enc` exists                              |
| Permission errors      | Make sure `profile/` and `screenshots/` are writable |
| Docker startup failure | Run without `-d` to see errors: `docker-compose up`                   |

---

**[Legal Disclaimer & Terms of Use](https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md) • [Responsible Use Guidelines](https://github.com/botswin/BotBrowser/blob/main/RESPONSIBLE_USE.md)**. BotBrowser is for authorized fingerprint protection and privacy research only.
