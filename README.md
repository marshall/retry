## retry
---

Continually run any command until it succeeds (or fails!)


#### Examples

1. Retry a command up to 5 times until it's exit code is 0, sleeping longer between each run

        $ retry echo 'test'

1. Run a command indefinitely until it's exit code is non-0, sleeping 2 seconds between each
      run:

        $ retry -n 0 -s 2 -x ls /dev/ttyUSB1

   or, if you prefer GNU style long options:

        $ retry --max-tries=0 --sleep=2 --retry-on-success ls /dev/ttyUSB1

#### Usage

[//]: # (begin-usage)

    retry v0.1.0
    Usage: retry [options] [--] cmd [args..]

    Options:
        -h, --help          Print this help menu
        -m, --max-sleep n   Max sleep in seconds between retries when using
                            exponential backoff. default=3600 (1 hour)
        -n, --max-retries n Max number of retries. Set to 0 for unlimited retries.
                            default=5
        -q, --quiet         Don't log anything
        -s, --sleep n       Sleep n seconds between retries. Overrides default
                            exponential backoff.
        -v, --verbose       More verbose logging
        -V, --version       Print version information
        -x, --retry-on-success
                            Retry when 'cmd' has an exit code of 0

[//]: # (end-usage)

