const {sep} = require('path');
const assert = require('assert');

const newDir = (path) => ({ path, size: 0, files: [], dirs: {} });
const newFile = (path, size) => ({ path, size })
const viewItem = (type, { path, size }) => ({ path, size, type });

class FsTree {

  constructor(path, pathSep = sep) {
    this._root = newDir(path);
    this._pathSep = sep;
  }

  viewRoot() {
    return [viewItem('D', this._root)].concat(this._viewNode(this._root));
  }

  viewPath(path) {
    const names = this._pathNames(path);
    const node = names.reduce((dir, name) => dir.dirs[name], this._root);
    return this._viewNode(node);
  }

  _viewNode(node) {
    const result = [];
    if (node) {
      for (let i in node.dirs) result.push(viewItem('D', node.dirs[i]));
      for (let i in node.files) result.push(viewItem('F', node.files[i]));
    }
    return result;
  }

  _pathNames(path) {
    const rootPath = this._root.path; // root=/home/user path=/home/user/files/file.txt
    const sliced = path.slice(rootPath.length + 1); // files/file.txt
    assert(sliced.length > 0 && path.startsWith(rootPath + '/'),
      `path (${path}) is not a child of root path (${rootPath})`);
    return sliced.split(this._pathSep); // ['files', 'file.txt']
  }

  addEntry(path, entry) {
    if (entry.type === 'D' && path === this._root.path) return; // adding root path is optional
    const names = this._pathNames(path);
    const entryName = names[names.length - 1];
    const entrySize = entry.size || 0;
    let node = this._root;
    // last item in names is the new item
    for (let i = 0; i < names.length - 1; ++i) {
      node.size += entrySize;
      node = node.dirs[names[i]];
    }
    if (entry.type === 'D') {
      node.dirs[entryName] = newDir(path);
    } else {
      node.size += entrySize;
      node.files.push(newFile(path, entry.size))
    }
  }
}

module.exports = FsTree;