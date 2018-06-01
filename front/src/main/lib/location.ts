import { h } from 'hyperapp'
import { State } from '../app'

export interface LocationState {
  route: string
}

export interface LocationActions {
  setCurrent: (s: LocationState) => LocationState
}

export const rawLocationActions: LocationActions = {
  setCurrent: (state: LocationState) => state,
}

export function locationSubscribe(sub: (s: LocationState) => any): void {
  function listener() {
    sub(parseLocation())
  }
  listener()
  window.addEventListener('hashchange', listener, false)
}

type LinkProps = { to: string } & JSX.IntrinsicElements

export const Link = (props: LinkProps, children) => (s: State) => {
  const { to, ...otherProps } = props
  return h('a', {
    href: `#/${to}`,
    ...otherProps,
  }, children)
}

export interface RouteProps { path: string, render: JSX.Element }

export const Route = (props: RouteProps) => (s: State) => (
  s.location.route === props.path ? props.render : undefined
)

function routeMatches(s: LocationState, path: string) {
  const c = s.route
}

export function parseLocation(): LocationState {
  const locationHash = window.location.hash
  const hash =  locationHash.split('#').slice(1).join('#')
  const route = hash.split('/').filter((x) => x).join('/')
  return { route }
}
