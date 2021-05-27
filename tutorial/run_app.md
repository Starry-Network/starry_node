Starry as a whole is made up of three parts: Node, Query and App.

So to start Starry, you need to start all three of them.

### Node

You can run it after build or run in Docker. The repository is [here](https://github.com/Starry-Network/starry_node)

#### Build and Run

- Build

  ```
  cargo build --release
  ```

- Run

  ```
  ./target/release/node-template --dev --ws-external
  ```



#### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command (`cargo build --release && ./target/release/node-template --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/node-template --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/node-template purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```



### Query

Query is a starry index built using sub-query, which can be used to query the stored data using GraphQl after the startup is complete. The repository is [here](https://github.com/Starry-Network/starry_query)

To run Query you need Node >= 14.0 and Docker. Also start Starry Node before starting Query.

#### Initialize

```
yarn install
```

#### Configure

 before you can run the Query, you need to modify the endpoint in Project.xml.

```xml
network:
  endpoint: ws://172.17.0.1:9944
```

#### Code generation

```
yarn codegen
```

#### Build

```
yarn build
```

#### Run

```
docker-compose pull && docker-compose up
```

If you want to use GraphQL Playground, you can open http://localhost:3000



### APP

To run App you need Node >= 14.0.

#### Initialize

```
yarn build
```

#### Configure

Before run App, you need rename .env.example to .env and  change them for your environment.

- REACT_APP_CHAIN_ENDPOINT

  This is Starry Node endpoint. If it is running locally, just keep using localhost.

- REACT_APP_IPFS_HOST

  This is IPFS Host. If you don't want to use infura, just change it to something else.

- REACT_APP_IPFS_PORT

  This is IPFS Port

- REACT_APP_IPFS_PROTOCOL

  What protocol is used by the ipfs service, http or https.

- REACT_APP_QUERY_ENDPOINT

  This is Starry Query endpoint.  If it is running locally, just keep using localhost.

#### Run

```
yarn start
```

