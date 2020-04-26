#!/bin/sh

if [ "$STS_INIT" ]; then
    echo "^fg(gray)dunno^fg()"
else
    echo "^fg(yellow)fetching...^fg()"
    if count=$(checkupdates | wc -l); then
        if [ "$count" -eq 0 ]; then
            echo "^fg(green)Updated^fg()"
        else
            echo "^fg(yellow)$count updates^fg()"
        fi
    else
        echo "^fg(red)failed^fg()"
    fi
fi
