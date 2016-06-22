A web interface for running Rust code. It is hosted at
<https://play.rust-lang.org/>.

The interface can also be accessed in most Rust-related channels on
`irc.mozilla.org`.

To use Playbot in a public channel, address your message to it.

    <you> playbot: println!("Hello, World");
    -playbot:#rust-offtopic- Hello, World
    -playbot:#rust-offtopic- ()
    <you> playbot: 1+2+3
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

Currently needs to be run on a system with access to Docker.

## Running the web server

First, create the Docker images that playpen will use:

```
sh docker/build.sh
```

Next, spin up the server.

```
cargo run --bin playpen
```

You should now be able to browse http://127.0.0.1:8080 and interact.

## IRC Bot Setup

You'll need to move `playbot.toml.example` to `playbot.toml` and then configure
it appropriately.

# Setting up the play.rust-lang.org server

First off, start off with a fresh Ubuntu AMI. These should be listed on
https://cloud-images.ubuntu.com/locator/ec2/, and the current one is the Xenial
us-west-1 64-bit hvm ebs-ssd server, ami-08490c68.

* Launch an m3.medium instance
* Launch into EC2-Classic
* Protect against accidental termination
* Make the disk ~100GB
* Use the existing playground security group

SSH through the bastion, then:

```
sudo apt-get update
sudo apt-get install python-pip apt-transport-https ca-certificates
sudo pip install pygments

curl https://sh.rustup.rs | sh
git clone https://github.com/rust-lang/rust-playpen

# see https://docs.docker.com/engine/installation/linux/ubuntulinux/
sudo apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys 58118E89F3A912897C070ADBF76221572C52609D
echo 'deb https://apt.dockerproject.org/repo ubuntu-xenial main' | sudo tee /etc/apt/sources.list.d/docker.list
sudo apt-get update
sudo apt-get install linux-image-extra-$(uname -r) docker-engine
sudo service docker start
sudo usermod -aG docker ubuntu
```

Next, configure `playbot.toml` copied from `playbot.toml.example`.

Next, open up a screen window (`screen -R`), and spin up two sessions:

```
cargo run --release --bin playpen
```

```
cargo run --release --bin playbot
```
