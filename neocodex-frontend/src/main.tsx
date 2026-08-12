import { render } from 'solid-js/web'
import App from './App'
import { initErrorMonitor } from './lib/errorMonitor'
import './styles/index.css'

initErrorMonitor()

render(() => <App />, document.getElementById('root')!)