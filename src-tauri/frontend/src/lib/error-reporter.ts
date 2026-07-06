const REPORT_URL = '/api/crash-report';

interface ErrorReport {
  message: string;
  stack?: string;
  component?: string;
  timestamp: string;
  url: string;
  userAgent: string;
}

export function reportError(error: Error, component?: string): void {
  try {
    const report: ErrorReport = {
      message: error.message,
      stack: error.stack,
      component,
      timestamp: new Date().toISOString(),
      url: window.location.href,
      userAgent: navigator.userAgent,
    };
    if (navigator.sendBeacon) {
      navigator.sendBeacon(REPORT_URL, JSON.stringify(report));
    } else {
      fetch(REPORT_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(report),
        keepalive: true,
      }).catch(() => {});
    }
  } catch {
  }
}

export function captureConsoleErrors(): void {
  const origOnError = window.onerror;
  window.onerror = (_msg, _source, _lineno, _colno, error) => {
    if (error) reportError(error);
    if (origOnError) return origOnError.call(window, _msg, _source, _lineno, _colno, error);
    return false;
  };
  const origOnUnhandled = window.onunhandledrejection;
  window.onunhandledrejection = (event) => {
    const error = event.reason instanceof Error ? event.reason : new Error(String(event.reason));
    reportError(error, 'unhandled-promise');
    if (origOnUnhandled) return origOnUnhandled.call(window, event);
  };
}
