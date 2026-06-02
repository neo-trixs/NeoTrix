#!/bin/bash
cp /Users/neo/Downloads/code/neotrix/deploy/com.neotrix.proxy-daemon.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.neotrix.proxy-daemon.plist
launchctl list | grep com.neotrix.proxy-daemon
