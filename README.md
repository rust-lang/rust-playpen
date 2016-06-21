A web interface for running Rust code built using [playpen][playpen]. It is
hosted at <https://play.rust-lang.org/>.

The interface can also be accessed in most Rust-related channels on
`irc.mozilla.org`.

To use Playbot in a public channel, address your message to it. Playbot
responds to both `playbot` and `>>`:

    <you> playbot: println!("Hello, World");
    -playbot:#rust-offtopic- Hello, World
    -playbot:#rust-offtopic- ()
    <you> >> 1+2+3
    -playbot:#rust-offtopic- 6

You can also private message Playbot your code to have it evaluated. In a
private message, don't preface the code with playbot's nickname:

    /msg playbot println!("Hello, World");

Playbot also understands several attributes to change how the code is compiled
and run. Examples include `~nightly` and `~beta` to change the release channel,
as well as `~mini` to execute the code as a standalone Rust file instead of
evaluating it as an expression. Example:

    <you> playbot, ~nightly ~mini #![feature(inclusive_range_syntax)] fn main() { println!("{:?}", 1...10) }

To see a list of attributes, use `~help`:

    <you> playbot ~help

# Running your own Rust-Playpen

## System Requirements

Rust-Playpen currently needs to be run on an Arch Linux system that meets
[playpen's requirements][playpen].

## IRC Bot Setup

#### Create `bitly_key`

The bot uses [bitly](https://bitly.com) as a URL shortener. Get an OAuth access token, and put it into a file called `bitly_key`, in the root directory of `rust-playpen`.

#### Modify `playbot.toml`

You'll also need to change the file `playbot.toml`. This configuration allows
the bot's nick to be registered, and can include the nick's password.

#### Registering and starting services

The working playpen has the IRC and Web services set up to automatically start at boot:

`/etc/systemd/system/rust-playpen-irc.service`

```
[Unit]
Description=Rust code evaluation sandbox (irc bots)
After=network.target

[Service]
ExecStart=/root/rust-playpen/bot.py

[Install]
WantedBy=multi-user.target
```

`/etc/systemd/system/rust-playpen-web.service`

```
[Unit]
Description=Rust code evaluation sandbox (web frontend)
After=network.target

[Service]
ExecStart=/root/rust-playpen/web.py

[Install]
WantedBy=multi-user.target
```

`/etc/systemd/system/rust-playpen-update.service`

```
[Unit]
Description=Playpen sandbox root updater

[Service]
Type=oneshot
ExecStart=/root/rust-playpen/init.sh
Environment=HOME=/root
```

`/etc/systemd/system/rust-playpen-update.timer`

```
[Unit]
Description=Playpen sandbox root update scheduler

[Timer]
OnBootSec=10min
OnCalendar=daily
Persistent=true
Unit=rust-playpen-update.service

[Install]
WantedBy=multi-user.target
```

If the services fail to start, kick them:

```
$ systemctl restart rust-playpen-irc.service
$ systemctl restart rust-playpen-web.service
```

[playpen]: https://github.com/thestinger/playpen
