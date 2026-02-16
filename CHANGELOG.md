Change Logs

## Release v0.0.3

### Better-On-Boarding:

Removed the way of manually add config files,
rather now on the fist time of start it asks for
ip and port for the server, then it saves it in a
config file at $HOME/.sentinel/config.toml

and also refactored the code.

## Release v0.0.21

### Removed 3 way handshake:

before it used ip from config and matches it with the incoming request.

but getting ips of all students computer is not possible for labs like os,
there ip changes due to dhcp server.

nothing to change in client, tested and connected with client
