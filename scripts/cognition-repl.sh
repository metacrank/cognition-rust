#!/bin/sh
term=alacritty

nohup $term -e sh -c "$(printf 'exec crank -s 6 stdbootstrap.cog stdquote.cog stdprefix.cog stdnamespace.cog common-fllibs.cog repl.cog %s' "$@")" &>/dev/null &
