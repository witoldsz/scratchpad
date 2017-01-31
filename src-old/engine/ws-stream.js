var websocket = require('websocket-stream')
var wss = websocket.createServer({server: someHTTPServer}, handle)
 
function handle(stream) {
  fs.createReadStream('bigdata.json').pipe(stream)
}