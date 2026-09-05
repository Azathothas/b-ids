# BotBrowser Fingerprint Protection Testing Suite

Automated validation framework for preventing fingerprint collection and cross-platform tracking prevention. Browser fingerprinting is recognized as a privacy threat by [W3C, major browser vendors, and EU regulators](../FINGERPRINT_PRIVACY.md).

Demonstrates controlled fingerprint protection validation using [Playwright](https://playwright.dev/docs/writing-tests) for academic and privacy research purposes. Operate these scripts in accordance with the project [Legal Disclaimer](../DISCLAIMER.md) and [Responsible Use Guidelines](../RESPONSIBLE_USE.md). If you observe any misuse, contact [support@botbrowser.io](mailto:support@botbrowser.io) so we can investigate and support affected services.

## Research Environment Setup

All tests utilize **controlled network environments** with appropriate proxy configurations for privacy research and fingerprint protection validation. Testing should be conducted through authorized institutional networks.

---

## Usage Context

These materials target university labs, privacy research groups, and other approved testing programs. Typical scenarios include:
- Fingerprint protection validation studies to prevent tracking systems from collecting identification data
- Academic coursework or workshops supervised by faculty covering privacy and browser APIs
- Privacy benchmarking in dedicated lab environments
- Authorized security research with written approval

Out-of-scope uses include production attacks, terms-of-service violations, or handling real customer data. When in doubt, obtain written permission and institutional ethics review (IRB or equivalent) before running any scenario.

### Test Data Policy

**All Test Scripts Use Only Synthetic/Invalid Data:**
- Random/generated usernames (not real accounts)
- Fake SSNs (e.g., "6789," which is obviously invalid)
- Non-existent email addresses
- Invalid credentials that will never authenticate
- Test endpoints and demo environments

**Never Use:**
- ❌ Real user accounts or credentials
- ❌ Valid personal information (SSN, DOB, etc.)
- ❌ Stolen or leaked credentials
- ❌ Production API endpoints without authorization

---

## Getting Started

### Step 1: Research Environment Setup

Create a `.env` file in the project root with your configuration:

```bash
BOTBROWSER_EXEC_PATH=/absolute/path/to/botbrowser
BOT_PROFILE_PATH=/absolute/path/to/profile.enc
```

**Example Configuration:**
```bash
BOTBROWSER_EXEC_PATH=/usr/local/bin/chromium
BOT_PROFILE_PATH=/home/user/profiles/profile.enc
```

**Required Configuration:**
- `BOTBROWSER_EXEC_PATH` → BotBrowser executable
- `BOT_PROFILE_PATH` → Profile file (.enc file)

### Step 2: Install & Run Research Suite

```bash
# Install dependencies
npm install

# Run tests
npx playwright test

# Generate analysis report
npx playwright show-report
```

---

## Research Methodology

**Purpose:** Controlled validation scripts for fingerprint protection and privacy research.

**Framework:** All validation must be conducted within authorized environments that comply with institutional ethics guidelines, applicable laws, and protocols.

**Data Collection:** Fingerprint data collected only from publicly accessible test interfaces and demonstration sites under controlled, non-production conditions.

⚠️ **Important:** Results from these tests demonstrate fingerprint protection validation and should not be used as instructions for unauthorized access or violating production security controls.

### Responsible Research Guidelines

Before running any validation tests, ensure you meet ALL of these requirements:

1. **Authorization:** You own the system OR have explicit written permission for testing
2. **Test Environment:** Using authorized test or demo endpoints only, never production systems
3. **Synthetic Data:** Only synthetic or invalid data, no real user information or valid credentials
4. **Legal Compliance:** Full compliance with applicable laws (CFAA, GDPR, CCPA, etc.)
5. **Terms of Service:** Not violating any website or service terms of use
6. **Research Purpose:** Results used exclusively for privacy research or educational purposes

**If you cannot verify ALL boxes above, DO NOT proceed with testing.**

### Financial Services Testing: Authorization Requirements

Validation tests involving financial institutions (e.g., `threatmetrix.spec.ts` with Vanguard/Fidelity):
- ⚠️ Use ONLY publicly accessible demo pages or authorized test environments
- ⚠️ Submit ONLY synthetic or invalid data (test identifiers, obviously invalid values)
- ⚠️ Do NOT attempt actual account authentication or access real user accounts
- ⚠️ Intended to analyze tracking methodology in authorized research context only
- ⚠️ Any production environment testing requires explicit written authorization from the financial institution

### Validation Environment Troubleshooting

| Issue | Solution |
|-------|----------|
| Network access restrictions | Use authorized institutional network infrastructure |
| Tests failing | Verify `.env` file paths and profile compatibility |
| Browser environment issues | Check BotBrowser executable permissions and validation environment setup |

## Academic Resources

- [Playwright Documentation](https://playwright.dev/docs/writing-tests)
- [BotBrowser Profile Configs](https://github.com/botswin/BotBrowser/blob/main/profiles/PROFILE_CONFIGS.md)
- [Test Results & Reports](./test-results/)
- [ACM Code of Ethics](https://www.acm.org/code-of-ethics)

## Research Dependencies

This project uses the following open-source libraries:

| Package | Purpose |
|---------|---------|
| [dotenv](https://www.npmjs.com/package/dotenv) | Environment variable management |
| [ghost-cursor](https://www.npmjs.com/package/ghost-cursor) | Natural interaction simulation |
| [ghost-cursor-playwright](https://www.npmjs.com/package/ghost-cursor-playwright) | Playwright integration for realistic interactions |

**Research Notice:** All libraries are used exclusively for fingerprint protection validation under controlled conditions with proper institutional oversight.

---

**[Legal Disclaimer & Terms of Use](https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md) • [Responsible Use Guidelines](https://github.com/botswin/BotBrowser/blob/main/RESPONSIBLE_USE.md)**. BotBrowser is for authorized fingerprint protection and privacy research only.
