const Reader = require('./Reader');
const reader = new Reader();
reader.on('event', e => console.log(JSON.stringify(e)));
reader.on('error', console.error);
reader.scan(process.argv[2]);