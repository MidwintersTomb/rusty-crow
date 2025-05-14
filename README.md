# rusty-crow
Remote control of Windows and Linux via IMAP.

cargo build --release

cargo build --release --target x86_64-pc-windows-gnu

USAGE:
    blackbird --username <username> --password <password> --time <time> --string <string>

OPTIONS:
    -h, --help                   Print help information
    -p, --password <password>    E-mail password
    -s, --string <string>        String identifier to check e-mail (not case sensitive)
    -t, --time <time>            Time interval to check email (in minutes)
    -u, --username <username>    E-mail username
    -V, --version                Print version information
