#!/bin/sh
term=alacritty

nohup $term -e sh -c "$(printf 'exec crank -s 6 std/bootstrap.cog std/quote.cog std/prefix.cog std/namespace.cog examples/common-fllibs.cog utils/repl.cog %s' "$@")" &>/dev/null &
