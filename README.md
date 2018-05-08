
# Semux Light

This is alpha stage, beta version comming soon.

---

![semux-light](https://raw.githubusercontent.com/witoldsz/semux-light/7019f3c572bca200ac18d8b8fd395f5fce93a340/semux-light-testnet-2018-05-02T10-42-44.png)

## Try it yourself:

- install [Node.js](https://nodejs.org/) (tested with 8.x LTS)
- enable Semux API
  ```
  #================
  # API
  #================

  # Be sure to set up authentication first before enabling API
  api.enabled = true

  # Listening address and port
  api.listenIp = 127.0.0.1
  api.listenPort = 5171

  # Basic authentication
  api.username = user
  api.password = 123456

  ```
- launch Semux (my advice: always use separate installation for testnet only)
  - semux-testnet/semux-cli.sh --network testnet
  - or
  - semux-testnet/semux-gui.sh --network testnet

- build Semux Light
  - frontend/GUI
    ```
    $ cd semux-testnet/front
    $ make dist
    ```
  - server
    ```
    $ cd ../server
    $ cat src/main/settings.ts
    export const settings = {
      semuxApiService: {
        address: 'http://127.0.0.1:5171/v2.0.0',
        user: 'user',
        pass: '123456',
      },
    }

    $ make deploy
    server listening: { address: '::', family: 'IPv6', port: 3333 }

    ```

- open browser location: http://localhost:3333/#/home?addr=SEM_ADDR1,SEM_ADDR2,…,n
