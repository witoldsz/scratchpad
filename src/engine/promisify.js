const promisify = fn => new Promise((resolve, reject) => {
  fn((err, result) => {
    if (err) reject(err);
    else resolve(result);
  });
});

module.exports = promisify;