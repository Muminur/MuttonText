import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
import _claude_lib as lib

PROMPT = (
    "Summarize the following text into 2-4 concise bullet points. "
    "Output ONLY the bullet points, nothing else."
)

def run():
    text = lib.get_selection()
    if not text:
        lib.notify("MuttonText", "No text selected.", urgency="critical")
        return ""
    lib.notify("MuttonText 📝", "Claude is summarizing...")
    result = lib.ask_claude(PROMPT, text)
    lib.notify("MuttonText 📝", "Done!")
    return result

if __name__ == "__main__":
    output = run()
    if output:
        keyboard.send_keys(output)  # noqa: F821
