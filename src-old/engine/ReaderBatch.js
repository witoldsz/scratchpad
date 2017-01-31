const Reader = require('./Reader');

const EVENT_SYMBOL = Reader.EVENT_SYMBOL;
const END_OF_EVENTS = Reader.END_OF_EVENTS;
const MAX_BUFFER_LEN = 1;

class ReaderBatch {
  constructor() {
    this._reader = new Reader();
    this._reader.on('error', err => {
      if (err.code !== 'EACCES') console.log(err);
    });
  }

  scan(root) {
    return this._reader.scan(root);
  }

  onBatch(callback) {
    let buffer = [];
    this._reader.on(EVENT_SYMBOL, event => {
      
      const flushNow = 
        (  event === END_OF_EVENTS 
        || buffer.push(event) >= MAX_BUFFER_LEN
        );
      
      if (flushNow) {
        const b = buffer;
        buffer = [];
        callback(b);
      }
    });
  }

  shutdown() {
    console.log('READER SHUTDOWN')
    this._reader.removeAllListeners(EVENT_SYMBOL);
  }
}

module.exports = ReaderBatch;
