# Rusty Crow
## Remote control of Windows and Linux via IMAP.

```
USAGE:
    blackbird --username <username> --password <password> --time <time> --string <string>

OPTIONS:
    -h, --help                   Print help information
    -p, --password <password>    E-mail password
    -s, --string <string>        String identifier to check e-mail (not case sensitive)
    -t, --time <time>            Time interval to check email (in minutes)
    -u, --username <username>    E-mail username
    -V, --version                Print version information
```

To compile for Linux on Linux:

`cargo build --release`

To compile for Windows on Linux:

`cargo build --release --target x86_64-pc-windows-gnu`

### Getting Started:

1. Follow this article for how to create an *App Password*: https://support.google.com/accounts/answer/185833?hl=en

2. ~Enable *IMAP* on your Gmail account~ *Update*: As of January 2025, Google has enabled IMAP for all accounts

3. Download the precompiled binaries or download the repo and compile it yourself.

### So how does this all work now?

If you've done everything correctly so far, and have Rusty Crow running on a device with your Gmail account and *App Password*, you're ready to get started.  The main thing to remember is the *identifier string* you picked.

Example:

If this was how you started up Rusty Crow via the BlackBird executable:

`./blackbird --username script.kiddie@gmail.com --password howtizwqjnyzgbcr --time 15 --string "library1475"`

Now, all you have to do is send a plain text e-mail to your Gmail account with a subject of "Command (Identifier)".

In the body of the e-mail you will include whatever commands you want to run.  **Note:** It is one command per line, blank lines are considered the end of command instructions, and anything below a blank line will be ignored.  (*Make sure your e-mail client is sending plain text for the body, or use shift+enter for each new line, as some mail clients will natively try to double space to look nice, or force a carriage return on word wrap.*)

Example:

Send e-mail to "script.kiddie@gmail.com" with a subject of "Command (library1475)".  The body of the e-mail is:

```
pwd
ls -la
cat /etc/passwd
```

Every 15 minutes the BlackBird client will check your Gmail account for new e-mail with commands.  When that interval triggers, it will execute the included commands and e-mail you the results including what command was sent, the response, and what it pulled from your original e-mail.

### Notes & Background:

- **Q:** What if I don't want to use Gmail and have my own IMAP server?

- **A:** Then it's up to you to configure things to use your server.  I've annotated the code with call outs for where to replace items for your own server.

- **Q:** What if I don't want to pass credential information on the command line?

- **A:** Wise decision.  I have annotated in the code the section to comment out to disable that configuration and the section to uncomment to hard code your credentials.  You will then need to compile the binary for either Windows or Linux.

- **Q:** Why "Rusty Crow"?

- **A:** I originally started this project in Rust before deciding to build a much more featured project in Go.  So, I moved the name over there, but kept the crow theme, and combined with it the fact it is written in Rust.

- **Q:** Will my anti-virus software yell at me for running this?

- **A:** In short - maybe.  In doing some testing, some flagged it, others didn't at all.  (Example: Avira hit it with a generic detection, but Defender couldn't care less.)

- **Q:** What's to keep this from being easily spotted?

- **A:** Would you think IMAP connecting to Gmail (or another mail provider) on a regular interval looks suspicious in network traffic?  Additionally, it will act as a command line mail client in that any e-mails that aren't *command* e-mails or replies to them will be written out to the command line in very basic fashion.  It kind of half assess the part.

- **Q:** Oh hey, this seems like I could use this for nefarious purposes... can I?

- **A:** Whatever you do with this is on you.  I will not be held responsible for anything dumb that you do.  If you do something dumb and get caught, tough luck, kid.
