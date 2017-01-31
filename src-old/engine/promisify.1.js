const promisify = () => {
  let cnt = 0, scheduled = 0;
  setInterval(() => console.log(`cnt=${cnt} scheduled=${scheduled}`), 1000);
  return fn => new Promise((resolve, reject) => {
    scheduled++;
    const doit = () => {
      if (cnt > 10) {
        setTimeout(doit, 1);
      } else {
        cnt++;
        fn((err, result) => {
          cnt--;
          scheduled--;
          if (err) reject(err);
          else resolve(result);
        });
      }
    };
    doit();
  });
};
module.exports = promisify;