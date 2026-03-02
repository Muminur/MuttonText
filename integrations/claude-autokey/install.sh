#!/usr/bin/env bash
# MuttonText × Claude AutoKey Integration — Automated Installer
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC}   $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUTOKEY_DIR="$HOME/.config/autokey/data/claude-snippets"
ENV_FILE="$HOME/.config/muttontext/.env"

echo ""
echo "  MuttonText × Claude AutoKey Integration Installer"
echo "  ================================================="
echo ""

# --- Step 1: Check OS ---
info "Checking system..."
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    error "This integration requires Linux (Ubuntu/Debian). macOS support is not yet available."
fi

if ! command -v apt-get &>/dev/null; then
    error "apt-get not found. This installer supports Debian/Ubuntu only."
fi
success "Linux/Debian system detected."

# --- Step 2: Install system packages ---
info "Installing autokey-gtk and xclip..."
if command -v autokey-gtk &>/dev/null && command -v xclip &>/dev/null; then
    success "autokey-gtk and xclip already installed."
else
    if command -v pkexec &>/dev/null && [ -n "$DISPLAY" ]; then
        pkexec apt-get install -y autokey-gtk xclip
    else
        sudo apt-get install -y autokey-gtk xclip
    fi
    success "System packages installed."
fi

# --- Step 3: Install Python packages ---
info "Installing Python dependencies (anthropic, python-dotenv)..."
if python3 -c "import anthropic, dotenv" 2>/dev/null; then
    success "Python packages already installed."
else
    pip install --quiet --break-system-packages anthropic python-dotenv 2>/dev/null || \
    pip install --quiet anthropic python-dotenv || \
    pip3 install --quiet anthropic python-dotenv
    success "Python packages installed."
fi

# --- Step 4: Create .env file ---
info "Setting up API key config at $ENV_FILE..."
mkdir -p "$(dirname "$ENV_FILE")"
if [ -f "$ENV_FILE" ] && grep -q "ANTHROPIC_API_KEY" "$ENV_FILE" && ! grep -q "your_key_here" "$ENV_FILE"; then
    success "API key already configured."
else
    if [ ! -f "$ENV_FILE" ] || grep -q "your_key_here" "$ENV_FILE"; then
        echo "ANTHROPIC_API_KEY=your_key_here" > "$ENV_FILE"
        chmod 600 "$ENV_FILE"
        warn "Created $ENV_FILE — you must add your real Anthropic API key before use."
        warn "Edit the file: nano $ENV_FILE"
        warn "Replace 'your_key_here' with your key from https://console.anthropic.com"
    fi
fi

# --- Step 5: Copy Python scripts ---
info "Installing Claude scripts to AutoKey data directory..."
mkdir -p "$AUTOKEY_DIR"
cp "$SCRIPT_DIR"/_claude_lib.py    "$AUTOKEY_DIR/"
cp "$SCRIPT_DIR"/expand_email.py   "$AUTOKEY_DIR/"
cp "$SCRIPT_DIR"/expand_tldr.py    "$AUTOKEY_DIR/"
cp "$SCRIPT_DIR"/expand_reply.py   "$AUTOKEY_DIR/"
cp "$SCRIPT_DIR"/expand_fix.py     "$AUTOKEY_DIR/"
success "Scripts installed to $AUTOKEY_DIR"

# --- Step 6: Install AutoKey abbreviation configs ---
info "Registering AutoKey abbreviations (;;email, ;;tldr, ;;reply, ;;fix)..."
AUTOKEY_FOLDER="$HOME/.config/autokey/data/claude-snippets"
mkdir -p "$AUTOKEY_FOLDER"

# Copy folder metadata
cp "$SCRIPT_DIR/autokey-configs/.folder" "$AUTOKEY_FOLDER/.folder"

# Copy JSON configs alongside matching Python scripts
for trigger in email tldr reply fix; do
    cp "$SCRIPT_DIR/autokey-configs/expand_${trigger}.json" "$AUTOKEY_FOLDER/expand_${trigger}.json"
done
success "AutoKey abbreviations registered."

# --- Step 7: Restart AutoKey if running ---
info "Checking AutoKey service..."
if pgrep -x autokey-gtk &>/dev/null; then
    info "Restarting AutoKey to pick up new scripts..."
    pkill -x autokey-gtk 2>/dev/null || true
    sleep 1
    DISPLAY="${DISPLAY:-:1}" nohup autokey-gtk &>/dev/null &
    success "AutoKey restarted."
else
    info "AutoKey not running — it will load the scripts on next launch."
fi

# --- Done ---
echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "  Next steps:"
echo ""

if grep -q "your_key_here" "$ENV_FILE" 2>/dev/null; then
    echo -e "  ${RED}1. Add your Anthropic API key:${NC}"
    echo "     nano $ENV_FILE"
    echo "     Replace 'your_key_here' with your key from https://console.anthropic.com"
    echo ""
fi

echo "  2. Start AutoKey (if not running):"
echo "     autokey-gtk &"
echo ""
echo "  3. Test it — open any text editor and try:"
echo "     • Select some rough notes → type ;;email"
echo "     • Select a paragraph     → type ;;tldr"
echo "     • Select a received msg  → type ;;reply"
echo "     • Select text with typos → type ;;fix"
echo ""
echo "  See TUTORIAL.md for a full walkthrough and examples."
echo ""
