import { useMemo } from "react";

interface StructuredContentProps {
  content: string;
  renderMarkdown: (text: string) => string;
}

type ContentFormat = "prose" | "list" | "steps" | "cards" | "code" | "mixed";

interface ContentSection {
  format: ContentFormat;
  content: string;
  label?: string;
}

function analyzeStructure(content: string): ContentSection[] {
  const sections: ContentSection[] = [];
  const trimmed = content.trim();
  if (!trimmed) return [{ format: "prose", content }];

  // split by double newlines first
  const blocks = trimmed.split(/\n\n+/);

  for (const block of blocks) {
    const lines = block.split("\n").filter(l => l.trim());
    if (lines.length === 0) continue;

    // Detect numbered steps (e.g., "1. ", "Step 1:")
    const stepMatch = block.match(/^(?:Step\s+\d+|#+\s+\d+\.|\d+\.\s)/im);
    if (stepMatch && lines.length >= 2) {
      sections.push({ format: "steps", content: block, label: "Steps" });
      continue;
    }

    // Detect card-like sections (### Header followed by content)
    const cardMatch = block.match(/^#{2,3}\s+(.+)/m);
    if (cardMatch && lines.length >= 3) {
      sections.push({ format: "cards", content: block, label: cardMatch[1] });
      continue;
    }

    // Detect fenced code blocks
    if (block.startsWith("```") && block.endsWith("```")) {
      sections.push({ format: "code", content: block });
      continue;
    }

    // Detect lists (- or * items, 3+ items)
    const listItems = lines.filter(l => /^[-*]\s/.test(l.trim()) || /^\d+[.)]\s/.test(l.trim()));
    if (listItems.length >= 3) {
      sections.push({ format: "list", content: block });
      continue;
    }

    // Default: prose
    sections.push({ format: "prose", content: block });
  }

  if (sections.length <= 1) return sections;

  // If mixed types, keep them as-is
  return sections;
}

function StepsSection({ content, renderMarkdown }: { content: string; renderMarkdown: (t: string) => string }) {
  const steps: { num: string; text: string }[] = [];
  const lines = content.split("\n").filter(l => l.trim());
  let currentNum = "";
  let currentText: string[] = [];

  for (const line of lines) {
    const headerMatch = line.match(/^(?:Step\s+(\d+)[:\s]+|#+\s+\d+\.\s+(.+))/i);
    const numberedMatch = line.match(/^(\d+)[.)]\s+(.+)/);
    if (headerMatch) {
      if (currentText.length) {
        steps.push({ num: currentNum, text: currentText.join("\n") });
      }
      currentNum = headerMatch[1] || "•";
      currentText = [headerMatch[2] || line.replace(/^#+\s*/, "")];
    } else if (numberedMatch) {
      if (currentText.length) {
        steps.push({ num: currentNum, text: currentText.join("\n") });
      }
      currentNum = numberedMatch[1];
      currentText = [numberedMatch[2]];
    } else {
      currentText.push(line);
    }
  }
  if (currentText.length) {
    steps.push({ num: currentNum, text: currentText.join("\n") });
  }

  return (
    <div className="sc-timeline">
      {steps.map((step, i) => (
        <div key={i} className="sc-step">
          <div className="sc-step-number">{step.num}</div>
          <div className="sc-step-content" dangerouslySetInnerHTML={{ __html: renderMarkdown(step.text) }} />
        </div>
      ))}
    </div>
  );
}

function ListSection({ content, renderMarkdown }: { content: string; renderMarkdown: (t: string) => string }) {
  const items: { icon?: string; text: string }[] = [];
  const lines = content.split("\n").filter(l => l.trim());

  for (const line of lines) {
    const match = line.match(/^[-*]\s+(.*)/);
    const numberedMatch = line.match(/^\d+[.)]\s+(.*)/);
    if (match) {
      items.push({ text: match[1] });
    } else if (numberedMatch) {
      items.push({ text: numberedMatch[1] });
    }
  }

  return (
    <div className="sc-list">
      {items.map((item, i) => (
        <div key={i} className="sc-list-item">
          <span className="sc-list-bullet">•</span>
          <span dangerouslySetInnerHTML={{ __html: renderMarkdown(item.text) }} />
        </div>
      ))}
    </div>
  );
}

function CardsSection({ content, renderMarkdown }: { content: string; renderMarkdown: (t: string) => string }) {
  const cards: { title: string; body: string }[] = [];
  const lines = content.split("\n");
  let currentTitle = "";
  let currentBody: string[] = [];

  for (const line of lines) {
    const headerMatch = line.match(/^#{2,3}\s+(.+)/);
    if (headerMatch) {
      if (currentTitle) {
        cards.push({ title: currentTitle, body: currentBody.join("\n").trim() });
      }
      currentTitle = headerMatch[1];
      currentBody = [];
    } else {
      currentBody.push(line);
    }
  }
  if (currentTitle) {
    cards.push({ title: currentTitle, body: currentBody.join("\n").trim() });
  }

  return (
    <div className="sc-cards">
      {cards.map((card, i) => (
        <div key={i} className="sc-card">
          <div className="sc-card-title">{card.title}</div>
          {card.body && (
            <div className="sc-card-body" dangerouslySetInnerHTML={{ __html: renderMarkdown(card.body) }} />
          )}
        </div>
      ))}
    </div>
  );
}

export default function StructuredContent({ content, renderMarkdown }: StructuredContentProps) {
  const sections = useMemo(() => analyzeStructure(content), [content]);

  if (sections.length <= 1 && sections[0]?.format === "prose") {
    return (
      <div className="message-content" dangerouslySetInnerHTML={{ __html: renderMarkdown(content) }} />
    );
  }

  // Count format types to decide if we need mixed layout
  const formatCounts = sections.reduce<Record<string, number>>((acc, s) => {
    acc[s.format] = (acc[s.format] || 0) + 1;
    return acc;
  }, {});

  const hasMixedFormats = Object.keys(formatCounts).length > 1;

  return (
    <div className={`sc-container ${hasMixedFormats ? "sc-mixed" : ""}`}>
      {sections.map((section, i) => {
        switch (section.format) {
          case "steps":
            return <StepsSection key={i} content={section.content} renderMarkdown={renderMarkdown} />;
          case "list":
            return <ListSection key={i} content={section.content} renderMarkdown={renderMarkdown} />;
          case "cards":
            return <CardsSection key={i} content={section.content} renderMarkdown={renderMarkdown} />;
          case "code":
            return <div key={i} className="sc-code-block" dangerouslySetInnerHTML={{ __html: renderMarkdown(section.content) }} />;
          default:
            return (
              <div key={i} className="sc-prose" dangerouslySetInnerHTML={{ __html: renderMarkdown(section.content) }} />
            );
        }
      })}
    </div>
  );
}
