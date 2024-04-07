# KAITe

This is a simple TUI based frontend for Anthropic Claude-3 based LLMs like Haiku, Sonnet and Opus. It's a proof of concept with a silly name that I just made in my spare time.

### To use: 
1. Clone from github, and do a `cargo install --path .`
2. Create a `.kaite.env` file in your homespace which contains your API_KEY in the format `API_KEY=<YOUR_API_KEY>`.
3. Type `kaite` in a terminal to fire it up.

### Commands

Edit Mode:
- `i` Enter edit mode to type your prompt.
- `escape` Exits edit mode and enters normal mode.

Normal Mode:
- `j` Scrolls down.
- `k` Scrolls up.
- `m` Toggles the model to use, options are only Haiku, Sonnet and Opus.
- `q` Quits.
