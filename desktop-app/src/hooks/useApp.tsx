import { createContext, useContext, useReducer, useEffect, type ReactNode } from 'react';

interface AppState {
  sidebarOpen: boolean;
  rightPanelOpen: boolean;
  currentTheme: 'light' | 'dark';
  currentLayout: 'default' | 'compact' | 'wide';
  currentSession: string;
}

interface AppContextType {
  state: AppState;
  toggleSidebar: () => void;
  toggleRightPanel: () => void;
  setTheme: (theme: 'light' | 'dark') => void;
  setLayout: (layout: 'default' | 'compact' | 'wide') => void;
  setCurrentSession: (session: string) => void;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

type AppAction =
  | { type: 'TOGGLE_SIDEBAR' }
  | { type: 'TOGGLE_RIGHT_PANEL' }
  | { type: 'SET_THEME'; payload: 'light' | 'dark' }
  | { type: 'SET_LAYOUT'; payload: 'default' | 'compact' | 'wide' }
  | { type: 'SET_CURRENT_SESSION'; payload: string };

function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case 'TOGGLE_SIDEBAR':
      return { ...state, sidebarOpen: !state.sidebarOpen };
    case 'TOGGLE_RIGHT_PANEL':
      return { ...state, rightPanelOpen: !state.rightPanelOpen };
    case 'SET_THEME':
      return { ...state, currentTheme: action.payload };
    case 'SET_LAYOUT':
      return { ...state, currentLayout: action.payload };
    case 'SET_CURRENT_SESSION':
      return { ...state, currentSession: action.payload };
    default:
      return state;
  }
}

function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, {
    sidebarOpen: true,
    rightPanelOpen: true,
    currentTheme: (localStorage.getItem('neo-theme') as 'light' | 'dark') || 'dark',
    currentLayout: 'default',
    currentSession: 'default',
  });

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', state.currentTheme);
    localStorage.setItem('neo-theme', state.currentTheme);
  }, [state.currentTheme]);

  const value: AppContextType = {
    state,
    toggleSidebar: () => dispatch({ type: 'TOGGLE_SIDEBAR' }),
    toggleRightPanel: () => dispatch({ type: 'TOGGLE_RIGHT_PANEL' }),
    setTheme: (theme: 'light' | 'dark') => dispatch({ type: 'SET_THEME', payload: theme }),
    setLayout: (layout: 'default' | 'compact' | 'wide') => dispatch({ type: 'SET_LAYOUT', payload: layout }),
    setCurrentSession: (session: string) => dispatch({ type: 'SET_CURRENT_SESSION', payload: session }),
  };

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

function useApp() {
  const context = useContext(AppContext);
  if (context === undefined) {
    throw new Error('useApp must be used within an AppProvider');
  }
  return context;
}

export { AppProvider, useApp };
