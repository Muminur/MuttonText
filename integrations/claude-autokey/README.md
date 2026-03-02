# Claude AutoKey Integration for MuttonText (Ubuntu/Linux)

Dynamic Claude-powered text expansion alongside MuttonText static snippets.

## What it does

Select any text, type a `;;trigger`, and Claude expands it:

| Trigger    | What it does                              |
|------------|-------------------------------------------|
| `;;email`  | Turns rough notes into a polished email   |
| `;;tldr`   | Summarizes selected text into bullets     |
| `;;reply`  | Drafts a professional reply               |
| `;;fix`    | Fixes grammar and spelling in-place       |

## Requirements

- Ubuntu/Linux with X11
- [AutoKey](https://github.com/autokey/autokey): `sudo apt install autokey-gtk`
- `xclip`: `sudo apt install xclip`
- Python: `pip install anthropic python-dotenv`
- Anthropic API key

## Setup

1. **Add your API key:**
   ```bash
   echo "ANTHROPIC_API_KEY=your_key_here" > ~/.config/muttontext/.env
   chmod 600 ~/.config/muttontext/.env
   ```

2. **Copy scripts to AutoKey:**
   ```bash
   mkdir -p ~/.config/autokey/data/claude-snippets
   cp integrations/claude-autokey/*.py ~/.config/autokey/data/claude-snippets/
   ```

3. **Register in AutoKey:**
   - Open AutoKey (`autokey-gtk`)
   - New Folder → `Claude Snippets`
   - For each trigger: New Script → set Abbreviation → paste script content → Save

   | Script | Abbreviation |
   |--------|-------------|
   | expand_email.py | `;;email` |
   | expand_tldr.py  | `;;tldr`  |
   | expand_reply.py | `;;reply` |
   | expand_fix.py   | `;;fix`   |

4. **Test:** Select text → type `;;email` → watch Claude expand it.

## How it works

```
User selects text → types ;;trigger → AutoKey fires Python script
    → xclip grabs X11 primary selection
    → notify-send "Claude thinking..."
    → Anthropic API call (claude-haiku-4-5-20251001)
    → AutoKey types Claude response in place
```

## API Key security

The API key lives at `~/.config/muttontext/.env` with `600` permissions — readable only by you.
