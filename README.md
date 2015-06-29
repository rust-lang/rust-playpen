A web interface for running Rust code built using [playpen][playpen]. It is
hosted at <http://play.rust-lang.org/>.

The interface can also be accessed in most Rust-related channels on
`irc.mozilla.org`.

To use Playbot in a public channel, address your message to it. Playbot
responds to both `playbot` and `rusti`: 

    <you> playbot: println!("Hello, World");
    -playbot:#rust-offtopic- Hello, World
    -playbot:#rust-offtopic- ()

You can also private message Playbot your code to have it evaluated. In a
private message, don't preface the code with playbot's nickname: 

    /msg playbot println!("Hello, World");

# Running your own Rust-Playpen

## System Requirements

Rust-Playpen currently needs to be run on an Arch Linux system that meets
[playpen's requirements][playpen]. 

The bot requires python 3, which is the default on Arch.

## IRC Bot Setup 

`playbot` on Mozilla IRC is run from a Rust-Playpen instance where Python
dependencies are installed system-wide. Get the latest versions of `pyyaml`,
`requests`, and `irc` from Pip. 

#### Create `shorten_key.py`

The bot uses [bitly](http://bitly.com) as a URL shortener. Get an OAuth access token, and put it
into a file called `shorten_key.py`, in the same directory as `bot.py`.
`shorten_key.py` just needs one line, of the form:

    key = "123abc123"

#### Create `irc.yaml`

You'll also need to create the file `irc.yaml` in the same directory as
`bot.py`. The IRC nick that the bot will use is hard-coded [in
bot.py][nickname]. This configuration assumes that the bot's nick is
registered, and includes the nick's password. The `irc.yaml` file will look
something like this:

```
   - server: irc.example.org
      port: 6667
      channels:
        - "#bots"
        - "#secret-clubhouse"
      keys: [null, "hunter2"]
      password: abc123abc
``` 

Note that the channel key is `null` for public channels. 


[playpen]: https://github.com/thestinger/playpen
[nickname]: https://github.com/rust-lang/rust-playpen/blob/master/bot.py#L140

