#!/usr/bin/dash

set -o errexit

#if you pass --backtrace at all, ensure it's the first arg! (reason: simplified dash syntax and keeping the args-with-spaces whole)
if [ "$1" = "--backtrace" ]; then
    export RUST_BACKTRACE=1
    shift
    #^ this removes --backtrace from args
    #if you get "error: Unrecognized option: 'backtrace'." that's from 'rustc' below and it just means that the caller of this script(web.py) did not pass --backtrace as the first arg as it's required.
    #technically the caller of this script is 'playpen', 'web.py' is one step above.
fi

TERM=xterm rustc - -o ./out "$@"
printf '\377' # 255 in octal
if [ "${*#*--test}" != "$*" ] && [ "${*#*--color=always}" != "$*" ]; then
    # For /evaluate.json, we have {test: true, color: true}. Let's make the
    # output coloured too.  This would be better in web.py, but we don't
    # have an easy way to allot parameters for ./out.
    TERM=xterm exec ./out --color=always
else
    exec ./out
fi
