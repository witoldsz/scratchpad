const Reader = require('./ReaderBatch');
const reader = new Reader();
// reader.onBatch(batch => batch.forEach(e => console.log(JSON.stringify(e))));
reader.onBatch(batch => {});
// reader.on('error', console.error);
reader.scan(process.argv[2]).then(console.log);