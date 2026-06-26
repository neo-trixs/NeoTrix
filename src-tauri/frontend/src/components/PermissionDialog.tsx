import React from "react";
import type { PermissionRequest } from "../types";

interface Props {
  request: PermissionRequest;
  onApprove: (id: string) => void;
  onDeny: (id: string) => void;
  onModify?: (id: string, modifiedAction: string) => void;
}

/**
 * PermissionDialog — modal for approving/denying agent-initiated actions.
 * Displays action type, target path, and details with approve/deny/modify controls.
 */
const PermissionDialog: React.FC<Props> = ({ request, onApprove, onDeny, onModify }) => {
  return (
    <div className="permission-overlay" onClick={() => onDeny(request.id)}>
      <div className="permission-dialog glass-panel" onClick={(e) => e.stopPropagation()}>
        <div className="permission-header">
          <span className="permission-icon">
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
              <path d="M10 2a3 3 0 00-3 3v1H5a1 1 0 00-1 1v8a1 1 0 001 1h10a1 1 0 001-1V7a1 1 0 00-1-1h-2V5a3 3 0 00-3-3z" stroke="currentColor" strokeWidth="1.5" />
              <circle cx="10" cy="12" r="1.5" fill="currentColor" />
            </svg>
          </span>
          <span>权限请求</span>
        </div>

        <div className="permission-body">
          <div className="permission-row">
            <span className="permission-label">操作</span>
            <span className="permission-value">{request.action}</span>
          </div>
          <div className="permission-row">
            <span className="permission-label">目标</span>
            <code className="permission-value permission-target">{request.target}</code>
          </div>
          <div className="permission-row">
            <span className="permission-label">详情</span>
            <span className="permission-value">{request.details}</span>
          </div>
        </div>

        <div className="permission-footer">
          {onModify && (
            <button className="btn-secondary" onClick={() => onModify(request.id, request.action)}>
              修改
            </button>
          )}
          <button className="btn-secondary" onClick={() => onDeny(request.id)}>拒绝</button>
          <button className="btn-primary" onClick={() => onApprove(request.id)}>允许</button>
        </div>
      </div>
    </div>
  );
};

export default PermissionDialog;
