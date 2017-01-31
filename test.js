const FsReader = require('./renderer/fs-reader.js');
const fsr = new FsReader(process.argv[2]);
fsr.scan().then(() => console.log(fsr.tree._root.size));
