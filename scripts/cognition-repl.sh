#!/bin/sh
term=alacritty

nohup $term -e sh -c "$(printf 'exec crank -s 7 std/bootstrap.cog std/quote.cog std/prefix.cog std/namespace.cog std/fllib.cog examples/common-fllibs.cog utils/repl.cog %s' "$@")" &>/dev/null &
