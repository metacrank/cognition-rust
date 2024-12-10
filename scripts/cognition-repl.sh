#!/bin/sh
term=alacritty

nohup $term -e sh -c "$(printf 'exec crank -s 4 stdbootstrap.cog stdquote.cog common-fllibs.cog repl.cog %s' "$@")" &>/dev/null &
