# redis-populator
Tool for populating a Redis database from a remote markdown table.

To configure from where to fetch the Markdown file, which column of the Markdown table to use as the key and the destination Redis.

A working usage example is in the docker-compose.yaml, but we can also run it as is with Docker:

```bash
$ docker run -e CONFIG_PATH=/config -v $(pwd)/config:/config:ro clopezgarcia/redis-populator:latest
```

With a config.yaml with the following structure:

```yaml
markdown:
  url: <Markdown URL>
  key: <Table key> # Will be used as Redis key

redis:
  base_url: <Redis URL>
  username: Optional<Redis Username> # If username is set, password must be set too
  password: Optional<Redis Password> # If password is set, username must be set too
```


## Future

Future improvements (not in order):

- Fetching auth.
- Redis auth.
- Local files implementation.
- Parsing of more file formats.