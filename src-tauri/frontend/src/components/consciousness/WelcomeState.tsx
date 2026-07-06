import React from "react";

const SUGGESTIONS = [
  {
    id: "blog",
    text: "Write a blog post",
    icon: <svg viewBox="0 0 14 14"><path d="M4 3L10 9M10 3L4 9" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/><rect x="2" y="2" width="10" height="10" rx="2" stroke="currentColor" strokeWidth="1.3"/></svg>,
  },
  {
    id: "explain",
    text: "Explain code",
    icon: <svg viewBox="0 0 14 14"><circle cx="7" cy="4.5" r="2.5" stroke="currentColor" strokeWidth="1.3"/><path d="M2 12c0-2.76 2.24-5 5-5s5 2.24 5 5" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/></svg>,
  },
  {
    id: "brainstorm",
    text: "Brainstorm ideas",
    icon: <svg viewBox="0 0 14 14"><circle cx="7" cy="4" r="3" stroke="currentColor" strokeWidth="1.3"/><path d="M3.5 12a3 3 0 017 0" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/><path d="M10 10l2.5 2.5M4 10l-2.5 2.5" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/></svg>,
  },
  {
    id: "debug",
    text: "Debug my code",
    icon: <svg viewBox="0 0 14 14"><path d="M5 5l4 4M9 5l-4 4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/><circle cx="7" cy="7" r="5.5" stroke="currentColor" strokeWidth="1.3"/></svg>,
  },
];

const WelcomeState: React.FC<{ onSuggestionClick: (text: string) => void }> = ({ onSuggestionClick }) => {
  return (
    <>
      <div className="hero">
        <div className="hero-svg">
          <svg viewBox="0 0 36 36" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="18" cy="18" r="16" stroke="var(--nt-primary)" strokeWidth="1.8" opacity="0.3" />
            <circle cx="18" cy="18" r="10" stroke="var(--nt-primary)" strokeWidth="1.6" opacity="0.5" />
            <circle cx="18" cy="18" r="4" fill="var(--nt-primary)" opacity="0.8" />
            <path d="M18 2v6M18 28v6M2 18h6M28 18h6" stroke="var(--nt-primary)" strokeWidth="1.5" strokeLinecap="round" opacity="0.4" />
            <path d="M7.5 7.5l4.5 4.5M24 24l4.5 4.5M7.5 28.5l4.5-4.5M24 12l4.5-4.5" stroke="var(--nt-primary)" strokeWidth="1.2" strokeLinecap="round" opacity="0.25" />
          </svg>
        </div>
        <h1>NeoTrix</h1>
      </div>

      <div className="qa">
        {SUGGESTIONS.map((s) => (
          <button key={s.id} className="qa-btn" onClick={() => onSuggestionClick(s.text)}>
            {s.icon}
            {s.text}
          </button>
        ))}
      </div>
    </>
  );
};

export default WelcomeState;
