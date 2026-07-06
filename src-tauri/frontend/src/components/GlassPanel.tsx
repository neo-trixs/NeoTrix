import React from "react";

interface Props {
  variant?: "regular" | "clear" | "strong";
  as?: "div" | "section" | "article" | "aside" | "nav";
  noPadding?: boolean;
  noHeader?: boolean;
  header?: React.ReactNode;
  footer?: React.ReactNode;
  className?: string;
  children?: React.ReactNode;
  style?: React.CSSProperties;
}

const variantClass: Record<string, string> = {
  regular: "lg-panel",
  clear: "lg-glass-clear lg-panel",
  strong: "lg-glass-strong lg-panel",
};

const GlassPanel = React.forwardRef<HTMLDivElement, Props>(({
  variant = "regular",
  as: Tag = "div",
  noPadding,
  noHeader,
  header,
  footer,
  className = "",
  children,
  style,
}, ref) => {
  const cls = `${variantClass[variant]} ${className}`;
  const bodyCls = noPadding ? "lg-panel-body-no-padding" : "lg-panel-body";
  return (
    <Tag ref={ref} className={cls} style={style}>
      {!noHeader && header && <div className="lg-panel-header">{header}</div>}
      <div className={bodyCls}>{children}</div>
      {footer && <div className="lg-panel-footer">{footer}</div>}
    </Tag>
  );
});

GlassPanel.displayName = "GlassPanel";

export default GlassPanel;
