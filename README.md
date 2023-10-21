# Description

`open-urls` is a simple tool to open one or more web pages in browser
tabs. It reads the URL's (or URI's) to open from a configuration file,
but can also read the from standard input (stdin).

It provides a very thin wrapper over the [underlying crate](https://crates.io/crates/open).

## Why?

### Opening a URL on the command line

Opening a URL on the command line is fraught with problems. Although
simple sites can be handled easily enough:

```bash
$ firefox --new-tab "https://example.com"
```

Complications can arise if you try to open URLs like this:

```bash
$ firefox --new-tab "https://example.com/search?foo=bar&something=a+b+c&done=true&term=hello%20world"
```

Why? Because the URLs can contain shell meta-characters such as `?`,
`&`.

It can get even more confusing if you want to expand a variable in a
URL as you would need to expand that, but not other parts of the URL:

```bash
$ firefox --new-tab "https://example.com/search?foo=$SEARCH_TERM&something=a+b+c&done=$done&term=hello%20world"
```

Hence, opening a URL from a shell can easily cause unintended
side-effects and corrupt the URL, meaning you cannot open it ;)

### Opening a URL in a crontab

It should be simple right? Not always. Will the following work as
expected?

```crontab
DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/1000/bus"
DISPLAY=:0.0

# minute hour  monthday month weekday             command
# 0-59   0-23  1-31     1-12  0-7 (0 or 7=Sunday) ...
0 8 * * * firefox --new-tab 'https://example.com/search?foo=bar&something=a+b+c&done=true&term=hello%20world'
```

**Answer:** no! Not only do you need to worry about shell meta
characters, you also need to worry about the percent character (`%`)
since `cron(8)` will convert percentage symbols to newlines and send everything
that follows to the standard input of the command!

## Install

- [Install rust](https://rustup.rs)

- Ensure the Cargo bin directory is in your `$PATH`:

  ```bash
  $ export PATH=$PATH:$HOME/.cargo/bin
  ```
- Checkout the code

  ```bash
  $ git clone https://github.com/jamesodhunt/open-urls
  $ cd open-urls
  ```

- Build and install:

  ```bash
  $ make
  ```

  Or,

  ```bash
  $ cargo install --path .
  ```

## Usage

### Minimal example

```
$ open-urls -c urls.conf
```

### Command-line options

There are a few available options.

Run the following for details:

```
$ open-urls --help
```

## Config file

The config file format:

- Assumes one URL / line.
- Uses `#` for comment lines that are ignored.
- Permits blank lines.
- Supports two fields, separated by a semi-colon (`;`) character:
  - The first field is the URL to open.
  - The second field is an optional browser (binary) name (which must
    be in your `$PATH`).

### Config example

```conf
# This line is a comment.

# The URL on the line below will, by default, be opened with your system's default browser
https://github.com

# However, the URL on the line below will, by default, be opened with the specified browser.
https://gitlab.com;firefox
```

## Limitations

You cannot open a URL and have the _browser_ background it. What
this means is that if you are viewing a tab and then you open a URL
using this tool with the same browser, the browser will -- annoyingly
-- switch to the newly opened tab. 

This is a limitation of the [underlying crate](https://crates.io/crates/open).
