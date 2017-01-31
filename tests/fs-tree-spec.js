'use strict'

const chai = require('chai');
const expect = chai.expect;
const FsTree = require('../renderer/fs-tree');

describe(`fs-tree`, () => {

  let fs;

  beforeEach(() => {
    fs = new FsTree('/home/user', '/');
  });

  it(`should view empty fs`, () => {
    expect(fs.viewRoot()).to.eql([
      { path: '/home/user', size: 0, type: 'D' }
    ]);
    expect(fs.viewPath('/home/user/does_not_exist')).to.eql([]);
  });

  it('should ignore when root path is added', () => {
    fs.addEntry('/home/user', { type: 'D' });
    expect(fs.viewRoot()).to.eql([
      { path: '/home/user', size: 0, type: 'D' }
    ]);
  });

  it(`should view with one file`, () => {
    fs.addEntry('/home/user/file.txt', { type: 'F', size: 123 });
    expect(fs.viewRoot()).to.eql([
      { path: '/home/user', size: 123, type: 'D' },
      { path: '/home/user/file.txt', size: 123, type: 'F' }
    ]);
  });

  it(`should view with one dir`, () => {
    fs.addEntry('/home/user/new_dir', { type: 'D' });
    expect(fs.viewRoot()).to.eql([
      { path: '/home/user', size: 0, type: 'D' },
      { path: '/home/user/new_dir', size: 0, type: 'D' }
    ]);
  });

  it(`should view with files and dirs`, () => {
    fs.addEntry('/home/user/new_dir_1', { type: 'D' });
    fs.addEntry('/home/user/new_dir_2', { type: 'D' });
    fs.addEntry('/home/user/old_file.txt', { size: 11, type: 'F' });
    fs.addEntry('/home/user/new_dir_2/new_file_2.txt', { size: 7, type: 'F' });

    expect(fs.viewRoot()).to.eql([
      { path: '/home/user', size: 18, type: 'D' },
      { path: '/home/user/new_dir_1', size: 0, type: 'D' },
      { path: '/home/user/new_dir_2', size: 7, type: 'D' },
      { path: '/home/user/old_file.txt', size: 11, type: 'F' },
    ]);

    expect(fs.viewPath('/home/user/new_dir_1')).to.eql([]);

    expect(fs.viewPath('/home/user/new_dir_2')).to.eql([
      { path: '/home/user/new_dir_2/new_file_2.txt', size: 7, type: 'F' },
    ]);
  });

});