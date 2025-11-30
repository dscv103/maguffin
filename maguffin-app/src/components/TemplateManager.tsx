import { useState } from "react";
import { useTemplates } from "../hooks/useTemplates";
import type { PrTemplate } from "../types";

interface TemplateManagerProps {
  onSelectTemplate?: (template: PrTemplate) => void;
}

export function TemplateManager({ onSelectTemplate }: TemplateManagerProps) {
  const {
    templates,
    loading,
    error,
    createTemplate,
    updateTemplate,
    deleteTemplate,
  } = useTemplates();

  const [isEditing, setIsEditing] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<PrTemplate | null>(null);
  const [name, setName] = useState("");
  const [body, setBody] = useState("");
  const [isDefault, setIsDefault] = useState(false);
  const [saving, setSaving] = useState(false);

  const handleCreate = () => {
    setEditingTemplate(null);
    setName("");
    setBody("");
    setIsDefault(false);
    setIsEditing(true);
  };

  const handleEdit = (template: PrTemplate) => {
    setEditingTemplate(template);
    setName(template.name);
    setBody(template.body);
    setIsDefault(template.is_default);
    setIsEditing(true);
  };

  const handleSave = async () => {
    if (!name.trim()) return;

    setSaving(true);
    try {
      if (editingTemplate) {
        await updateTemplate(editingTemplate.id, name, body, isDefault);
      } else {
        await createTemplate(name, body, isDefault);
      }
      setIsEditing(false);
      setEditingTemplate(null);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (confirm("Are you sure you want to delete this template?")) {
      await deleteTemplate(id);
    }
  };

  const handleCancel = () => {
    setIsEditing(false);
    setEditingTemplate(null);
  };

  if (loading) {
    return <div className="template-manager loading">Loading templates...</div>;
  }

  if (error) {
    return <div className="template-manager error">Error: {error}</div>;
  }

  if (isEditing) {
    return (
      <div className="template-manager editing">
        <h3>{editingTemplate ? "Edit Template" : "Create Template"}</h3>
        <div className="template-form">
          <div className="form-group">
            <label htmlFor="template-name">Name</label>
            <input
              id="template-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Template name"
            />
          </div>
          <div className="form-group">
            <label htmlFor="template-body">Body</label>
            <textarea
              id="template-body"
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder="Template content. Use {{branch}}, {{author}}, {{date}}, {{title}} as placeholders."
              rows={12}
            />
            <small className="placeholder-hint">
              Available placeholders: {"{{branch}}"}, {"{{author}}"}, {"{{date}}"}, {"{{title}}"}
            </small>
          </div>
          <div className="form-group checkbox">
            <label>
              <input
                type="checkbox"
                checked={isDefault}
                onChange={(e) => setIsDefault(e.target.checked)}
              />
              Set as default template
            </label>
          </div>
          <div className="form-actions">
            <button onClick={handleCancel} disabled={saving}>
              Cancel
            </button>
            <button onClick={handleSave} disabled={saving || !name.trim()} className="primary">
              {saving ? "Saving..." : "Save"}
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="template-manager">
      <div className="template-header">
        <h3>PR Templates</h3>
        <button onClick={handleCreate} className="primary">
          + New Template
        </button>
      </div>

      {templates.length === 0 ? (
        <div className="template-empty">
          <p>No templates yet. Create one to get started.</p>
        </div>
      ) : (
        <ul className="template-list">
          {templates.map((template) => (
            <li key={template.id} className="template-item">
              <div className="template-info">
                <span className="template-name">
                  {template.name}
                  {template.is_default && (
                    <span className="default-badge">Default</span>
                  )}
                </span>
                <span className="template-preview">
                  {template.body.slice(0, 100)}
                  {template.body.length > 100 ? "..." : ""}
                </span>
              </div>
              <div className="template-actions">
                {onSelectTemplate && (
                  <button onClick={() => onSelectTemplate(template)} className="select-btn">
                    Use
                  </button>
                )}
                <button onClick={() => handleEdit(template)}>Edit</button>
                <button onClick={() => handleDelete(template.id)} className="danger">
                  Delete
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
