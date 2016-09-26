const fs = require('fs');
const join = require('path').join;
const EventEmitter = require('events');
const promisify = require('./promisify');

const last = arr => arr[arr.length - 1];

class Reader extends EventEmitter {

  constructor() {
    super();
    this._dirNames = [];
  }

  scan(root) {
    this.emit('event', {type: 'ROOT', name: root});
    return this._readDir(root, root, []);
  }

  _readDir(name, path, pathArray) {
    const dir_id = this._dirNames.push(name) - 1;
    this.emit('event', {type: 'D', name, parent: pathArray, dir_id});
    const newPathArray = pathArray.concat(dir_id);
    return promisify(cb => fs.readdir(path, cb))
      .then(names => Promise.all(
        names.map(name => this._readName(name, join(path, name), pathArray))
      ))
      .catch(err => this.emit('error', err))
      ;
  }

  _readName(name, path, pathArray) {
    return promisify(cb => fs.lstat(path, cb))
      .then(stat => {
        return stat.isDirectory()
          ? this._readDir(name, path, pathArray)
          : stat.isSymbolicLink()
          ? this._readLink(name, path, pathArray)
          : stat.isFile()
          ? this.emit('event', {type: 'F', name, dir_id: last(pathArray), size: stat.size})
          : undefined
      })
      .catch(err => this.emit('error', err))
      ;
  }

  _readLink(name, path, pathArray) {
    return promisify(cb => fs.readlink(path, cb))
      .then(target => {
        this.emit('event', {type: 'L', name, dir_id: last(pathArray), target})
      })
      ;
  }
}

module.exports = Reader;
