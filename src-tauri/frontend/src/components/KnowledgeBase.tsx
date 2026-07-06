import React, { useState } from "react";
import type { KnowledgeEntry } from "../types";

interface Props {
  entries: KnowledgeEntry[];
  onAdd: (entry: Omit<KnowledgeEntry, "id" | "created" | "updated">) => void;
  onDelete: (id: string) => void;
  onSearch: (query: string) => void;
}

/**
 * KnowledgeBase — knowledge entry management panel.
 * Supports search, add/delete entries, tag-based categorization, and grouped display by category.
 */
const KnowledgeBase: React.FC<Props> = ({ entries, onAdd, onDelete, onSearch }) => {
  const [query, setQuery] = useState("");
  const [showForm, setShowForm] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newSource, setNewSource] = useState("");
  const [newCategory, setNewCategory] = useState("");
  const [newContent, setNewContent] = useState("");
  const [newTags, setNewTags] = useState("");

  const handleSearch = (value: string) => {
    setQuery(value);
    onSearch(value);
  };

  const handleAdd = () => {
    if (!newTitle.trim()) return;
    onAdd({
      title: newTitle,
      source: newSource,
      category: newCategory || "general",
      tags: newTags.split(",").map((t) => t.trim()).filter(Boolean),
      content: newContent,
    });
    setNewTitle("");
    setNewSource("");
    setNewCategory("");
    setNewContent("");
    setNewTags("");
    setShowForm(false);
  };

  const grouped = entries.reduce<Record<string, KnowledgeEntry[]>>((acc, entry) => {
    const cat = entry.category || "未分类";
    if (!acc[cat]) acc[cat] = [];
    acc[cat].push(entry);
    return acc;
  }, {});

  return (
    <div className="knowledge-base">
      <div className="knowledge-header">
        <h3>知识库</h3>
        <button className="btn-icon" onClick={() => setShowForm(!showForm)} title="添加知识">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M8 3v10M3 8h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
        </button>
      </div>

      <div className="knowledge-search">
        <input type="text" value={query} onChange={(e) => handleSearch(e.target.value)} placeholder="搜索知识库..." className="knowledge-input" />
      </div>

      {showForm && (
        <div className="knowledge-form">
          <input type="text" value={newTitle} onChange={(e) => setNewTitle(e.target.value)} placeholder="标题" className="knowledge-input" />
          <input type="text" value={newSource} onChange={(e) => setNewSource(e.target.value)} placeholder="来源 (可选)" className="knowledge-input" />
          <input type="text" value={newCategory} onChange={(e) => setNewCategory(e.target.value)} placeholder="分类 (可选)" className="knowledge-input" />
          <input type="text" value={newTags} onChange={(e) => setNewTags(e.target.value)} placeholder="标签, 逗号分隔 (可选)" className="knowledge-input" />
          <textarea value={newContent} onChange={(e) => setNewContent(e.target.value)} placeholder="内容" className="knowledge-textarea" rows={4} />
          <div className="knowledge-form-actions">
            <button className="btn-secondary" onClick={() => setShowForm(false)}>取消</button>
            <button className="btn-primary" onClick={handleAdd}>添加</button>
          </div>
        </div>
      )}

      <div className="knowledge-list">
        {Object.entries(grouped).length === 0 ? (
          <div className="knowledge-empty">
            <p>知识库为空</p>
            <p className="knowledge-hint">点击 + 添加知识条目</p>
          </div>
        ) : (
          Object.entries(grouped).map(([category, items]) => (
            <div key={category} className="knowledge-category">
              <div className="knowledge-category-title">{category}</div>
              {items.map((entry) => (
                <div key={entry.id} className="knowledge-entry">
                  <div className="knowledge-entry-header">
                    <span className="knowledge-entry-title">{entry.title}</span>
                    <button className="btn-icon" onClick={() => onDelete(entry.id)} title="删除">
                      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                        <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
                      </svg>
                    </button>
                  </div>
                  <div className="knowledge-entry-source">{entry.source}</div>
                  <div className="knowledge-entry-tags">
                    {entry.tags.map((tag) => (
                      <span key={tag} className="knowledge-tag">{tag}</span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default KnowledgeBase;
