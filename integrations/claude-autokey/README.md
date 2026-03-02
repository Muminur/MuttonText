# Claude AutoKey Integration for MuttonText

Dynamic Claude-powered text expansion alongside MuttonText on Ubuntu/Linux.

Select any text, type a `;;trigger`, and Claude expands it in-place — no copy-paste, no context switching.

| Trigger    | What it does                              |
|------------|-------------------------------------------|
| `;;email`  | Turns rough notes into a polished email   |
| `;;tldr`   | Summarizes selected text into bullets     |
| `;;reply`  | Drafts a professional reply               |
| `;;fix`    | Fixes grammar and spelling in-place       |

## Quick Start

```bash
git clone https://github.com/Muminur/MuttonText.git
cd MuttonText
bash integrations/claude-autokey/install.sh
```

Then add your Anthropic API key:
```bash
nano ~/.config/muttontext/.env
# Replace 'your_key_here' with your key from https://console.anthropic.com
```

Start AutoKey: `autokey-gtk &`

Select some text → type `;;email` → done.

## Full Tutorial

See **[TUTORIAL.md](TUTORIAL.md)** for:
- Step-by-step setup with examples
- Demos of all four triggers
- How to add your own custom triggers
- Troubleshooting guide

## Requirements

- Ubuntu/Linux with X11
- AutoKey: `sudo apt install autokey-gtk`
- xclip: `sudo apt install xclip`
- Python: `pip install anthropic python-dotenv`
- [Anthropic API key](https://console.anthropic.com)

## Files

| File | Purpose |
|------|---------|
| `install.sh` | One-command automated setup |
| `_claude_lib.py` | Shared library (API, xclip, notifications) |
| `expand_email.py` | `;;email` trigger script |
| `expand_tldr.py` | `;;tldr` trigger script |
| `expand_reply.py` | `;;reply` trigger script |
| `expand_fix.py` | `;;fix` trigger script |
| `autokey-configs/` | AutoKey abbreviation metadata (installed automatically) |

## Security

API key stored at `~/.config/muttontext/.env` with `600` permissions (your account only). Selected text is sent to the Anthropic API — see [Anthropic's privacy policy](https://www.anthropic.com/legal/privacy).
