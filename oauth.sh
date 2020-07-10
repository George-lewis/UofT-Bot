#!/bin/sh

CLIENT_ID="726858679550476289"
PERMISSIONS="0"

printf "https://discord.com/api/oauth2/authorize?client_id=${CLIENT_ID}&scope=bot&permissions=${PERMISSIONS}"