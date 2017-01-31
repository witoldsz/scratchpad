const WebSocketServer = require('ws').Server
const Reader = require('./ReaderBatch');

const wss = new WebSocketServer({ port: 3000 });

wss.on('connection', ws => {
  const readers = [];
  
  ws.on('close', () => {
    console.log('close');
    readers.forEach(reader => reader.shutdown());
  });
  
  ws.on('message', message => {
    const reader = new Reader();
    readers.push(reader);
    reader.onBatch(e => ws.send(JSON.stringify(e)));
    reader
      .scan(message)
      .then(() => console.log(`scan ${message} done`));
  });
  
});
