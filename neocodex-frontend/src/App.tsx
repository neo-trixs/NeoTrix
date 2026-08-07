import { Router, Route } from '@solidjs/router'
import { Chat } from './routes/Chat'
import { TrafficLights } from './components/TrafficLights'

export default function App() {
  return (
    <>
      <TrafficLights />
      <Router>
        <Route path="/" component={Chat} />
        <Route path="/chat" component={Chat} />
      </Router>
    </>
  )
}