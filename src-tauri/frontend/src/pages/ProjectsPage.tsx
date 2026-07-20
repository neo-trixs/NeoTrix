import React, { useEffect, useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../stores";
import * as api from "../lib/api";
import type { Project, ProjectChat, ProjectSource, ProjectInstruction } from "../types";
import styles from "./ProjectsPage.module.css";

interface ProjectsPageProps {}

const ProjectsPage: React.FC<ProjectsPageProps> = () => {
  const navigate = useNavigate();
  const sessions = useStore((s) => s.sessions);
  const activeSessionIndex = useStore((s) => s.activeSessionIndex);
  const setSessions = useStore((s) => s.setSessions);
  const setActiveSessionIndex = useStore((s) => s.setActiveSessionIndex);

  const [projects, setProjects] = useState<Project[]>([]);
  const [chats, setChats] = useState<Record<string, ProjectChat[]>>({});
  const [sources, setSources] = useState<Record<string, ProjectSource[]>>({});
  const [instructions, setInstructions] = useState<Record<string, ProjectInstruction[]>>({});

  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [selectedChatId, setSelectedChatId] = useState<string | null>(null);

  const [showCreateProject, setShowCreateProject] = useState(false);
  const [showCreateChat, setShowCreateChat] = useState(false);
  const [showAddSource, setShowAddSource] = useState(false);
  const [showAddInstruction, setShowAddInstruction] = useState(false);

  const [newProjectName, setNewProjectName] = useState("");
  const [newProjectPath, setNewProjectPath] = useState("");
  const [newProjectType, setNewProjectType] = useState("local");
  const [newProjectDesc, setNewProjectDesc] = useState("");

  const [newChatName, setNewChatName] = useState("");

  const [newSourceType, setNewSourceType] = useState("folder");
  const [newSourcePath, setNewSourcePath] = useState("");
  const [newSourceUrl, setNewSourceUrl] = useState("");
  const [newSourceName, setNewSourceName] = useState("");

  const [newInstruction, setNewInstruction] = useState("");

  const [loading, setLoading] = useState(false);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  const showStatus = useCallback((msg: string) => {
    setStatusMsg(msg);
    setTimeout(() => setStatusMsg(null), 3000);
  }, []);

  const loadProjects = useCallback(async () => {
    try {
      setLoading(true);
      const list = await invoke<Project[]>("project_list");
      setProjects(list);
    } catch (e) {
      console.error("Failed to load projects:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadChats = useCallback(async (projectId: string) => {
    try {
      const list = await invoke<ProjectChat[]>("project_chat_list", { projectId });
      setChats((prev) => ({ ...prev, [projectId]: list }));
    } catch (e) {
      console.error("Failed to load chats:", e);
    }
  }, []);

  const loadSources = useCallback(async (projectId: string) => {
    try {
      const list = await invoke<ProjectSource[]>("project_source_list", { projectId });
      setSources((prev) => ({ ...prev, [projectId]: list }));
    } catch (e) {
      console.error("Failed to load sources:", e);
    }
  }, []);

  const loadInstructions = useCallback(async (projectId: string) => {
    try {
      const list = await invoke<ProjectInstruction[]>("project_instruction_list", { projectId });
      setInstructions((prev) => ({ ...prev, [projectId]: list }));
    } catch (e) {
      console.error("Failed to load instructions:", e);
    }
  }, []);

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  useEffect(() => {
    if (selectedProjectId) {
      loadChats(selectedProjectId);
      loadSources(selectedProjectId);
      loadInstructions(selectedProjectId);
    }
  }, [selectedProjectId, loadChats, loadSources, loadInstructions]);

  const handleCreateProject = async () => {
    if (!newProjectName.trim() || !newProjectPath.trim()) return;
    try {
      const project = await invoke<Project>("project_create", {
        name: newProjectName.trim(),
        path: newProjectPath.trim(),
        projectType: newProjectType,
        description: newProjectDesc.trim() || undefined,
      });
      setProjects((prev) => [project, ...prev]);
      setShowCreateProject(false);
      setNewProjectName("");
      setNewProjectPath("");
      setNewProjectDesc("");
      setSelectedProjectId(project.id);
      showStatus(`Project "${project.name}" created`);
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleScanProject = async () => {
    if (!newProjectPath.trim()) return;
    try {
      const project = await invoke<Project>("project_scan_directory", { path: newProjectPath.trim() });
      setProjects((prev) => [project, ...prev]);
      setShowCreateProject(false);
      setNewProjectName("");
      setNewProjectPath("");
      setSelectedProjectId(project.id);
      showStatus(`Project "${project.name}" scanned and added`);
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleUpdateProject = async (id: string, updates: Partial<Project>) => {
    try {
      const project = await invoke<Project>("project_update", { id, ...updates });
      setProjects((prev) => prev.map((p) => (p.id === id ? project : p)));
      showStatus("Project updated");
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleDeleteProject = async (id: string) => {
    if (!window.confirm("Delete this project? This cannot be undone.")) return;
    try {
      await invoke("project_delete", { id });
      setProjects((prev) => prev.filter((p) => p.id !== id));
      setChats((prev) => { const next = { ...prev }; delete next[id]; return next; });
      setSources((prev) => { const next = { ...prev }; delete next[id]; return next; });
      setInstructions((prev) => { const next = { ...prev }; delete next[id]; return next; });
      if (selectedProjectId === id) setSelectedProjectId(null);
      showStatus("Project deleted");
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleCreateChat = async () => {
    if (!selectedProjectId || !newChatName.trim()) return;
    try {
      const chat = await invoke<ProjectChat>("project_chat_create", {
        projectId: selectedProjectId,
        name: newChatName.trim(),
        sessionId: undefined,
      });
      setChats((prev) => ({
        ...prev,
        [selectedProjectId]: [chat, ...(prev[selectedProjectId] || [])],
      }));
      setShowCreateChat(false);
      setNewChatName("");
      showStatus(`Chat "${chat.name}" created`);
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleOpenChat = async (chat: ProjectChat) => {
    setSelectedChatId(chat.id);
    if (chat.sessionId) {
      const allSessions = await invoke<any[]>("session_list");
      const idx = allSessions.findIndex((s) => s.id === chat.sessionId);
      if (idx >= 0) {
        setActiveSessionIndex(idx);
      }
    }
    navigate("/");
  };

  const handleCreateSource = async () => {
    if (!selectedProjectId || !newSourceName.trim()) return;
    try {
      const source = await invoke<ProjectSource>("project_source_add", {
        projectId: selectedProjectId,
        sourceType: newSourceType,
        path: newSourcePath.trim() || undefined,
        url: newSourceUrl.trim() || undefined,
        name: newSourceName.trim(),
      });
      setSources((prev) => ({
        ...prev,
        [selectedProjectId]: [source, ...(prev[selectedProjectId] || [])],
      }));
      setShowAddSource(false);
      setNewSourcePath("");
      setNewSourceUrl("");
      setNewSourceName("");
      showStatus(`Source "${source.name}" added`);
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleCreateInstruction = async () => {
    if (!selectedProjectId || !newInstruction.trim()) return;
    try {
      const ins = await invoke<ProjectInstruction>("project_instruction_add", {
        projectId: selectedProjectId,
        content: newInstruction.trim(),
      });
      setInstructions((prev) => ({
        ...prev,
        [selectedProjectId]: [ins, ...(prev[selectedProjectId] || [])],
      }));
      setShowAddInstruction(false);
      setNewInstruction("");
      showStatus("Instruction added");
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const selectedProject = projects.find((p) => p.id === selectedProjectId);
  const projectChats = selectedProjectId ? chats[selectedProjectId] || [] : [];
  const projectSources = selectedProjectId ? sources[selectedProjectId] || [] : [];
  const projectInstructions = selectedProjectId ? instructions[selectedProjectId] || [] : [];

  return (
    <div className={styles.projectsPage}>
      {statusMsg && <div className={styles.toast}>{statusMsg}</div>}

      <div className={styles.projectsSidebar}>
        <div className={styles.sidebarHeader}>
          <h2>Projects</h2>
          <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={() => setShowCreateProject(true)}>
            + New Project
          </button>
        </div>

        <div className={styles.projectsList}>
          {loading ? (
            <div className={styles.loading}>Loading projects...</div>
          ) : projects.length === 0 ? (
            <div className={styles.emptyState}>
              <p>No projects yet</p>
              <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowCreateProject(true)}>
                Create your first project
              </button>
            </div>
          ) : (
            projects.map((project) => (
              <div
                key={project.id}
                className={`${styles.projectItem} ${selectedProjectId === project.id ? styles.selected : ""} ${project.pinned ? styles.pinned : ""} ${project.archived ? styles.archived : ""}`}
                onClick={() => setSelectedProjectId(project.id)}
              >
                <div className={styles.projectInfo}>
                  <div className={styles.projectHeader}>
                    <span className={styles.projectName}>{project.name}</span>
                    {project.pinned && <span className={styles.pinBadge}>📌</span>}
                    {project.archived && <span className={styles.archiveBadge}>📦</span>}
                  </div>
                  <div className={styles.projectMeta}>
                    <span className={styles.projectPath}>{project.path}</span>
                    <span className={styles.projectType}>{project.project_type}</span>
                  </div>
                  {project.description && <div className={styles.projectDesc}>{project.description}</div>}
                </div>
                <div className={styles.projectActions}>
                  <button
                    className={`${styles.iconBtn} ${project.pinned ? styles.active : ""}`}
                    onClick={(e) => { e.stopPropagation(); handleUpdateProject(project.id, { pinned: !project.pinned }); }}
                    title={project.pinned ? "Unpin" : "Pin"}
                  >
                    {project.pinned ? "📌" : "📍"}
                  </button>
                  <button
                    className={`${styles.iconBtn} ${styles.danger}`}
                    onClick={(e) => { e.stopPropagation(); handleDeleteProject(project.id); }}
                    title="Delete"
                  >
                    🗑️
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {selectedProject && (
        <div className={styles.projectsMain}>
          <div className={styles.mainHeader}>
            <div>
              <h2>{selectedProject.name}</h2>
              <p className={styles.projectPath}>{selectedProject.path}</p>
            </div>
            <div className={styles.mainActions}>
              <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowCreateChat(true)}>
                + New Chat
              </button>
              <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowAddSource(true)}>
                + Add Source
              </button>
              <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowAddInstruction(true)}>
                + Add Instruction
              </button>
            </div>
          </div>

          <div className={styles.mainTabs}>
            <button className={`${styles.tab} ${!selectedChatId ? styles.active : ""}`} onClick={() => setSelectedChatId(null)}>
              Overview
            </button>
            <button className={`${styles.tab} ${selectedChatId === "chats" ? styles.active : ""}`} onClick={() => setSelectedChatId("chats")}>
              Chats ({projectChats.length})
            </button>
            <button className={`${styles.tab} ${selectedChatId === "sources" ? styles.active : ""}`} onClick={() => setSelectedChatId("sources")}>
              Sources ({projectSources.length})
            </button>
            <button className={`${styles.tab} ${selectedChatId === "instructions" ? styles.active : ""}`} onClick={() => setSelectedChatId("instructions")}>
              Instructions ({projectInstructions.length})
            </button>
          </div>

          {selectedChatId === null && (
            <div className={`${styles.tabContent} ${styles.overview}`}>
              <div className={styles.overviewGrid}>
                <div className={styles.overviewCard}>
                  <h3>Project Info</h3>
                  <p><strong>Type:</strong> {selectedProject.project_type}</p>
                  <p><strong>Created:</strong> {new Date(selectedProject.created_at * 1000).toLocaleString()}</p>
                  <p><strong>Updated:</strong> {new Date(selectedProject.updated_at * 1000).toLocaleString()}</p>
                  {selectedProject.description && <p><strong>Description:</strong> {selectedProject.description}</p>}
                </div>
              </div>
            </div>
          )}

          {selectedChatId === "chats" && (
            <div className={styles.tabContent}>
              <div className={styles.chatsList}>
                {projectChats.length === 0 ? (
                  <div className={styles.emptyState}>
                    <p>No chats in this project</p>
                    <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowCreateChat(true)}>
                      Create first chat
                    </button>
                  </div>
                ) : (
                  projectChats.map((chat) => (
                    <div
                      key={chat.id}
                      className={`${styles.chatItem} ${chat.pinned ? styles.pinned : ""} ${chat.archived ? styles.archived : ""}`}
                    >
                      <div className={styles.chatInfo}>
                        <span className={styles.chatName}>{chat.name} {chat.pinned && "📌"}</span>
                        <span className={styles.chatMeta}>
                          {chat.message_count} messages · {new Date(chat.updated_at * 1000).toLocaleString()}
                        </span>
                      </div>
                      <div className={styles.chatActions}>
                        <button className={`${styles.iconBtn} ${chat.pinned ? styles.active : ""}`}
                          onClick={() => handleUpdateChat(chat.id, { pinned: !chat.pinned })}
                          title={chat.pinned ? "Unpin" : "Pin"}
                        >
                          {chat.pinned ? "📌" : "📍"}
                        </button>
                        <button className={styles.iconBtn} onClick={() => handleOpenChat(chat)} title="Open">
                          →
                        </button>
                        <button className={`${styles.iconBtn} ${styles.danger}`} onClick={() => handleDeleteChat(chat.id)} title="Delete">
                          🗑️
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          )}

          {selectedChatId === "sources" && (
            <div className={styles.tabContent}>
              <div className={styles.sourcesList}>
                {projectSources.length === 0 ? (
                  <div className={styles.emptyState}>
                    <p>No sources connected</p>
                    <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowAddSource(true)}>
                      Add source
                    </button>
                  </div>
                ) : (
                  projectSources.map((source) => (
                    <div
                      key={source.id}
                      className={`${styles.sourceItem} ${source.enabled ? "" : styles.disabled}`}
                    >
                      <div className={styles.sourceInfo}>
                        <span className={styles.sourceName}>{source.name} {source.enabled ? "" : "⏸"}</span>
                        <span className={styles.sourceType}>{source.source_type}</span>
                        {source.path && <span className={styles.sourcePath}>{source.path}</span>}
                        {source.url && <span className={styles.sourceUrl}>{source.url}</span>}
                      </div>
                      <div className={styles.sourceActions}>
                        <button className={`${styles.iconBtn} ${source.enabled ? "" : styles.active}`}
                          onClick={() => handleUpdateSource(source.id, { enabled: !source.enabled })}
                          title={source.enabled ? "Disable" : "Enable"}
                        >
                          {source.enabled ? "⏸" : "▶"}
                        </button>
                        <button className={`${styles.iconBtn} ${styles.danger}`} onClick={() => handleDeleteSource(source.id)} title="Delete">
                          🗑️
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          )}

          {selectedChatId === "instructions" && (
            <div className={styles.tabContent}>
              <div className={styles.instructionsList}>
                {projectInstructions.length === 0 ? (
                  <div className={styles.emptyState}>
                    <p>No instructions set</p>
                    <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowAddInstruction(true)}>
                      Add instruction
                    </button>
                  </div>
                ) : (
                  projectInstructions.map((ins) => (
                    <div
                      key={ins.id}
                      className={`${styles.instructionItem} ${ins.enabled ? "" : styles.disabled}`}
                    >
                      <div className={styles.instructionContent}>
                        <span className={styles.instructionStatus}>{ins.enabled ? "✓" : "○"}</span>
                        <pre>{ins.content}</pre>
                      </div>
                      <div className={styles.instructionActions}>
                        <button className={`${styles.iconBtn} ${ins.enabled ? "" : styles.active}`}
                          onClick={() => handleUpdateInstruction(ins.id, { enabled: !ins.enabled })}
                          title={ins.enabled ? "Disable" : "Enable"}
                        >
                          {ins.enabled ? "⏸" : "▶"}
                        </button>
                        <button className={`${styles.iconBtn} ${styles.danger}`} onClick={() => handleDeleteInstruction(ins.id)} title="Delete">
                          🗑️
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          )}

          {/* Create Project Modal */}
          {showCreateProject && (
            <div className={styles.modalOverlay} onClick={() => setShowCreateProject(false)}>
              <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
                <div className={styles.modalHeader}>
                  <h3>Create Project</h3>
                  <button className={styles.modalClose} onClick={() => setShowCreateProject(false)}>✕</button>
                </div>
                <div className={styles.modalBody}>
                  <p className={styles.hint}>Enter a folder path to scan, or fill in details manually</p>
                  <div className={styles.formGroup}>
                    <label>Folder Path</label>
                    <input
                      type="text"
                      value={newProjectPath}
                      onChange={(e) => setNewProjectPath(e.target.value)}
                      placeholder="/path/to/project"
                    />
                  </div>
                  <div className={styles.formGroup}>
                    <label>Project Name (auto-detected from folder)</label>
                    <input
                      type="text"
                      value={newProjectName}
                      onChange={(e) => setNewProjectName(e.target.value)}
                      placeholder="My Project"
                    />
                  </div>
                  <div className={styles.formGroup}>
                    <label>Type</label>
                    <select value={newProjectType} onChange={(e) => setNewProjectType(e.target.value)}>
                      <option value="local">Local Folder</option>
                      <option value="git">Git Repository</option>
                      <option value="rust">Rust (Cargo)</option>
                      <option value="node">Node.js</option>
                      <option value="python">Python</option>
                      <option value="go">Go</option>
                    </select>
                  </div>
                  <div className={styles.formGroup}>
                    <label>Description (optional)</label>
                    <textarea
                      value={newProjectDesc}
                      onChange={(e) => setNewProjectDesc(e.target.value)}
                      placeholder="Project description..."
                      rows={3}
                    />
                  </div>
                  <div className={styles.modalActions}>
                    <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowCreateProject(false)}>Cancel</button>
                    <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={handleScanProject}>Scan Folder</button>
                    <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={handleCreateProject}>Create Project</button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Create Chat Modal */}
          {showCreateChat && (
            <div className={styles.modalOverlay} onClick={() => setShowCreateChat(false)}>
              <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
                <div className={styles.modalHeader}>
                  <h3>New Chat</h3>
                  <button className={styles.modalClose} onClick={() => setShowCreateChat(false)}>✕</button>
                </div>
                <div className={styles.modalBody}>
                  <div className={styles.formGroup}>
                    <label>Chat Name</label>
                    <input
                      type="text"
                      value={newChatName}
                      onChange={(e) => setNewChatName(e.target.value)}
                      placeholder="New chat"
                      autoFocus
                    />
                  </div>
                  <div className={styles.modalActions}>
                    <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowCreateChat(false)}>Cancel</button>
                    <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={handleCreateChat}>Create Chat</button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Add Source Modal */}
          {showAddSource && (
            <div className={styles.modalOverlay} onClick={() => setShowAddSource(false)}>
              <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
                <div className={styles.modalHeader}>
                  <h3>Add Source</h3>
                  <button className={styles.modalClose} onClick={() => setShowAddSource(false)}>✕</button>
                </div>
                <div className={styles.modalBody}>
                  <div className={styles.formGroup}>
                    <label>Source Type</label>
                    <select value={newSourceType} onChange={(e) => setNewSourceType(e.target.value)}>
                      <option value="folder">Local Folder</option>
                      <option value="url">URL / Website</option>
                      <option value="github">GitHub Repository</option>
                      <option value="notion">Notion Page</option>
                    </select>
                  </div>
                  {(newSourceType === "folder" || newSourceType === "github") && (
                    <div className={styles.formGroup}>
                      <label>Path</label>
                      <input
                        type="text"
                        value={newSourcePath}
                        onChange={(e) => setNewSourcePath(e.target.value)}
                        placeholder="/path/to/folder"
                      />
                    </div>
                  )}
                  {(newSourceType === "url" || newSourceType === "notion") && (
                    <div className={styles.formGroup}>
                      <label>URL</label>
                      <input
                        type="url"
                        value={newSourceUrl}
                        onChange={(e) => setNewSourceUrl(e.target.value)}
                        placeholder="https://example.com"
                      />
                    </div>
                  )}
                  <div className={styles.formGroup}>
                    <label>Display Name</label>
                    <input
                      type="text"
                      value={newSourceName}
                      onChange={(e) => setNewSourceName(e.target.value)}
                      placeholder="My Source"
                    />
                  </div>
                  <div className={styles.modalActions}>
                    <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowAddSource(false)}>Cancel</button>
                    <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={handleCreateSource}>Add Source</button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Add Instruction Modal */}
          {showAddInstruction && (
            <div className={styles.modalOverlay} onClick={() => setShowAddInstruction(false)}>
              <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
                <div className={styles.modalHeader}>
                  <h3>Add Instruction</h3>
                  <button className={styles.modalClose} onClick={() => setShowAddInstruction(false)}>✕</button>
                </div>
                <div className={styles.modalBody}>
                  <div className={styles.formGroup}>
                    <label>Instruction Content</label>
                    <textarea
                      value={newInstruction}
                      onChange={(e) => setNewInstruction(e.target.value)}
                      placeholder="e.g., Always use TypeScript strict mode..."
                      rows={4}
                      autoFocus
                    />
                  </div>
                  <div className={styles.modalActions}>
                    <button className={`${styles.btn} ${styles.btnSecondary}`} onClick={() => setShowAddInstruction(false)}>Cancel</button>
                    <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={handleCreateInstruction}>Add Instruction</button>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );

  // Helper functions for chat/source/instruction actions
  const handleUpdateChat = async (chatId: string, updates: Partial<ProjectChat>) => {
    try {
      const chat = await invoke<ProjectChat>("project_chat_update", { chatId, ...updates });
      setChats((prev) => ({
        ...prev,
        [selectedProjectId!]: prev[selectedProjectId!]?.map((c) => (c.id === chatId ? chat : c)) || [],
      }));
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleDeleteChat = async (chatId: string) => {
    if (!window.confirm("Delete this chat?")) return;
    try {
      await invoke("project_chat_delete", { chatId });
      setChats((prev) => ({
        ...prev,
        [selectedProjectId!]: prev[selectedProjectId!]?.filter((c) => c.id !== chatId) || [],
      }));
      showStatus("Chat deleted");
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleUpdateSource = async (sourceId: string, updates: Partial<ProjectSource>) => {
    try {
      const source = await invoke<ProjectSource>("project_source_update", { sourceId, ...updates });
      setSources((prev) => ({
        ...prev,
        [selectedProjectId!]: prev[selectedProjectId!]?.map((s) => (s.id === sourceId ? source : s)) || [],
      }));
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleDeleteSource = async (sourceId: string) => {
    if (!window.confirm("Delete this source?")) return;
    try {
      await invoke("project_source_delete", { sourceId });
      setSources((prev) => ({
        ...prev,
        [selectedProjectId!]: prev[selectedProjectId!]?.filter((s) => s.id !== sourceId) || [],
      }));
      showStatus("Source deleted");
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleUpdateInstruction = async (instructionId: string, updates: Partial<ProjectInstruction>) => {
    try {
      const ins = await invoke<ProjectInstruction>("project_instruction_update", { instructionId, ...updates });
      setInstructions((prev) => ({
        ...prev,
        [selectedProjectId!]: prev[selectedProjectId!]?.map((i) => (i.id === instructionId ? ins : i)) || [],
      }));
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };

  const handleDeleteInstruction = async (instructionId: string) => {
    if (!window.confirm("Delete this instruction?")) return;
    try {
      await invoke("project_instruction_delete", { instructionId });
      setInstructions((prev) => ({
        ...prev,
        [selectedProjectId!]: prev[selectedProjectId!]?.filter((i) => i.id !== instructionId) || [],
      }));
      showStatus("Instruction deleted");
    } catch (e) {
      showStatus(`Error: ${e}`);
    }
  };
};

export default ProjectsPage;