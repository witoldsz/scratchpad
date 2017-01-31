const Readable = require('stream').Readable;
const fs = require('fs');
const join = require('path').join;
const promisify = require('./promisify');

const first = arr => arr[0];
const last = arr => arr[arr.length - 1];
const init = arr => arr.slice(0, arr.length - 1);
const tail = arr => arr.slice(1, arr.length);
const logError =
  err => {
    if (err.code !== 'EACCES') console.log(err);
  }

class MyReadable extends Readable {
  constructor(root) {
    super();
    // this._next = {type: 'D', name: root, path: root, pathArr: []};
    this._next = [[this._readDir, root, root, []]];
    this._nextDirId = 0;
    this._promise = Promise.resolve();
  }

  _read() {
    _continue();
  }

  _continue() {
    const next = this._next.shift();
    const fn = first(next);
    const args = tail(next);
    return fn.apply(this, args)
      .then(({ next, publish } = {}) => {
        if (next) {
          this._next = this._next.concat(next);
        }
        if (publish) {
          this._keepReading = this.push(JSON.stringify(publish) + '\n');
        }
        if (this._keepReading) {
          this._read();
        }
      })
      .catch(logError)
      ;
  }

  _readDir(name, path, pathArray) {
    const dirId = this._nextDirId++;
    const newPathArray = pathArray.concat(dirId);
    return promisify(cb => fs.readdir(path, cb))
      .then(names => (
        { next: names.map(n => [this._readName, n, join(path, n), newPathArray])
        , publish: ['D', name, dirId, pathArray]
        }))
      ;
  }

  _readName(name, path, pathArray) {
    return promisify(cb => fs.lstat(path, cb))
      .then(stat => (  
        stat.isDirectory()
          ? { next: [[this._readDir, name, path, pathArray]]
            , publish: null
            }
          // : stat.isSymbolicLink()
          // ? this._readLink(name, path, pathArray)
          : stat.isFile()
          ? { next: []
            , publish: ['F', name, last(pathArray), stat.size]
            }
          : { next: [], publish: null }
      ))
      ;
  }
}

const myReadable = new MyReadable(process.argv[2]);
myReadable.pipe(process.stdout)
myReadable.on('end', () => console.log('KONIEC'))