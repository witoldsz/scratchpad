const fs = require('fs');
const join = require('path').join;
const EventEmitter = require('events');
const promisify = require('./lib/promisify');
const promiseMap = require('./lib/promiseMap');

const last = arr => arr[arr.length - 1];

const EVENT_SYMBOL = Symbol('EVENT_SYMBOL');
const END_OF_EVENTS = Symbol('END_OF_EVENTS');

class Reader extends EventEmitter {

  constructor() {
    super();
    this._dirNames = [];
  }

  scan(root) {
    return this._readDir(root, root, [])
      .then(() => this._emit(END_OF_EVENTS));
  }

  _emit(event) {
    this.emit(EVENT_SYMBOL, event);
  }

  _readDir(name, path, pathArray) {
    if (this.listenerCount(EVENT_SYMBOL) < 1) return Promise.resolve();
    const dir_id = this._dirNames.push(name) - 1;
    const newPathArray = pathArray.concat(dir_id);
    this._emit(['D', name, pathArray, dir_id]);
    return promisify(cb => fs.readdir(path, cb))
      .then(promiseMap(name => this._readName(name, join(path, name), newPathArray)))
      .catch(err => this.emit('error', err))
      ;
  }

  _readName(name, path, pathArray) {
    return promisify(cb => fs.lstat(path, cb))
      .then(stat => (
        stat.isDirectory() ? this._readDir(name, path, pathArray) : 
        stat.isSymbolicLink() ? this._readLink(name, path, pathArray) : 
        stat.isFile() ? this._emit(['F', name, last(pathArray), stat.size]) : 
        undefined
      ))
      .catch(err => this.emit('error', err))
      ;
  }

  _readLink(name, path, pathArray) {
    return promisify(cb => fs.readlink(path, cb))
      .then(target => {
        this._emit(['L', name, last(pathArray), target])
      })
      ;
  }
}

Reader.EVENT_SYMBOL = EVENT_SYMBOL;
Reader.END_OF_EVENTS = END_OF_EVENTS;
module.exports = Reader;
