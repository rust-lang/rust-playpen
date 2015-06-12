#!/usr/bin/dash

set -o errexit

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
