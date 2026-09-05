# RESPONSIBLE USE GUIDELINES

BotBrowser is a privacy protection platform designed for fingerprint protection validation. Browser fingerprinting is recognized as a privacy threat by [W3C, major browser vendors, and EU regulators](FINGERPRINT_PRIVACY.md). These guidelines clarify the expectations for anyone who accesses binaries, profiles, or source artifacts.

## REQUIRED CONDITIONS

- **AUTHORIZATION:** Operate the software only on systems you own or have written permission to test. Maintain signed approvals or tickets for auditing.
- **CONTROLLED ENVIRONMENTS:** Prefer testbeds, sandboxes, or clearly documented demo endpoints. Production services require explicit opt-in from the owner.
- **SYNTHETIC DATA ONLY:** Use test credentials, synthetic identities, or anonymized datasets. Never process personal data or live customer accounts.
- **LEGAL COMPLIANCE:** Follow all applicable laws (e.g., CFAA, GDPR, CCPA) and institutional policies. When uncertain, consult qualified legal counsel before proceeding.
- **ETHICS REVIEW:** Obtain IRB or equivalent ethics clearance whenever research involves user-impacting systems or data collection.

## PROHIBITED ACTIVITIES

- Deploying BotBrowser against production systems without explicit authorization from the system owner.
- Harvesting personal data, account credentials, session tokens, or payment information.
- Providing the software to third parties who intend to violate laws or service terms.
- Using BotBrowser to facilitate fraud, spam, scalping, unauthorized ticket purchasing, or other unauthorized activities.

## RESPONSIBLE DISCLOSURE & COOPERATION

- **ABUSE REPORTING:** Service operators or security vendors can contact the maintainers at [support@botbrowser.io](mailto:support@botbrowser.io). Include evidence (timestamps, IPs, scripts) so investigations can begin immediately.
- **LICENSE REVOCATION:** Maintainers may revoke access to binaries or profiles for any breach of these rules.
- **VENDOR COLLABORATION:** We coordinate with security vendors to share indicators of compromise and mitigate ongoing abuse.
- **TESTING REPORTS:** Qualified vendors who need additional visibility into BotBrowser evaluations may request redacted test reports or joint review calls after verifying ownership of the affected property.

## PUBLICATION & SHARING

- When publishing research, clearly state the privacy protection or analytical objective. Do not release operational details or techniques that could facilitate unauthorized use.
- Redact sensitive partner data from papers, blog posts, talks, and sample code.
- Respect third-party disclosure timelines and embargo agreements.

## SECURITY HYGIENE

- Store profiles and binaries in secured locations with restricted access.
- Rotate proxy credentials and other secrets used in experiments.
- Update promptly to the latest BotBrowser release to obtain security fixes and policy updates.

By continuing to use BotBrowser you acknowledge and accept these requirements in addition to the project [Legal Disclaimer](DISCLAIMER.md). Contact the maintainers with any questions before you proceed.
