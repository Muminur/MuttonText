import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
import _claude_lib as lib

PROMPT = (
    "Fix all grammar, spelling, and punctuation errors in the following text. "
    "Preserve the original tone and meaning exactly. "
    "Output ONLY the corrected text, nothing else."
)

def run():
    text = lib.get_selection()
    if not text:
        lib.notify("MuttonText", "No text selected.", urgency="critical")
        return ""
    lib.notify("MuttonText ✏️", "Claude is fixing your text...")
    result = lib.ask_claude(PROMPT, text)
    lib.notify("MuttonText ✏️", "Done!")
    return result

if __name__ == "__main__":
    output = run()
    if output:
        keyboard.send_keys(output)  # noqa: F821
