# Problem Manager Bot

[![CI](https://github.com/nikgaevoy/problem-manager-bot/actions/workflows/ci.yml/badge.svg)](https://github.com/nikgaevoy/problem-manager-bot/actions/workflows/ci.yml)

A Telegram bot for collecting and managing competitive programming problems from group chats into a Google Spreadsheet.

## How it works

The bot watches a Telegram group for messages tagged with a configured hashtag (e.g. `#problem`). When it sees one, it parses the message as a problem submission and saves it to a local JSONL log file. A separate load step pushes the accumulated problems into a Google Sheets spreadsheet.

### Message format

```
#problem Legend describing the problem statement.
```

or, with a name:

```
#problem Problem Name
Legend describing the problem statement.
```

If the text after the hashtag contains a newline, the first line is the problem name and the rest is the legend. Otherwise the entire text is treated as the legend and the name is left blank.

## Commands

| Command | Where | Description |
|---------|-------|-------------|
| `/set_name <name>` | Any chat | Set your display name for problem attribution |
| `/help` | Any chat | Show available commands |
| `/set_difficulty <value>` | Group chat | Set difficulty for the target problem (reply > focus > last problem) |
| `/set_tags <value>` | Group chat | Set tags for the target problem (reply > focus > last problem) |
| `/editorial` (or `/solution`) | Group chat | Reply to the editorial message to attach it to the target problem; if the editorial message is itself a reply to the statement, that link is used, otherwise falls back to focus > last problem |
| `/focus_problem` | Group chat | Reply to a problem to make it the target for subsequent commands for 20 minutes; without a reply, clears the focus |
| `/clear_focus` | Group chat | Clear the focused problem, reverting to the last-problem default |
| `/load` | Group chat | Push pending problems to the spreadsheet |
| `/leave` | Group chat | Make the bot leave the chat |

The bot can also be run from the command line:

```sh
cargo run -- load   # push pending problems to the spreadsheet
cargo run -- scan <path>   # scan a Telegram Desktop HTML export for missed problems
```

### Recovering missed problems

The Telegram Bot API does not allow bots to retrieve past messages. To recover problems posted before the bot was added, or while it was offline:

1. Open Telegram Desktop → right-click the group → **Export chat history** → select **HTML** format
2. Run `cargo run -- scan <path> --chat-username mygroup` (public group) or `--chat-id 1234567890` (private group) to enable deduplication against already-saved problems

## Setup

### Prerequisites

- A Telegram bot token (from [@BotFather](https://t.me/BotFather))
- A Google Cloud service account with access to the Google Sheets API
- A Google Spreadsheet to write into

### Environment variables

Copy `.env.example` to `.env` and fill in the values:

| Variable | Description |
|----------|-------------|
| `TELOXIDE_TOKEN` | Telegram bot token |
| `HASHTAG` | Hashtag to watch for (e.g. `problem`) |
| `SPREADSHEET_ID` | Google Sheets spreadsheet ID |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to the GCP service account JSON key file |
| `SHEET_NAME` | Sheet tab name to append rows to (default: `Sheet1`) |
| `DATE_FORMAT` | [strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) format for the date column (default: `%Y-%m-%d %H:%M:%S`) |
| `PROBLEMS_FILE` | Path for the pending problems log (default: `problems.jsonl`) |
| `LOADED_PROBLEMS_FILE` | Path for the already-loaded problems log (default: `loaded_problems.jsonl`) |

### Running

```sh
# Listen for Telegram messages
cargo run -- listen

# Push pending problems to the spreadsheet
cargo run -- load
```
