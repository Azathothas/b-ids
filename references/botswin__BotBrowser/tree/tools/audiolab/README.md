# AudioLab: Web Audio Forensics for Privacy Protection (Beta)

Record every Web Audio API call to see exactly what tracking code is doing with audio fingerprinting. Study collection techniques and verify that BotBrowser's privacy protections work against them.

---

<a id="what-is-audiolab"></a>

## What Is AudioLab?

AudioLab records every Web Audio API call so you can see exactly what tracking code is trying to collect. Audio fingerprinting is a common tracking surface used by many systems to identify users through platform-specific audio processing differences.

- **See what trackers collect**: watch every AudioContext, node creation, parameter setting, and data extraction call
- **Identify recipes**: automatic detection of common audio fingerprinting patterns
- **Visualize audio graphs**: interactive topology view showing node routing (oscillator -> compressor -> destination)
- **Inspect extracted data**: see sample previews, sums, frequency data, and codec checks
- **Test your defenses**: verify that BotBrowser's audio noise protection works

---

<a id="try-it-now"></a>

## Try It Now

> **[Launch Audio Viewer](https://botswin.github.io/BotBrowser/tools/audiolab/audio_viewer.html?jsonl=https://botswin.github.io/BotBrowser/tools/audiolab/audio_sample.jsonl)**: Interactive demo preloaded with a sample recording. Filter events, view the audio graph topology, and inspect extracted data.

### Demo Resources

| Resource | Description |
|----------|-------------|
| **[Audio Viewer](../../docs/tools/audiolab/audio_viewer.html)** | Interactive JSONL viewer with graph visualization and recipe detection |
| **[Audio Test Scene](../../docs/tools/audiolab/audio_test.html)** | Web Audio API test page that exercises all recorded surfaces |
| **[Sample JSONL](../../docs/tools/audiolab/audio_sample.jsonl)** | Multi-context recording of the test scene (149 events, 9 contexts, 15 event types) |

---

<a id="development-status"></a>

## Current Development Status

| Component | Status |
|-----------|--------|
| **JSONL Recording** | **Shipped**: complete Web Audio API call recording |
| **Audio Viewer** | **Shipped**: HTML-based event viewer with graph topology and recipe detection |
| **Recipe Detection** | **Shipped**: automatic identification of common audio fingerprinting patterns |
| **Profile Integration** | **Planned**: audio fingerprint data in profiles for cross-platform consistency |

---

<a id="quick-start"></a>

## Quick Start: Recording Audio Fingerprint Collection

**Step 1: Start recording** (see [`--bot-audio-record-file`](../../CLI_FLAGS.md#flag-bot-audio-record-file))
```bash
chromium \
  --bot-profile=/absolute/path/to/profile.enc \
  --bot-audio-record-file=/tmp/audiolab.jsonl \
  --user-data-dir="$(mktemp -d)"
```

**Step 2: Visit a site and let tracking happen**
Go to the website you want to study. Let it load normally. AudioLab will record every Web Audio API call the tracking code makes.

**Step 3: Look at what was recorded**
Close BotBrowser. Your recording is saved to `/tmp/audiolab.jsonl`. Open it in the [Audio Viewer](https://botswin.github.io/BotBrowser/tools/audiolab/audio_viewer.html) or inspect it directly:

```bash
# Pretty-print all events
cat /tmp/audiolab.jsonl | jq .

# Show only data extraction events
cat /tmp/audiolab.jsonl | jq 'select(.type == "read_channel_data" or .type == "analyser_read")'

# Show the audio graph topology
cat /tmp/audiolab.jsonl | jq 'select(.type == "connect")'
```

---

<a id="recording-format"></a>

## Recording Format

### Event Types

| Event Type | Description | Example |
|------------|-------------|---------|
| **`context_create`** | OfflineAudioContext / AudioContext creation | channels, frames, sampleRate |
| **`node_create`** | Audio node instantiation | oscillator, dynamics_compressor, analyser, gain, biquad_filter, and more |
| **`param_set`** | Parameter value changes | type, frequency, threshold, knee, ratio, attack, release |
| **`connect`** | Node routing topology | oscillator -> compressor -> destination |
| **`disconnect`** | Node disconnection | node.disconnect() |
| **`start`** | Oscillator/source start | start(when) |
| **`stop`** | Source stop | stop() |
| **`start_rendering`** | OfflineAudioContext rendering trigger | startRendering() |
| **`read_channel_data`** | AudioBuffer data extraction | channel, sample_count, first/last 10 values, sum |
| **`read_copy_from_channel`** | AudioBuffer copyFromChannel | channel, copy_count, offset |
| **`analyser_read`** | AnalyserNode data reads | getFloatFrequencyData, getFloatTimeDomainData, getByte* |
| **`reduction_read`** | DynamicsCompressor reduction | reduction value |
| **`freq_response`** | BiquadFilter/IIR frequency response | magnitude + phase arrays |
| **`property_read`** | Context property access | baseLatency, outputLatency, getOutputTimestamp |
| **`codec_check`** | Codec support queries | canPlayType, MediaSource.isTypeSupported |

---

<a id="use-cases"></a>

## Use Cases

| Scenario | How AudioLab Helps |
|----------|-------------------|
| **Privacy Defense Development** | See how trackers try to fingerprint users through audio and build stronger protections |
| **Tracking Analysis** | Document exactly what Web Audio API calls tracking code makes |
| **Recipe Identification** | Automatically detect which audio fingerprinting pattern a site uses |
| **Cross-Platform Validation** | Compare recordings from different platforms to verify consistent protection |
| **Privacy Research** | Study real-world audio fingerprint collection with complete records |

---

<a id="whats-next"></a>

## What's Next

### Profile Integration (Planned)
Store collected audio fingerprint data in profiles so BotBrowser can return consistent audio output across platforms:
- **Cross-platform consistency**: same audio fingerprint on macOS, Windows, and Linux
- **Recipe-aware interception**: recognize and respond to each fingerprinting pattern
- **Authentic values**: captured from real devices for maximum compatibility

---

**[Legal Disclaimer & Terms of Use](https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md) - [Responsible Use Guidelines](https://github.com/botswin/BotBrowser/blob/main/RESPONSIBLE_USE.md)**. BotBrowser is for authorized fingerprint protection and privacy research only.
