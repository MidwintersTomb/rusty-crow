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

To compile on Linux:

`cargo build --release`

To compile on Windows:

`cargo build --release --target x86_64-pc-windows-gnu`

### Notes & Background:

- **Q:** Why "Rusty Crow"?

- **A:** I originally started this project in Rust before deciding to build a much more featured project in Go.  So, I moved the name over there, but kept the crow theme, and combined with it the fact it is written in Rust.

- **Q:** Will my anti-virus software yell at me for running this?

- **A:** In short - maybe.  In doing some testing, some flagged it, others didn't at all.  (Example: Avira hit it with a generic detection, but Defender couldn't care less.)

- **Q:** What's to keep this from being easily spotted?

- **A:** Would you think IMAP connecting to Gmail (or another mail provider) on a regular interval looks suspicious in network traffic?  Additionally, it will act as a command line mail client in that any e-mails that aren't *command* e-mails or replies to them will be written out to the command line in very basic fashion.  It kind of half assess the part.

- **Q:** Oh hey, this seems like I could use this for nefarious purposes... can I?

- **A:** Whatever you do with this is on you.  I will not be held responsible for anything dumb that you do.  If you do something dumb and get caught, tough luck, kid.
