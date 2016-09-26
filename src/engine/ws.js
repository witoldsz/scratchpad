const WebSocketServer = require('ws').Server
const Reader = require('./Reader');

const wss = new WebSocketServer({ port: 3000 });

wss.on('connection', ws => {
  ws.on('message', message => {
    console.log('received: %s', message);
    const reader = new Reader();
    reader.on('events', e => ws.send(JSON.stringify(e)));
    reader
      .readdir(message)
      .then(() => console.log(reader));
  });
});