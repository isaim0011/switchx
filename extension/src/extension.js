const vscode = require('vscode');
const { execFile, exec } = require('child_process');
const path = require('path');
const fs = require('fs');

/**
 * Antigravity & Universal AI Account Switcher Provider
 * Connects directly to real profile state in Antigravity IDE, Cursor, and system tools
 */
class AccountSwitcherViewProvider {
  constructor(extensionUri) {
    this._extensionUri = extensionUri;
    this._view = undefined;
    this._scriptPath = path.join(this._extensionUri.fsPath, 'scripts', 'profile_manager.ps1');
  }

  resolveWebviewView(webviewView, context, _token) {
    this._view = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this._extensionUri]
    };

    webviewView.webview.html = this._getHtmlForWebview(webviewView.webview);

    webviewView.webview.onDidReceiveMessage(async (message) => {
      switch (message.command) {
        case 'refresh':
          await this.loadProfiles();
          break;
        case 'saveCurrentProfile':
          await this.promptSaveProfile();
          break;
        case 'switchProfile':
          await this.switchProfile(message.name);
          break;
        case 'deleteProfile':
          await this.deleteProfile(message.name);
          break;
      }
    });

    this.loadProfiles();
  }

  async runPowerShell(args) {
    return new Promise((resolve) => {
      const psCommand = `powershell.exe -ExecutionPolicy Bypass -NoProfile -File "${this._scriptPath}" ${args}`;
      exec(psCommand, (error, stdout, stderr) => {
        if (error) {
          resolve({ success: false, error: stderr || error.message });
        } else {
          try {
            const data = JSON.parse(stdout.trim());
            resolve({ success: true, data });
          } catch (e) {
            resolve({ success: true, raw: stdout.trim() });
          }
        }
      });
    });
  }

  async loadProfiles() {
    if (!this._view) return;

    const res = await this.runPowerShell('-Action List');
    let profilesList = [];
    if (res.success && res.data && res.data.Profiles) {
      profilesList = res.data.Profiles;
    }

    this._view.webview.postMessage({
      command: 'updateProfiles',
      profiles: profilesList
    });
  }

  async promptSaveProfile() {
    const name = await vscode.window.showInputBox({
      prompt: 'Enter a profile name for this active Antigravity session (e.g. Work, Personal, Alt1)',
      validateInput: (val) => {
        if (!val || val.trim().length === 0) return 'Name cannot be empty';
        if (/[\\/:*?"<>|]/.test(val)) return 'Invalid characters in name';
        return null;
      }
    });

    if (!name) return;

    vscode.window.withProgress({
      location: vscode.ProgressLocation.Notification,
      title: `Saving current account profile as '${name}'...`,
      cancellable: false
    }, async () => {
      const res = await this.runPowerShell(`-Action Save -ProfileName "${name.trim()}"`);
      if (res.success) {
        vscode.window.showInformationMessage(`Account profile '${name}' saved successfully!`);
        await this.loadProfiles();
      } else {
        vscode.window.showErrorMessage(`Failed to save profile: ${res.error}`);
      }
    });
  }

  async switchProfile(name) {
    const confirm = await vscode.window.showWarningMessage(
      `Switch to profile '${name}'? Antigravity will restart to apply authentication session.`,
      { modal: true },
      'Switch & Restart'
    );

    if (confirm !== 'Switch & Restart') return;

    vscode.window.withProgress({
      location: vscode.ProgressLocation.Notification,
      title: `Loading profile '${name}' and restarting Antigravity...`,
      cancellable: false
    }, async () => {
      await this.runPowerShell(`-Action Load -ProfileName "${name}"`);
    });
  }

  async deleteProfile(name) {
    const confirm = await vscode.window.showWarningMessage(
      `Are you sure you want to delete saved profile '${name}'?`,
      { modal: true },
      'Delete'
    );

    if (confirm !== 'Delete') return;

    const res = await this.runPowerShell(`-Action Delete -ProfileName "${name}"`);
    if (res.success) {
      vscode.window.showInformationMessage(`Profile '${name}' deleted.`);
      await this.loadProfiles();
    } else {
      vscode.window.showErrorMessage(`Failed to delete profile: ${res.error}`);
    }
  }

  _getHtmlForWebview(webview) {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Antigravity Account Switcher</title>
  <style>
    :root {
      --bg: #090d16;
      --card-bg: rgba(22, 30, 49, 0.7);
      --card-border: rgba(255, 255, 255, 0.08);
      --card-hover: rgba(30, 41, 67, 0.85);
      --text-main: #f1f5f9;
      --text-muted: #94a3b8;
      --accent-blue: #38bdf8;
      --accent-green: #34d399;
      --accent-rose: #f43f5e;
      --font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }

    body {
      background: radial-gradient(circle at 10% 10%, #0d1527 0%, var(--bg) 100%);
      color: var(--text-main);
      font-family: var(--font-family);
      font-size: 12px;
      padding: 12px;
      line-height: 1.4;
      user-select: none;
    }

    .header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 14px;
      padding-bottom: 10px;
      border-bottom: 1px solid var(--card-border);
    }

    .title-group {
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .logo-badge {
      width: 24px;
      height: 24px;
      background: linear-gradient(135deg, #0284c7, #38bdf8);
      border-radius: 6px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: 800;
      font-size: 12px;
      color: #fff;
      box-shadow: 0 0 10px rgba(56, 189, 248, 0.3);
    }

    .title {
      font-size: 13px;
      font-weight: 700;
      letter-spacing: 0.5px;
      background: linear-gradient(90deg, #fff, #94a3b8);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }

    .btn-row {
      display: flex;
      gap: 6px;
    }

    .btn {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      color: var(--text-main);
      padding: 4px 8px;
      border-radius: 5px;
      cursor: pointer;
      font-size: 11px;
      font-weight: 600;
      display: inline-flex;
      align-items: center;
      gap: 4px;
      transition: all 0.15s ease;
      backdrop-filter: blur(8px);
    }

    .btn:hover {
      background: var(--card-hover);
      border-color: rgba(255, 255, 255, 0.2);
    }

    .btn-primary {
      background: linear-gradient(135deg, #0284c7, #2563eb);
      border: 1px solid rgba(255, 255, 255, 0.15);
      color: #fff;
    }

    .btn-primary:hover {
      background: linear-gradient(135deg, #0369a1, #1d4ed8);
      box-shadow: 0 0 8px rgba(37, 99, 235, 0.4);
    }

    .banner {
      background: rgba(56, 189, 248, 0.08);
      border: 1px solid rgba(56, 189, 248, 0.2);
      border-radius: 6px;
      padding: 10px;
      margin-bottom: 12px;
      font-size: 11px;
      color: var(--text-muted);
      line-height: 1.4;
    }

    .banner strong {
      color: #fff;
    }

    .card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 8px;
      padding: 12px;
      margin-bottom: 10px;
      backdrop-filter: blur(12px);
      transition: border-color 0.2s;
    }

    .card:hover {
      border-color: rgba(255, 255, 255, 0.18);
    }

    .card-top {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 8px;
    }

    .card-title {
      font-size: 13px;
      font-weight: 700;
      color: #fff;
    }

    .card-meta {
      font-size: 10.5px;
      color: var(--text-muted);
      margin-bottom: 10px;
    }

    .card-actions {
      display: flex;
      gap: 6px;
      justify-content: flex-end;
    }

    .empty-state {
      text-align: center;
      padding: 30px 10px;
      color: var(--text-muted);
    }
  </style>
</head>
<body>
  <div class="header">
    <div class="title-group">
      <div class="logo-badge">AG</div>
      <div class="title">ACCOUNT SWITCHER</div>
    </div>
    <div class="btn-row">
      <button class="btn btn-primary" id="btn-save">+ Save Current</button>
      <button class="btn" id="btn-refresh">↻</button>
    </div>
  </div>

  <div class="banner">
    <strong>Real Session Manager:</strong> Log into any Google Account in Antigravity, click <strong>+ Save Current</strong>, and switch between your accounts anytime without repeated browser logins.
  </div>

  <div id="profiles-container">
    <div class="empty-state">Loading saved profiles...</div>
  </div>

  <script>
    const vscode = acquireVsCodeApi();

    document.getElementById('btn-refresh').addEventListener('click', () => {
      vscode.postMessage({ command: 'refresh' });
    });

    document.getElementById('btn-save').addEventListener('click', () => {
      vscode.postMessage({ command: 'saveCurrentProfile' });
    });

    window.switchProfile = (name) => {
      vscode.postMessage({ command: 'switchProfile', name });
    };

    window.deleteProfile = (name) => {
      vscode.postMessage({ command: 'deleteProfile', name });
    };

    window.addEventListener('message', event => {
      const msg = event.data;
      if (msg.command === 'updateProfiles') {
        renderProfiles(msg.profiles || []);
      }
    });

    function renderProfiles(profiles) {
      const container = document.getElementById('profiles-container');
      container.innerHTML = '';

      if (profiles.length === 0) {
        container.innerHTML = '<div class="empty-state">No saved profiles yet.<br><br>Log into an account and click <strong>+ Save Current</strong> above.</div>';
        return;
      }

      profiles.forEach(p => {
        const card = document.createElement('div');
        card.className = 'card';
        card.innerHTML = \`
          <div class="card-top">
            <span class="card-title">\${p.Name}</span>
            <span style="font-size:10px; background:rgba(52,211,153,0.15); color:var(--accent-green); padding:2px 6px; border-radius:4px; font-weight:700;">READY</span>
          </div>
          <div class="card-meta">Saved: \${p.Created} • Size: \${p.Size} MB</div>
          <div class="card-actions">
            <button class="btn" style="color:var(--accent-rose);" onclick="deleteProfile('\${p.Name}')">Delete</button>
            <button class="btn btn-primary" onclick="switchProfile('\${p.Name}')">Switch to Profile</button>
          </div>
        \`;
        container.appendChild(card);
      });
    }
  </script>
</body>
</html>`;
  }
}

function activate(context) {
  const provider = new AccountSwitcherViewProvider(context.extensionUri);

  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider('switchx.dashboardView', provider)
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('switchx.openDashboard', () => {
      vscode.commands.executeCommand('switchx.dashboardView.focus');
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('switchx.saveProfile', () => {
      provider.promptSaveProfile();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('switchx.refresh', () => {
      provider.loadProfiles();
    })
  );
}

function deactivate() {}

module.exports = {
  activate,
  deactivate
};
