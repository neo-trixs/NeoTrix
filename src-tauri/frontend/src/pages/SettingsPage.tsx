import React from "react";
import { useNavigate } from "react-router-dom";
import { SettingsView } from "../components/neocodex";
import styles from "./SettingsPage.module.css";

const SettingsPage: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className={styles.overlay} onClick={() => navigate("/")}>
      <div className={styles.modal} role="dialog" aria-modal="true" aria-label="Settings" onClick={(e) => e.stopPropagation()}>
        <button className={styles.close} onClick={() => navigate("/")} aria-label="Close settings">
          <svg viewBox="0 0 16 16">
            <path d="M4 4l8 8M12 4l-8 8" />
          </svg>
        </button>
        <SettingsView />
      </div>
    </div>
  );
};

export default SettingsPage;
