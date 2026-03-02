# Tutorial: Claude-Powered Dynamic Text Expansion with MuttonText

This tutorial walks you through setting up Claude AI as a dynamic text expander alongside MuttonText on Ubuntu/Linux. When you're done, selecting text and typing `;;email` will instantly transform your rough notes into a polished email using Claude.

## What You'll Build

```
Your rough notes → ;;email → Claude API → Polished email, typed in-place
```

Four triggers, each powered by Claude:

| Trigger    | Input                          | Output                              |
|------------|--------------------------------|-------------------------------------|
| `;;email`  | Bullet point notes             | Full professional email body        |
| `;;tldr`   | Long article or document       | 2–4 concise bullet point summary    |
| `;;reply`  | Message or email you received  | Polite, professional reply draft    |
| `;;fix`    | Text with typos/grammar errors | Corrected text, same tone/meaning   |

---

## Prerequisites

Before starting, make sure you have:

- **Ubuntu 20.04+** (or any Debian-based Linux) with **X11** display server
- **MuttonText** installed — [download here](https://github.com/Muminur/MuttonText/releases/latest)
- **Python 3.8+** (check: `python3 --version`)
- An **Anthropic API key** — [get one here](https://console.anthropic.com) (free tier available)

> **Note:** This integration requires X11. Wayland support is not yet available due to xclip limitations.

---

## Step 1: Get Your Anthropic API Key

1. Go to [console.anthropic.com](https://console.anthropic.com)
2. Sign up or log in
3. Navigate to **API Keys** → **Create Key**
4. Copy the key (starts with `sk-ant-...`) — you'll need it in Step 3

---

## Step 2: Clone and Run the Installer

Open a terminal and run:

```bash
# Clone the MuttonText repo (if you haven't already)
git clone https://github.com/Muminur/MuttonText.git
cd MuttonText

# Run the installer
bash integrations/claude-autokey/install.sh
```

The installer will:
- Install `autokey-gtk` and `xclip` (system packages)
- Install `anthropic` and `python-dotenv` (Python packages)
- Copy the Claude scripts to AutoKey's data directory
- Register the `;;email`, `;;tldr`, `;;reply`, `;;fix` abbreviations automatically
- Create `~/.config/muttontext/.env` for your API key

---

## Step 3: Add Your API Key

The installer creates a config file at `~/.config/muttontext/.env`. Open it and add your key:

```bash
nano ~/.config/muttontext/.env
```

Replace `your_key_here` with your actual API key:

```
ANTHROPIC_API_KEY=sk-ant-api03-xxxxxxxxxxxxxxxx
```

Save and exit (`Ctrl+O`, `Enter`, `Ctrl+X`).

> **Security:** This file has `600` permissions — only your user account can read it. Never share or commit this file.

---

## Step 4: Start AutoKey

AutoKey is the trigger engine that detects your `;;keywords` and fires the Claude scripts.

```bash
autokey-gtk &
```

You should see the AutoKey icon appear in your system tray (top-right of your desktop). AutoKey will automatically start with your system on next login.

---

## Step 5: Your First Expansion — `;;email`

Let's test the email trigger end to end.

1. **Open a text editor** — gedit, VS Code, Mousepad, or any app where you can type

2. **Type some rough notes**, for example:
   ```
   meeting with sarah tomorrow 3pm
   discuss q1 budget
   need approval for new hire
   bring laptop
   ```

3. **Select all that text** (Ctrl+A or click and drag)

4. **Type `;;email`** (don't press Enter — just type the trigger)

5. **Watch what happens:**
   - A notification appears: "Claude is writing your email..."
   - The trigger disappears
   - A polished email appears in its place, something like:

   ```
   Dear Sarah,

   I hope this message finds you well. I wanted to confirm our meeting scheduled
   for tomorrow at 3:00 PM.

   During our session, I'd like to discuss the Q1 budget and seek your approval
   for a new hire. Please ensure you bring your laptop as we may need to review
   some documents together.

   Looking forward to our conversation.

   Best regards,
   [Your name]
   ```

> **Tip:** The expansion takes 1–3 seconds while Claude generates the response. A notification lets you know it's working.

---

## Step 6: Try All Four Triggers

### `;;tldr` — Summarize anything

1. Copy a long article, email, or document
2. Paste it into a text editor
3. Select it all
4. Type `;;tldr`

**Example input:**
```
The quarterly earnings report showed strong performance across all divisions.
Revenue increased by 23% year-over-year, driven primarily by the new product
line launched in Q3. Customer acquisition costs dropped by 15% following the
marketing strategy overhaul. However, operational expenses increased by 8%
due to infrastructure investments. The board approved a dividend increase
of $0.05 per share...
```

**Example output:**
```
• Revenue up 23% YoY, driven by Q3 product launch
• Customer acquisition costs down 15% after marketing strategy changes
• Operational expenses up 8% due to infrastructure investment
• Board approved $0.05/share dividend increase
```

---

### `;;reply` — Draft a professional reply

1. Select an email or message you received
2. Type `;;reply`

**Example input (selected text):**
```
Hi, I wanted to follow up on the project proposal I sent last week. Have you
had a chance to review it? We're hoping to kick off by end of month.
```

**Example output:**
```
Thank you for following up on the project proposal. I apologize for the delay
in getting back to you — I've had a chance to review it and find it very
promising.

I'd like to schedule a call this week to discuss a few questions before we
finalize our decision. Would Thursday or Friday work for you?

We're equally eager to move forward and appreciate your patience.

Best regards
```

---

### `;;fix` — Fix grammar and spelling

1. Type something with intentional (or accidental) errors
2. Select it
3. Type `;;fix`

**Example input:**
```
helo, i am writting to inqure about the posibilty of sceduling a meating
next weak to dicuss the propsal you sented last mnth.
```

**Example output:**
```
Hello, I am writing to inquire about the possibility of scheduling a meeting
next week to discuss the proposal you sent last month.
```

---

## Step 7: Error Handling

The integration handles common errors gracefully:

| Situation | What happens |
|-----------|-------------|
| Nothing selected | Notification: "No text selected." — nothing is typed |
| No API key set | Notification: "Error" — check `~/.config/muttontext/.env` |
| API rate limit hit | Notification: "Error" — try again in a moment |
| xclip not installed | Notification: "No text selected." — run `sudo apt install xclip` |

---

## Step 8: Add Your Own Custom Triggers

You can add custom Claude triggers by creating new scripts.

**Example: `;;formal` — rewrite text in a formal tone**

1. Create `~/.config/autokey/data/claude-snippets/expand_formal.py`:

```python
import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
import _claude_lib as lib

PROMPT = (
    "Rewrite the following text in a formal, professional tone. "
    "Preserve the meaning exactly. Output ONLY the rewritten text."
)

def run():
    text = lib.get_selection()
    if not text:
        lib.notify("MuttonText", "No text selected.", urgency="critical")
        return ""
    lib.notify("MuttonText 🎩", "Making it formal...")
    result = lib.ask_claude(PROMPT, text)
    lib.notify("MuttonText 🎩", "Done!")
    return result

if __name__ == "__main__":
    output = run()
    if output:
        keyboard.send_keys(output)  # noqa: F821
```

2. Create `~/.config/autokey/data/claude-snippets/expand_formal.json`:

```json
{
    "type": "script",
    "description": "Claude Formal — rewrite selected text in formal tone",
    "abbreviation": {
        "abbreviations": [";;formal"],
        "backspace": true,
        "ignoreCase": false,
        "immediate": false,
        "triggerInside": false,
        "wordChars": "[\\w]"
    },
    "hotkey": { "modifiers": [], "hotKey": null },
    "modes": [3],
    "usageCount": 0,
    "sendMode": "kb",
    "showInTrayMenu": false,
    "prompt": false,
    "store": {}
}
```

3. Restart AutoKey: `pkill autokey-gtk; autokey-gtk &`

4. Done — select text, type `;;formal`, Claude rewrites it.

---

## Troubleshooting

### Trigger types but nothing happens

- Check AutoKey is running: look for its icon in the system tray
- Verify scripts are in place: `ls ~/.config/autokey/data/claude-snippets/`
- Check AutoKey logs: **AutoKey → View → Error Log**

### "No text selected" notification even when text is selected

- This integration uses X11 primary selection (highlight = selected)
- Make sure you **highlight** text with mouse or keyboard before typing the trigger
- xclip reads the primary selection — confirming `xclip -selection primary -o` returns your text

### Claude returns an error / notification says Error

1. Check your API key is correct:
   ```bash
   cat ~/.config/muttontext/.env
   ```
2. Test the API directly:
   ```bash
   python3 -c "
   from dotenv import load_dotenv
   import os, anthropic
   load_dotenv(os.path.expanduser('~/.config/muttontext/.env'))
   client = anthropic.Anthropic()
   msg = client.messages.create(model='claude-haiku-4-5-20251001', max_tokens=50, messages=[{'role':'user','content':'say hi'}])
   print(msg.content[0].text)
   "
   ```
3. If you get an `AuthenticationError`, update your API key in `~/.config/muttontext/.env`

### AutoKey doesn't start on login

```bash
# AutoKey adds itself to startup automatically — verify:
ls ~/.config/autostart/autokey*.desktop
# If missing, create it:
cp /usr/share/applications/autokey-gtk.desktop ~/.config/autostart/
```

---

## How It Works (Technical)

```
User selects text (X11 primary selection)
    ↓
User types ;;email
    ↓
AutoKey detects ";;email" abbreviation
    ↓
AutoKey backspaces the trigger characters
    ↓
AutoKey runs expand_email.py
    ↓
expand_email.py calls get_selection() → xclip reads primary selection
    ↓
notify-send shows "Claude is writing your email..."
    ↓
Anthropic API call: claude-haiku-4-5-20251001
    ↓
Response text returned
    ↓
notify-send shows "Done!"
    ↓
AutoKey types the response via keyboard.send_keys()
```

**Model used:** `claude-haiku-4-5-20251001` — fast (1–3 seconds), low cost (~$0.001 per expansion)

**Privacy:** Your selected text is sent to the Anthropic API to generate a response. No data is stored locally beyond your API key in `~/.config/muttontext/.env`.

---

## Files Reference

| File | Location | Purpose |
|------|----------|---------|
| `_claude_lib.py` | `~/.config/autokey/data/claude-snippets/` | Shared library: API calls, xclip, notifications |
| `expand_*.py` | `~/.config/autokey/data/claude-snippets/` | One file per trigger |
| `expand_*.json` | `~/.config/autokey/data/claude-snippets/` | AutoKey abbreviation config |
| `.env` | `~/.config/muttontext/` | Your Anthropic API key (chmod 600) |

---

## Getting Help

- **Issues:** [github.com/Muminur/MuttonText/issues](https://github.com/Muminur/MuttonText/issues)
- **AutoKey docs:** [github.com/autokey/autokey/wiki](https://github.com/autokey/autokey/wiki)
- **Anthropic API docs:** [docs.anthropic.com](https://docs.anthropic.com)
