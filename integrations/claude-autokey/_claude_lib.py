import os
import subprocess
from dotenv import load_dotenv
import anthropic

load_dotenv(os.path.expanduser("~/.config/muttontext/.env"))

def _get_client():
    return anthropic.Anthropic(api_key=os.getenv("ANTHROPIC_API_KEY"))

def get_selection() -> str:
    try:
        result = subprocess.check_output(
            ["xclip", "-selection", "primary", "-o"],
            stderr=subprocess.DEVNULL
        )
        return result.decode("utf-8", errors="replace").strip()
    except subprocess.CalledProcessError:
        return ""

def notify(title: str, message: str = "", urgency: str = "normal"):
    subprocess.Popen(
        ["notify-send", "-u", urgency, title, message],
        stderr=subprocess.DEVNULL
    )

def ask_claude(system_prompt: str, user_text: str) -> str:
    client = _get_client()
    msg = client.messages.create(
        model="claude-haiku-4-5-20251001",
        max_tokens=1024,
        system=system_prompt,
        messages=[{"role": "user", "content": user_text}]
    )
    return msg.content[0].text
