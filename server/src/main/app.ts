import * as express from 'express'
import * as path from 'path'
import * as httpProxy from 'http-proxy'
import * as request from 'superagent'
import * as hsts from 'hsts'
import { settings } from './settings'

const staticsPath = path.join(__dirname, '..', '..', 'front', 'dist')

async function main() {
  const app = express()
  app.use(hsts())
  app.use(express.static(staticsPath))

  const { address, user, pass } = settings.semuxApiService
  const proxy = httpProxy.createProxy({
    target: address,
    auth: `${user}:${pass}`,
    proxyTimeout: 5000,
  })
  const proxyMiddleware = (req, res, next) => {
    proxy.web(req, res, undefined, next)
  }
  app.get('/v2.1.0/info', proxyMiddleware)
  app.get('/v2.1.0/account', proxyMiddleware)
  app.get('/v2.1.0/account/transactions', proxyMiddleware)
  app.get('/v2.1.0/account/pending-transactions', proxyMiddleware)
  app.get('/v2.1.0/account/votes', proxyMiddleware)
  app.get('/v2.1.0/delegates', proxyMiddleware)
  app.get('/v2.1.0/latest-block', proxyMiddleware)
  app.post('/v2.1.0/transaction/raw', proxyMiddleware)

  app.use((err, req, res, next) => {
    res.status(500).json({
      success: false,
      message: `proxy server error: ${err.message}`,
    })
  })

  const addr = await new Promise((resolve) => {
    const server = app.listen(3333, '127.0.0.1', () => resolve(server.address()))
  })
  console.log('server listening:', addr)
}

main()
