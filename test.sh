#!/bin/env bash

# depends on `cat`, `tmux`
# convert escape sequences to human readable structure

SESSION_NAME=CUP
WINDOW_NAME="cat"
FIXTURE_PATH=/home/kenji/git/zellij-nix/zellij/fixture
INPUT_PATH="$1"


tmux new-session -d -s ${SESSION_NAME}
tmux new-window -d -t =${SESSION_NAME} -n ${WINDOW_NAME}
tmux send-keys -t =${SESSION_NAME}:=${WINDOW_NAME} "cat ${INPUT_PATH} & sleep 5000" Enter

sleep 3
tmux capture-pane -t =${SESSION_NAME}:=${WINDOW_NAME}
tmux save-buffer ${FIXTURE_PATH}

#cleanup
tmux kill-session -t =${SESSION_NAME}
