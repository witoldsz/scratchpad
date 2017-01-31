const serialMap = promiseMapper => items => {
  return items.reduce(
    (prev, item) => prev.then(() => promiseMapper(item)),
    Promise.resolve());
};

module.exports = serialMap;