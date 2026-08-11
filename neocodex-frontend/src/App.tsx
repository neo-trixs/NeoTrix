import { Router, Route } from '@solidjs/router'
import { Chat } from './routes/Chat'
import { GlobeView } from './components/GlobeView'
import { TrafficLights } from './components/TrafficLights'

function GlobeRoute() {
  return <GlobeView limit={5000} height={700} />
}

export default function App() {
  return (
    <>
      <TrafficLights />
      <Router>
        <Route path="/" component={Chat} />
        <Route path="/chat" component={Chat} />
        <Route path="/globe" component={GlobeRoute} />
      </Router>
    </>
  )
}