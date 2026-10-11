const vscode = require('vscode');
const { execFile } = require('child_process');
const path = require('path');
const fs = require('fs');

/**
 * SwitchX Webview View Provider for Activity Bar Sidebar
 */
class SwitchXViewProvider {
  constructor(extensionUri) {
    this._extensionUri = extensionUri;
    this._view = undefined;
    this._binaryPath = this._locateBinary();
  }

  _locateBinary() {
    // 1. Check local release/debug build
    const devPath = path.resolve(__dirname, '..', '..', 'target', 'debug', 'switchx.exe');
    if (fs.existsSync(devPath)) {
      return devPath;
    }
    const releasePath = path.resolve(__dirname, '..', '..', 'target', 'release', 'switchx.exe');
    if (fs.existsSync(releasePath)) {
      return releasePath;
    }
    // Fallback to system PATH
    return 'switchx';
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
          await this.refreshData();
          break;
        case 'switchAccount':
          await this.switchAccount(message.provider, message.name);
          break;
        case 'autoNext':
          await this.autoNext(message.provider);
          break;
        case 'addAccount':
          await this.promptAddAccount();
          break;
      }
    });

    // Initial load
    this.refreshData();
  }

  async runCli(args) {
    return new Promise((resolve) => {
      execFile(this._binaryPath, args, (error, stdout, stderr) => {
        if (error) {
          resolve({ success: false, error: stderr || error.message });
        } else {
          resolve({ success: true, output: stdout });
        }
      });
    });
  }

  async refreshData() {
    if (!this._view) return;

    // Fetch accounts and snapshots directly from the Rust engine
    const listRes = await this.runCli(['list']);
    const statusRes = await this.runCli(['status']);

    this._view.webview.postMessage({
      command: 'updateData',
      listOutput: listRes.output || '',
      statusOutput: statusRes.output || '',
      isError: !listRes.success
    });
  }

  async switchAccount(provider, name) {
    const res = await this.runCli(['switch', provider, name]);
    if (res.success) {
      vscode.window.showInformationMessage(`SwitchX: Switched ${provider} session to ${name}!`);
      await this.refreshData();
    } else {
      vscode.window.showErrorMessage(`SwitchX error: ${res.error}`);
    }
  }

  async autoNext(provider) {
    const res = await this.runCli(['auto-next', provider]);
    if (res.success) {
      vscode.window.showInformationMessage(`SwitchX: Auto-failover activated for ${provider}!`);
      await this.refreshData();
    } else {
      vscode.window.showErrorMessage(`SwitchX error: ${res.error}`);
    }
  }

  async promptAddAccount() {
    const provider = await vscode.window.showQuickPick(
      [
        { label: 'antigravity', description: 'Google Antigravity & Cursor' },
        { label: 'claude-code', description: 'Claude Code CLI' },
        { label: 'copilot', description: 'GitHub Copilot / Codex' }
      ],
      { placeHolder: 'Select AI Provider to add account profile for' }
    );
    if (!provider) return;

    const name = await vscode.window.showInputBox({
      prompt: `Enter account nickname for ${provider.label} (e.g. work, personal, alt1)`
    });
    if (!name) return;

    const email = await vscode.window.showInputBox({
      prompt: `Enter optional email address for identification`
    });

    const args = ['add', provider.label, name];
    if (email) {
      args.push('--email', email);
    }

    const res = await this.runCli(args);
    if (res.success) {
      vscode.window.showInformationMessage(`SwitchX: Account '${name}' added for ${provider.label}!`);
      await this.refreshData();
    } else {
      vscode.window.showErrorMessage(`SwitchX error: ${res.error}`);
    }
  }

  _getHtmlForWebview(webview) {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>SwitchX Pro</title>
  <style>
    :root {
      --bg: #090d16;
      --card-bg: rgba(22, 30, 49, 0.7);
      --card-border: rgba(255, 255, 255, 0.08);
      --card-hover: rgba(30, 41, 67, 0.85);
      --text-main: #f1f5f9;
      --text-muted: #94a3b8;
      --accent-blue: #38bdf8;
      --accent-purple: #c084fc;
      --accent-green: #34d399;
      --accent-amber: #fbbf24;
      --accent-rose: #f43f5e;
      --font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    }

    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }

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
      width: 22px;
      height: 22px;
      background: linear-gradient(135deg, var(--accent-blue), var(--accent-purple));
      border-radius: 6px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: 800;
      font-size: 11px;
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
      font-weight: 500;
      display: inline-flex;
      align-items: center;
      gap: 4px;
      transition: all 0.15s ease;
      backdrop-filter: blur(8px);
    }

    .btn:hover {
      background: var(--card-hover);
      border-color: rgba(255, 255, 255, 0.2);
      transform: translateY(-1px);
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

    .tab-bar {
      display: flex;
      gap: 4px;
      margin-bottom: 12px;
      background: rgba(15, 23, 42, 0.6);
      padding: 3px;
      border-radius: 6px;
      border: 1px solid var(--card-border);
    }

    .tab {
      flex: 1;
      text-align: center;
      padding: 5px 0;
      border-radius: 4px;
      cursor: pointer;
      font-size: 11px;
      font-weight: 600;
      color: var(--text-muted);
      transition: all 0.15s;
    }

    .tab.active {
      background: var(--card-hover);
      color: #fff;
      box-shadow: 0 1px 3px rgba(0,0,0,0.3);
    }

    .card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 8px;
      padding: 12px;
      margin-bottom: 10px;
      backdrop-filter: blur(12px);
      transition: border-color 0.2s, transform 0.2s;
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
      font-size: 12px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 6px;
    }

    .badge {
      font-size: 9px;
      font-weight: 700;
      padding: 2px 6px;
      border-radius: 4px;
      letter-spacing: 0.4px;
      text-transform: uppercase;
    }

    .badge-optimal {
      background: rgba(52, 211, 153, 0.15);
      color: var(--accent-green);
      border: 1px solid rgba(52, 211, 153, 0.3);
    }

    .badge-critical {
      background: rgba(244, 63, 94, 0.15);
      color: var(--accent-rose);
      border: 1px solid rgba(244, 63, 94, 0.3);
    }

    .badge-active {
      background: rgba(56, 189, 248, 0.15);
      color: var(--accent-blue);
      border: 1px solid rgba(56, 189, 248, 0.3);
    }

    .progress-section {
      margin: 8px 0;
    }

    .progress-label-row {
      display: flex;
      justify-content: space-between;
      margin-bottom: 4px;
      font-size: 10.5px;
      color: var(--text-muted);
    }

    .progress-bar-bg {
      height: 6px;
      background: rgba(255, 255, 255, 0.08);
      border-radius: 3px;
      overflow: hidden;
      position: relative;
    }

    .progress-bar-fill {
      height: 100%;
      border-radius: 3px;
      transition: width 0.4s ease;
    }

    .fill-green {
      background: linear-gradient(90deg, #10b981, #34d399);
      box-shadow: 0 0 6px rgba(52, 211, 153, 0.4);
    }

    .fill-red {
      background: linear-gradient(90deg, #e11d48, #f43f5e);
      box-shadow: 0 0 6px rgba(244, 63, 94, 0.4);
    }

    .card-footer {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-top: 10px;
      padding-top: 8px;
      border-top: 1px solid rgba(255, 255, 255, 0.05);
      font-size: 10.5px;
      color: var(--text-muted);
    }

    .switch-action {
      background: rgba(255, 255, 255, 0.06);
      border: 1px solid var(--card-border);
      color: var(--text-main);
      font-size: 10px;
      font-weight: 600;
      padding: 3px 8px;
      border-radius: 4px;
      cursor: pointer;
      transition: all 0.15s;
    }

    .switch-action:hover {
      background: var(--accent-blue);
      color: #000;
      border-color: var(--accent-blue);
    }

    .active-indicator {
      display: flex;
      align-items: center;
      gap: 4px;
      color: var(--accent-blue);
      font-weight: 600;
    }

    .dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      background: var(--accent-blue);
      box-shadow: 0 0 6px var(--accent-blue);
    }
  </style>
</head>
<body>
  <div class="header">
    <div class="title-group">
      <div class="logo-badge">SX</div>
      <div class="title">SWITCHX PRO</div>
    </div>
    <div class="btn-row">
      <button class="btn" id="btn-add">+ Add</button>
      <button class="btn" id="btn-refresh">↻</button>
    </div>
  </div>

  <div class="tab-bar">
    <div class="tab active" data-filter="all">All Apps</div>
    <div class="tab" data-filter="claude-code">Claude</div>
    <div class="tab" data-filter="antigravity">Antigravity</div>
    <div class="tab" data-filter="copilot">Copilot</div>
  </div>

  <div id="accounts-container">
    <!-- Dynamic Cards injected via script -->
  </div>

  <script>
    const vscode = acquireVsCodeApi();
    let currentFilter = 'all';

    document.getElementById('btn-refresh').addEventListener('click', () => {
      vscode.postMessage({ command: 'refresh' });
    });

    document.getElementById('btn-add').addEventListener('click', () => {
      vscode.postMessage({ command: 'addAccount' });
    });

    document.querySelectorAll('.tab').forEach(tab => {
      tab.addEventListener('click', () => {
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        currentFilter = tab.dataset.filter;
        renderCards();
      });
    });

    // Mock/Render State
    let accounts = [
      {
        provider: 'claude-code',
        name: 'work',
        email: 'work@corp.com',
        isActive: false,
        used5h: 95,
        total5h: 100,
        pct5h: 5.0,
        reset5h: 'Rolling freely',
        health: 'critical'
      },
      {
        provider: 'claude-code',
        name: 'personal',
        email: 'personal@gmail.com',
        isActive: true,
        used5h: 0,
        total5h: 100,
        pct5h: 100.0,
        reset5h: 'Full capacity',
        health: 'optimal'
      },
      {
        provider: 'antigravity',
        name: 'primary',
        email: 'dev@gmail.com',
        isActive: true,
        used5h: 12,
        total5h: 50,
        pct5h: 76.0,
        reset5h: 'Rolling freely',
        health: 'optimal'
      },
      {
        provider: 'copilot',
        name: 'dev-org',
        email: 'github@company.com',
        isActive: true,
        used5h: 20,
        total5h: 150,
        pct5h: 86.6,
        reset5h: 'Full capacity',
        health: 'optimal'
      }
    ];

    function renderCards() {
      const container = document.getElementById('accounts-container');
      container.innerHTML = '';

      const filtered = accounts.filter(a => currentFilter === 'all' || a.provider === currentFilter);

      if (filtered.length === 0) {
        container.innerHTML = '<div style="text-align:center; padding:20px; color:#64748b;">No accounts found for this category.</div>';
        return;
      }

      filtered.forEach(acc => {
        const card = document.createElement('div');
        card.className = 'card';

        const isCrit = acc.health === 'critical';
        const badgeClass = isCrit ? 'badge-critical' : 'badge-optimal';
        const fillClass = isCrit ? 'fill-red' : 'fill-green';

        card.innerHTML = \`
          <div class="card-top">
            <div class="card-title">
              <span>\${acc.name}</span>
              <span class="badge \${badgeClass}">\${acc.health}</span>
            </div>
            \${acc.isActive ? '<div class="active-indicator"><span class="dot"></span>ACTIVE</div>' : ''}
          </div>

          <div style="font-size:10.5px; color:#64748b; margin-bottom:8px;">
            \${acc.provider} • \${acc.email || 'No email'}
          </div>

          <div class="progress-section">
            <div class="progress-label-row">
              <span>5-Hour Rolling Sprint</span>
              <span>\${acc.used5h}/\${acc.total5h} (\${acc.pct5h.toFixed(1)}%)</span>
            </div>
            <div class="progress-bar-bg">
              <div class="progress-bar-fill \${fillClass}" style="width: \${acc.pct5h}%;"></div>
            </div>
          </div>

          <div class="card-footer">
            <span>Reset: \${acc.reset5h}</span>
            \${acc.isActive 
              ? '<button class="switch-action" style="background:#0284c7; color:#fff;" onclick="handleAutoNext(\\'\${acc.provider}\\')">⚡ Failover</button>'
              : \`<button class="switch-action" onclick="handleSwitch('\\'\${acc.provider}\\', \\'\${acc.name}\\')">Switch Active</button>\`
            }
          </div>
        \`;

        container.appendChild(card);
      });
    }

    window.handleSwitch = (provider, name) => {
      vscode.postMessage({ command: 'switchAccount', provider, name });
    };

    window.handleAutoNext = (provider) => {
      vscode.postMessage({ command: 'autoNext', provider });
    };

    window.addEventListener('message', event => {
      const msg = event.data;
      if (msg.command === 'updateData') {
        renderCards();
      }
    });

    renderCards();
  </script>
</body>
</html>`;
  }
}

function activate(context) {
  const provider = new SwitchXViewProvider(context.extensionUri);

  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider('switchx.dashboardView', provider)
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('switchx.openDashboard', () => {
      vscode.commands.executeCommand('switchx.dashboardView.focus');
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('switchx.autoNext', async () => {
      const target = await vscode.window.showQuickPick(['claude-code', 'antigravity', 'copilot'], {
        placeHolder: 'Select provider for automatic failover'
      });
      if (target) {
        await provider.autoNext(target);
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('switchx.refresh', async () => {
      await provider.refreshData();
    })
  );
}

function deactivate() {}

module.exports = {
  activate,
  deactivate
};
