import { createRoot } from 'react-dom/client';
import { AppProvider } from './hooks/useApp';

import App from './App';
import './styles/index.css';

function Root() {
  return (
    <AppProvider>
      <App />
    </AppProvider>
  );
}

const container = document.getElementById('root');
if (container) {
  const root = createRoot(container);
  root.render(<Root />);
}
