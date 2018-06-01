import { h, app } from 'hyperapp'
import { State, Actions } from '../app'
import { readInputFile } from '../lib/utils'
import { WalletState, Wallet, createNewWallet, validateWallet, validatePassword, walletHref } from '../model/wallet'
import { InfoType } from '../model/info'
import { successOf } from '../lib/webdata'
import semux from 'semux'
import { Password } from '../lib/password'
import { Either } from 'tsmonad'

export interface WelcomeState {
  action: Action
}

type Action = LoadAction | CreateNewAction | undefined

export const initialWelcomeState: WelcomeState = {
  action: undefined,
}

interface LoadAction {
  kind: 'LoadAction'
  walletFile: any,
  errorMsg: string
}

const loadAction: LoadAction = {
  kind: 'LoadAction',
  walletFile: undefined,
  errorMsg: '',
}

interface CreateNewAction {
  kind: 'CreateNewAction'
  wallet: Wallet | undefined
  errorMsg: string
  importKeys: boolean
  myWalletIsSafe: boolean
}

const createNewAction: CreateNewAction = {
  kind: 'CreateNewAction',
  wallet: undefined,
  errorMsg: '',
  importKeys: false,
  myWalletIsSafe: false,
}

function isLoad(a: Action): a is LoadAction {
  return !!a && a.kind === 'LoadAction'
}
function isCreateNew(a: Action): a is CreateNewAction {
  return !!a && a.kind === 'CreateNewAction'
}

export interface WelcomeActions {
  setAction: (_: Action) => (s: WelcomeState) => WelcomeState
  setWalletFileBody: (body: Either<string, any>) => (s: WelcomeState) => WelcomeState
  load: (_: [Password, State, Actions]) => (s: WelcomeState, a: WelcomeActions) => WelcomeState
  create: (_: [Password, Password, State, Actions, string[]]) => (s: WelcomeState) => WelcomeState
  restoreInitialState: () => WelcomeState
}

export const rawWelcomeActions: WelcomeActions = {
  setAction: (action) => (state) => ({ ...state, action }),

  setWalletFileBody: (bodyE) => (state) => (
    bodyE.caseOf({
      left: (loadMessage) => ({ ...state, loadMessage }),
      right: (body) => ({ ...state, loadMessage: '', walletFile: body }),
    })
  ),

  load: ([password, rootState, rootActions]) => (state, actions) => {
    try {
      successOf(rootState.info).fmap((info) => {
        const wallet = validateWallet(state.walletFile, info.network)
        rootActions.setWallet(validatePassword(wallet, password))
      })
      return initialWelcomeState
    } catch (error) {
      return { ...state, loadMessage: error.message }
    }
  },

  create: ([password, password2, rootState, rootActions, privateKeys]) => (state) => {
    if (!password.equals(password2)) {
      return { ...state, createMessage: 'Passwords does not match' }
    }
    if (password.isEmpty()) {
      return { ...state, createMessage: 'Password cannot be empty' }
    }
    return successOf(rootState.info)
      .fmap((info) => {
        const wallet = createNewWallet(password, info.network, privateKeys)
        console.log('wallet', wallet)
        return { ...state, walletFile: wallet }
      })
      .valueOr(state)
  },

  restoreInitialState: () => initialWelcomeState,
}

export const WelcomeView = () => (rootState: State, rootActions: Actions) => {
  const state = rootState.welcome
  const actions = rootActions.welcome
  const action = state.action

  return isCreateNew(action) && state.walletFile
    ? newWalletView(action, rootState.wallet, actions)
    : actionsView(rootState, rootActions)
}

function actionsView(rootState: State, rootActions: Actions) {
  const actions = rootActions.welcome
  const state = rootState.welcome
  const action = state.action
  return <div class="pa3">
    <h1>Welcome to the Semux Light!</h1>

    <div class="mv2">
      <label>
        <input
          type="radio"
          name="welcome"
          checked={isLoad(state.action)}
          onclick={() => {
            actions.setAction(loadAction)
            document.getElementById('load')!.click()
          }}
        />
        {' '}Load wallet from file
    </label>
      <input
        class="clip"
        type="file"
        id="load"
        onchange={(evt) => readInputFile(evt.target)
          .then((body) => actions.setWalletFileBody(Either.right(body)))
          .catch((err) => actions.setWalletFileBody(Either.left(err.message)))
        }
      />
    </div>

    <div class="mv2">
      <label>
        <input
          type="radio"
          name="welcome"
          checked={isCreateNew(action)}
          onclick={() => {
            actions.setAction(createNewAction)
          }}
        />
        {' '}Create new wallet file
    </label>
    </div>

    <div class="mv3">
      <label class="fw7 f6">
        Password
      <input
          key="password"
          id="password"
          type="password"
          autocomplete="off"
          class="db pa2 br2 b--black-20 ba f6"
        />
      </label>
    </div>

    {isLoad(action) &&
      <div>
        <button onclick={() => actions.load([passwordById('password'), rootState, rootActions])}>
          Load wallet
      </button>
        <span class="ml2 dark-red">{action.errorMsg}</span>
      </div>
    }

    {isCreateNew(action) &&
      <div>
        <div class="mv3">
          <label class="fw7 f6">
            Repeat password
          <input
              key="password2"
              id="password2"
              type="password"
              autocomplete="off"
              class="db pa2 br2 b--black-20 ba f6"
            />
          </label>
        </div>

        <div class="mv3">
          <label class="fw7 f6">
            <input
              type="checkbox"
              name="import"
              checked={action.importKeys}
              onclick={() => {
                actions.setAction({ ...action, importKeys: !action.importKeys })
              }}
            />
            {' '}Import keys
        </label>
        </div>
        {action.importKeys &&
          <div class="mv3">
            <label class="fw7 f6">
              Private keys
            <br />
              <textarea
                class="w-100"
                id="privateKeys"
              />
            </label>
          </div>
        }

        <div>
          <button
            onclick={() => actions.create([
              passwordById('password'),
              passwordById('password2'),
              rootState,
              rootActions,
              privateKeysOfTextarea('privateKeys'),
            ])}
          >
            Create new wallet and save the file!
        </button><span class="ml2 dark-red">{action.errorMsg}</span>
        </div>
      </div>
    }
  </div>
}

function newWalletView(action: CreateNewAction, wallet: WalletState, actions: WelcomeActions) {
  return <div class="pa3">
    <div class="mv2">
      <h1>Your new wallet is (almost) ready!</h1>
      <p><b>Remember:</b></p>
      <ul>
        <li>The wallet + password = full access to your funds.</li>
        <li>Keep you wallet and password safe, but not together.</li>
        <li>If your password is easy to guess and wallet easy to get: your funds are not safe.</li>
        <li>Semux Light never sends wallet and/or passwords over the wire. Ever.</li>
      </ul>

      <p>Now, save your wallet:{' '}
        <a href={walletHref(wallet)} download="semux-wallet.json">
          click here
        </a>
      </p>
    </div>
    <div class="mv2">
      <label>
        <input
          type="checkbox"
          onclick={() => { actions.setAction({
            ...action, myWalletIsSafe: !action.myWalletIsSafe,
          })}}
        />
        {' '}Yes, my wallet is safe now.
      </label>
    </div>
    {action.myWalletIsSafe &&
      <div class="mv2">
        <button onclick={() => actions.restoreInitialState()}>
          I am ready to load my new wallet!
        </button>
      </div>
    }
  </div>
}

function passwordById(id: string): Password {
  return new Password((document.getElementById(id) as HTMLInputElement).value)
}

function privateKeysOfTextarea(id: string): string[] {
  const elem = document.getElementById(id) as HTMLTextAreaElement
  return elem ? elem.value.split(/\s/).map((e) => e.trim()).filter((e) => e) : []
}
