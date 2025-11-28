import { AVAILABLE_SHORTCUTS } from "../hooks/useKeyboardShortcuts";

interface KeyboardShortcutsHelpProps {
  onClose: () => void;
}

export function KeyboardShortcutsHelp({ onClose }: KeyboardShortcutsHelpProps) {
  return (
    <div className="keyboard-shortcuts-overlay" onClick={onClose}>
      <div
        className="keyboard-shortcuts-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <header>
          <h2>Keyboard Shortcuts</h2>
          <button onClick={onClose} aria-label="Close">
            ×
          </button>
        </header>
        <div className="shortcuts-list">
          {AVAILABLE_SHORTCUTS.map((shortcut) => (
            <div key={shortcut.key} className="shortcut-item">
              <kbd className="shortcut-key">{shortcut.key}</kbd>
              <span className="shortcut-description">{shortcut.description}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
