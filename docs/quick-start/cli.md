# wg-quickrs command line interface

You can also use the command line interface to manage the config.
To get documentation, just pass in `--help` option.

The `installer.sh` script should also install the autocomplete scripts for bash or zsh.

```sh
# wg-quickrs <TAB>           # Shows available commands (agent, config)
wg-quickrs --help
# $ wg-quickrs
# A tool to manage the peer and network configuration of the WireGuard-based overlay network over the web console
# 
# Usage: wg-quickrs [OPTIONS] <COMMAND>
# 
# Commands:
#   agent   Run agent commands
#   config  Edit agent configuration options
#   help    Print this message or the help of the given subcommand(s)
# 
# Options:
#   -v, --verbose
#           Increase verbosity level from Info to Debug
#       --wg-quickrs-config-folder <WG_QUICKRS_CONFIG_FOLDER>
#           [default: /etc/wg-quickrs/]
#   -h, --help
#           Print help
#   -V, --version
#           Print version

# wg-quickrs agent <TAB>     # Shows available agent subcommands
wg-quickrs agent --help
# Run agent commands
# 
# Usage: wg-quickrs agent <COMMAND>
# 
# Commands:
#   init  Initialize the wg-quickrs agent.
#         Configuration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command
#   run   Run the wg-quickrs agent
#   help  Print this message or the help of the given subcommand(s)
# 
# Options:
#   -h, --help  Print help

# wg-quickrs config <TAB>    # Shows available config subcommands
wg-quickrs config --help
# Edit agent configuration options
# 
# Usage: wg-quickrs config <COMMAND>
# 
# Commands:
#   enable   Enable a configuration option
#   disable  Disable a configuration option
#   set      Set a configuration value
#   reset    Reset a configuration option
#   get      Get a configuration value
#   list     List network entities in human-readable format
#   remove   Remove network entities
#   add      Add network entities
#   help     Print this message or the help of the given subcommand(s)
# 
# Options:
#   -h, --help  Print help
```
