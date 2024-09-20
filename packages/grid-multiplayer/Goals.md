# Grid Multiplayer class

Opens a websocket server that observers a grid, receive changes, validate them and send them.

- [x] Create a websocket connection that accept receive connections opening request with a given ID
- [ ] Create a grid on the storage if does not exist one yet with the received ID
- [ ] Get the grid from the storage if already exists some with the Given ID
- [ ] Serve the grid to the client
- [ ] Receive update requests from the client, apply on the grid, and change back the response