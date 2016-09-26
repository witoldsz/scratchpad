const koa = require('koa');
const serve = require('koa-static');
const websockify = require('koa-websocket');
const Reader = require('./Reader');

const app = websockify(koa());

app.use(serve('../gui'));
app.ws.use(function* (next) {
  this.websocket.on('message', (message) => {
    console.log(message);
    const reader = new Reader();
    reader.on('event', event => this.websocket.send(JSON.stringify(event)));
    reader._readdir(message); 
  }); 
  yield next;
});

app.listen(3000);