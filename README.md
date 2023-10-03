# Reaper

## Intro
Reaper will spawn the program that's given as an argument, and wait untill it dies.  
When that process dies, any left behind children of that process will be killed.  
This uses the `PR_SET_CHILD_SUBREAPER` argument for prctl.

## Usage

```./reaper /usr/bin/child_spawning_process```
