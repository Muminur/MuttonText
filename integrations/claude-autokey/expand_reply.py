import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
import _claude_lib as lib

PROMPT = (
    "Draft a polite, professional reply to the following message. "
    "Output ONLY the reply body, no subject, no extra commentary."
)

def run():
    text = lib.get_selection()
    if not text:
        lib.notify("MuttonText", "No text selected.", urgency="critical")
        return ""
    lib.notify("MuttonText 💬", "Claude is drafting a reply...")
    result = lib.ask_claude(PROMPT, text)
    lib.notify("MuttonText 💬", "Done!")
    return result

if __name__ == "__main__":
    output = run()
    if output:
        keyboard.send_keys(output)  # noqa: F821
