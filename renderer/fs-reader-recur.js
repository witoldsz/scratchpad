const fs = require('fs');
const promisify = require('./lib/promisify');
const serialMap = require('./lib/serial-map');
const FsTree = require('./fs-tree');

const pathUtils = require('path');
const join = (a, b) => a + pathUtils.sep + b;

const fsReaddir = path => promisify(cb => fs.readdir(path, cb));
const fsLstat = path => promisify(cb => fs.lstat(path, cb));
const fsReadlink = path => promisify(cb => fs.readlink(path, cb));

const EVENT_SYMBOL = Symbol('EVENT_SYMBOL');
const END_OF_EVENTS = Symbol('END_OF_EVENTS');

class FsReader {

  constructor(path) {
    this.tree = new FsTree(path);
    this.errors = [];
    this.rootPath = path;
  }

  _try(cb) {
    try { return cb(); } catch (err) { this.errors.push(err); }
  }

  scan() {
    return this._readDir(this.rootPath);
  }

  _readDir(path) {
    this.tree.addEntry(path, { type: 'D' });
    const names = this._try(() => fs.readdirSync(path));
    for (let i = 0; i < names.length; ++i) {
      this._readName(join(path, names[i]));
    }
  }

  _readName(path) {
    const stat = this._try(() => fs.lstatSync(path));
    if (stat.isDirectory()) return this._readDir(path);
    // if (stat.isSymbolicLink()) return this._readLink(path);
    if (stat.isFile()) return this.tree.addEntry(path, { type:'F', size: stat.size });
  }

  // TODO: links support here and in fs-tree.js
  // _readLink(path) {
  //   return fsReadlink(path, cb)
  //     .then(target => {
  //       this.addEntry(path, { type:'L', target })
  //     })
  //     ;
  // }
}

FsReader.EVENT_SYMBOL = EVENT_SYMBOL;
FsReader.END_OF_EVENTS = END_OF_EVENTS;
module.exports = FsReader;
