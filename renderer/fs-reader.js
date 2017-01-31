const fs = require('fs');
const FsTree = require('./fs-tree');
const pathUtils = require('path');
const join = (a, b) => a + pathUtils.sep + b;
const BATCH_DURATION = 150; //[ms]

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
    return new Promise((resolve, reject) => {
      const stack = [this.rootPath];
      const doContinue = () => {
        setTimeout(() => {
          this._scanBatch(stack);
          if (stack.length > 0) doContinue();
          else resolve();
        });
      };
      doContinue();
    });
  }

  _scanBatch(stack) {
    const t0 = Date.now();
    while ( Date.now() - t0 < BATCH_DURATION && stack.length > 0) {
      const path = stack.pop();
      const stat = this._try(() => fs.lstatSync(path));
      const type = (
        stat.isDirectory() ? 'D' :
          stat.isFile() ? 'F' :
            stat.isSymbolicLink() ? 'L' :
              '?');
      this.tree.addEntry(path, { type, size: stat.size });
      if (type === 'D') {
        this._try(() => fs.readdirSync(path).forEach(p => stack.push(join(path, p))));
      }
    }
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

module.exports = FsReader;
