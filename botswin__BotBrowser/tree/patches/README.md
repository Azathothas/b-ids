# BotBrowser Patch Examples

For Fingerprint Protection and Privacy Research.

> These are selected examples from BotBrowser's internals for fingerprint protection and privacy research. The full core remains proprietary for compatibility testing.

BotBrowser includes proprietary enhancements built on top of Chromium, developed in-house for fingerprint protection and privacy research.

Only the GUI ([BotBrowserLauncher](https://github.com/botswin/BotBrowser/tree/main/launcher)) is open source.
The patches here are select examples published to illustrate techniques for fingerprint protection and privacy research without exposing the full engine.

⚠️ Usage policy: These examples are for fingerprint protection, privacy research, and authorized analysis only. Use in compliance with institutional policies and applicable laws.

---

## What's Inside

- **Proprietary Foundation**
  BotBrowser’s engine is built in-house, with no forks or external project references.

- **Selective Academic Disclosure**
  Only select patches are published for educational purposes. The rest remain proprietary to maintain compatibility testing integrity.

- **Comprehensive Compatibility Coverage**
  These patches address fingerprint protection across multiple surfaces, informed by:
  1. Security analysis of browser compatibility mechanisms
  2. V8-level study of tracking data collection and validation
  3. Compatibility testing experience in controlled environments

## Contact (Privacy Research Inquiries Only)

For fingerprint protection and privacy research collaboration or licensing review, please reach out:

<table>
  <tr><td>Email</td><td><a href="mailto:support@botbrowser.io">support@botbrowser.io</a></td></tr>
  <tr><td>Telegram</td><td><a href="https://t.me/botbrowser_support">@botbrowser_support</a></td></tr>
</table>

---

## Directory Tree

Below is the structure patches-showing which Chromium source files we’ve tweaked:

```
patches_v2/136.0.7103.49/
│
├─ base/
│  ├─ android/java/src/org/chromium/base/process_launcher/
│  └─ version_info/
│
├─ botbrowser/
│
├─ build/
│  └─ linux/sysroot_scripts/
│
├─ chrome/
│  ├─ android/java/
│  ├─ app/
│  ├─ browser/
│  │  ├─ ash/login/
│  │  ├─ devtools/
│  │  └─ ui/startup/
│  ├─ common/
│  ├─ installer/linux/debian/
│  └─ renderer/
│
├─ chromecast/
│  └─ browser/
│
├─ components/
│  ├─ component_updater/
│  ├─ crash/core/app/
│  ├─ embedder_support/
│  ├─ gwp_asan/client/
│  ├─ language/core/browser/
│  ├─ nacl/zygote/
│  ├─ password_manager/core/browser/
│  ├─ payments/core/
│  ├─ policy/core/common/cloud/
│  ├─ prefs/
│  ├─ update_client/net/
│  └─ version_info/
│
├─ content/
│  ├─ app/
│  ├─ browser/
│  │  ├─ bluetooth/
│  │  ├─ client_hints/
│  │  ├─ devtools/protocol/
│  │  ├─ gpu/
│  │  ├─ quota/
│  │  ├─ renderer_host/
│  │  ├─ speech/
│  │  └─ web_contents/
│  ├─ child/
│  ├─ gpu/
│  ├─ public/browser/
│  ├─ shell/browser/
│  └─ zygote/
│
├─ gpu/
│  ├─ command_buffer/client/
│  └─ config/
│
├─ media/
│  └─ capture/video/
│
├─ pdf/
│
├─ services/
│  ├─ network/p2p/
│  └─ passage_embeddings/
│
├─ third_party/
│  ├─ blink/
│  │  ├─ common/
│  │  │  ├─ device_memory/
│  │  │  └─ web_preferences/
│  │  ├─ public/common/web_preferences/
│  │  └─ renderer/
│  │     ├─ core/
│  │     │  ├─ css/, dom/, events/, execution_context/
│  │     │  ├─ exported/, frame/, html/, inspector/
│  │     │  ├─ layout/, loader/, permissions_policy/
│  │     │  └─ timezone/, timing/
│  │     ├─ modules/
│  │     │  ├─ battery/, canvas/, encoding/, encryptedmedia/
│  │     │  ├─ keyboard/, mediarecorder/, mediasource/
│  │     │  ├─ mediastream/, peerconnection/, plugins/
│  │     │  └─ quota/, webaudio/, webgl/, webgpu/
│  │     └─ platform/
│  │        ├─ fonts/
│  │        └─ media/
│  ├─ crashpad/crashpad/client/
│  └─ harfbuzz-ng/src/src/
│
├─ tools/
│  └─ gritsettings/
│
├─ ui/
│  ├─ base/l10n/
│  └─ resources/fonts/
│
└─ v8/
   └─ src/
      ├─ execution/
      ├─ heap/
      ├─ inspector/
      └─ numbers/
```


---

## Build Guide

See [build/README.md](build/README.md) for instructions on building BotBrowser from Chromium source, including platform-specific `args.gn` configurations.

---

**[Legal Disclaimer & Terms of Use](https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md) • [Responsible Use Guidelines](https://github.com/botswin/BotBrowser/blob/main/RESPONSIBLE_USE.md)**. BotBrowser is for authorized fingerprint protection and privacy research only.
