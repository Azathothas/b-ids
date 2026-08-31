#!/bin/bash
# BotBrowser Benchmark -Setup benchmark suites (local deployment)
#
# Downloads Speedometer 3.0, JetStream 2, and MotionMark for local serving.
# Repos: https://github.com/WebKit/Speedometer
#        https://github.com/WebKit/JetStream
#        https://github.com/WebKit/MotionMark

set -e
cd "$(dirname "$0")"
mkdir -p benchmarks

echo "=== Setting up benchmark suites ==="

# Speedometer 3.0
# Branch release/3.0 contains the release version
if [ -d "benchmarks/speedometer" ]; then
  echo "[OK] Speedometer 3.0 already cloned"
else
  echo "[...] Cloning Speedometer 3.0 (release/3.0 branch)..."
  git clone --depth 1 --branch release/3.0 https://github.com/WebKit/Speedometer.git benchmarks/speedometer
  echo "[...] Installing Speedometer dependencies..."
  cd benchmarks/speedometer && npm install && cd ../..
  echo "[OK] Speedometer 3.0 ready"
fi

# JetStream 2
if [ -d "benchmarks/jetstream" ]; then
  echo "[OK] JetStream 2 already cloned"
else
  echo "[...] Cloning JetStream 2..."
  git clone --depth 1 https://github.com/WebKit/JetStream.git benchmarks/jetstream
  echo "[...] Installing JetStream dependencies..."
  cd benchmarks/jetstream && npm install 2>/dev/null && cd ../.. || cd ../..
  echo "[OK] JetStream 2 ready"
fi

# MotionMark
if [ -d "benchmarks/motionmark" ]; then
  echo "[OK] MotionMark already cloned"
else
  echo "[...] Cloning MotionMark..."
  git clone --depth 1 https://github.com/WebKit/MotionMark.git benchmarks/motionmark
  echo "[OK] MotionMark ready"
fi

echo ""
echo "=== Setup complete ==="
echo ""
echo "Benchmark directories:"
ls -la benchmarks/
echo ""
echo "Next steps:"
echo "  1. npm install              # Install playwright-core"
echo "  2. node scripts/run-all.js  # Run all benchmarks"
echo ""
echo "Individual runs:"
echo "  node scripts/bench-baseline.js --mode headless"
echo "  node scripts/bench-baseline.js --mode headed"
echo "  node scripts/bench-scale.js --mode headless --max-scale 50"
echo "  node scripts/bench-scale.js --mode headed --max-scale 50"
echo ""
echo "NOTE: Speedometer 3.0 requires 'npm run build' in some cases."
echo "      If the local serve fails, try: cd benchmarks/speedometer && npm run build"
