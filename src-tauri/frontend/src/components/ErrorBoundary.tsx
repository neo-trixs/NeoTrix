import React from "react";

interface Props {
  children: React.ReactNode;
  fallback?: React.ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback || (
        <div className="error-boundary glass-panel" style={{ padding: 20, margin: 8 }}>
          <h3 style={{ color: "var(--danger)", marginBottom: 8 }}>渲染错误</h3>
          <pre style={{ fontSize: 12, whiteSpace: "pre-wrap", color: "var(--mac-text-secondary)" }}>
            {this.state.error?.message}
          </pre>
          <button
            className="btn-primary"
            style={{ marginTop: 12 }}
            onClick={() => this.setState({ hasError: false, error: null })}
          >
            重试
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
