#!/usr/bin/env bash
set -e

echo "=========================================================="
echo "    CAPRI DESKTOP LINUX RPM ISOLATED TEST HARNESS         "
echo "=========================================================="

echo "[1/4] Checking RPM package installation..."
rpm -qi capri-desktop | grep -E "Name|Version|Architecture|Install Date|Summary"

echo ""
echo "[2/4] Testing reachability of Ollama construct..."
OLLAMA_URL="${OLLAMA_URL:-http://127.0.0.1:11434}"
if curl -s "$OLLAMA_URL/api/tags" > /dev/null; then
    echo "  -> Connected to Ollama at $OLLAMA_URL"
else
    echo "  -> Warning: Cannot reach Ollama at $OLLAMA_URL directly."
fi

echo ""
echo "[3/4] Running headless Capri smoke-test (Sprites, Fonts, Model)..."
/usr/bin/capri --ollama-url "$OLLAMA_URL" --model capri --smoke-test

echo ""
echo "[4/4] Executing test chat prompt loop with 3B abliterated Capri construct..."
echo "--- Prompt 1 ---"
capri --ollama-url "$OLLAMA_URL" --model capri --test-chat "Capri, confirm construct integrity."

echo ""
echo "--- Prompt 2 ---"
capri --ollama-url "$OLLAMA_URL" --model capri --test-chat "What is your opinion on Arasaka?"

echo ""
echo "[BONUS] Verifying X11 GUI launch in virtual display (Xvfb)..."
xvfb-run -a -s "-screen 0 1024x768x24" capri --smoke-test

echo ""
echo "=========================================================="
echo "  ALL CHECKS PASSED: Capri RPM is fully operational!      "
echo "=========================================================="
