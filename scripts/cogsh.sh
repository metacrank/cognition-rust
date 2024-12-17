#!/bin/sh
term=alacritty
nohup $term -e $COGLIB_DIR/utils/cogsh.cog &>/dev/null &
