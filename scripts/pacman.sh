#!/bin/sh

if [ "$STS_INIT" ]; then
    echo "^fg(gray)not checked^fg()"
else
    echo "^fg(yellow)fetching...^fg()"
    if count=$(checkupdates | wc -l); then
        if [ "$count" -eq 0 ]; then
            echo up to date!
        else
            echo "^fg(green)$count updates^fg()"
        fi
    else
        echo "^fg(red)failed^fg()"
    fi
fi
