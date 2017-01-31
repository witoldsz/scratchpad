const fs = require('fs');
const join = require('path').join;
const promisify = require('./lib/promisify');
const serialMap = require('./lib/serial-map');
const FsTree = require('./fs-tree');

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

  scan() {
    return this._readDir(this.rootPath);
  }

  _readDir(path) {
    this.tree.addEntry(path, { type: 'D' });
    return fsReaddir(path)
      .then(serialMap(name => this._readName(join(path, name))))
      .catch(err => this.errors.push(err))
      ;
  }

  _readName(path) {
    return fsLstat(path)
      .then(stat => (
        stat.isDirectory() ? this._readDir(path) :
        // stat.isSymbolicLink() ? this._readLink(path) :
        stat.isFile() ? this.tree.addEntry(path, { type:'F', size: stat.size }) :
        undefined
      ))
      .catch(err => this.errors.push(err))
      ;
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
