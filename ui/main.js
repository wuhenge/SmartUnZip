const invoke = window.__TAURI__?.core?.invoke;
const openDialog = window.__TAURI__?.dialog?.open;

let config = null;
let originalConfig = null;

function updateSaveButton() {
    const saveBtn = document.getElementById('btn-save');
    const currentConfig = collectForm();
    const hasChanges = JSON.stringify(currentConfig) !== JSON.stringify(originalConfig);
    saveBtn.disabled = !hasChanges;
}

function initTheme() {
    const savedTheme = localStorage.getItem('theme') || 'dark';
    setTheme(savedTheme);
}

function setTheme(theme) {
    const root = document.documentElement;
    const darkIcon = document.getElementById('theme-icon-dark');
    const lightIcon = document.getElementById('theme-icon-light');
    const themeText = document.getElementById('theme-text');
    
    if (theme === 'light') {
        root.setAttribute('data-theme', 'light');
        darkIcon.style.display = 'none';
        lightIcon.style.display = 'block';
        themeText.textContent = '深色';
    } else {
        root.removeAttribute('data-theme');
        darkIcon.style.display = 'block';
        lightIcon.style.display = 'none';
        themeText.textContent = '浅色';
    }
    
    localStorage.setItem('theme', theme);
}

function toggleTheme() {
    const currentTheme = localStorage.getItem('theme') || 'dark';
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
}

function showError(message) {
    const toast = document.getElementById('toast');
    const toastMessage = document.getElementById('toast-message');
    toastMessage.textContent = message;
    toast.className = 'toast show';
    setTimeout(() => {
        toast.className = 'toast';
    }, 5000);
}

async function init() {
    if (!invoke) {
        showError('Tauri API 不可用');
        return;
    }

    try {
        const configPath = await invoke('get_config_path');
        document.getElementById('config-path').textContent = configPath;

        config = await invoke('load_config');
        console.log('Loaded config:', config);
        populateForm(config);
        originalConfig = collectForm();
        document.getElementById('btn-save').disabled = true;

        // 初始化右键菜单状态
        await initContextMenuStatus();
    } catch (e) {
        console.error('加载配置失败:', e);
        showError('加载配置失败: ' + e);
    }
}

// 右键菜单管理
let contextMenuRegistered = false;
let isTogglingContextMenu = false;

async function initContextMenuStatus() {
    try {
        contextMenuRegistered = await invoke('check_context_menu');
        document.getElementById('context-menu-toggle').checked = contextMenuRegistered;
    } catch (e) {
        console.error('检查右键菜单状态失败:', e);
    }
}

async function onContextMenuToggle() {
    if (isTogglingContextMenu) return;
    
    const toggle = document.getElementById('context-menu-toggle');
    const targetState = toggle.checked;
    
    isTogglingContextMenu = true;
    toggle.disabled = true;

    try {
        if (targetState) {
            await invoke('add_context_menu');
            contextMenuRegistered = true;
            showToast('右键菜单已启用');
        } else {
            await invoke('remove_context_menu');
            contextMenuRegistered = false;
            showToast('右键菜单已禁用');
        }
    } catch (e) {
        console.error('操作失败:', e);
        showToast('操作失败: ' + e, false);
        // 恢复开关状态
        toggle.checked = contextMenuRegistered;
    } finally {
        isTogglingContextMenu = false;
        toggle.disabled = false;
    }
}

function populateForm(config) {
    document.getElementById('seven-zip-path').value = config.SevenZipPath || '';
    document.getElementById('nested-depth').value = config.NestedArchiveDepth || 0;
    document.getElementById('auto-exit').checked = config.AutoExit || false;
    document.getElementById('extract-nested').checked = config.ExtractNestedFolders || false;
    document.getElementById('delete-empty').checked = config.DeleteEmptyFolders || false;
    document.getElementById('folder-threshold').value = config.CreateFolderThreshold || 1;
    document.getElementById('flatten-wrapper').checked = config.FlattenWrapperFolder || false;
    document.getElementById('delete-source').checked = config.DeleteSourceAfterExtract || false;
    document.getElementById('open-folder').checked = config.OpenFolderAfterExtract || false;
    document.getElementById('debug-mode').checked = config.DebugMode || false;
    
    renderList('passwords-list', config.Passwords || [], 'password');
    renderList('delete-files-list', config.DeleteFiles || [], 'file');
    renderList('delete-folders-list', config.DeleteFolders || [], 'folder');
    
    setupChangeListeners();
}

function setupChangeListeners() {
    const inputs = document.querySelectorAll('input[type="text"], input[type="number"], input[type="checkbox"]');
    inputs.forEach(input => {
        input.addEventListener('input', updateSaveButton);
        input.addEventListener('change', updateSaveButton);
    });
}

function renderList(containerId, items, type) {
    const container = document.getElementById(containerId);
    container.innerHTML = '';
    
    const listItems = document.createElement('div');
    listItems.className = 'list-items';
    
    items.forEach((item, index) => {
        const itemEl = document.createElement('div');
        itemEl.className = 'list-item';
        itemEl.innerHTML = `
            <input type="text" value="${escapeHtml(item)}" data-index="${index}">
            <button class="list-item-remove" data-index="${index}">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                    <line x1="18" y1="6" x2="6" y2="18"/>
                    <line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
            </button>
        `;
        listItems.appendChild(itemEl);
    });
    
    container.appendChild(listItems);
    
    const addBtn = document.createElement('button');
    addBtn.className = 'list-add';
    addBtn.innerHTML = `
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        添加${type === 'password' ? '密码' : type === 'file' ? '文件' : '文件夹'}
    `;
    container.appendChild(addBtn);
    
    listItems.querySelectorAll('.list-item-remove').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const index = parseInt(e.currentTarget.dataset.index);
            items.splice(index, 1);
            renderList(containerId, items, type);
            updateSaveButton();
        });
    });
    
    listItems.querySelectorAll('input').forEach(input => {
        input.addEventListener('input', updateSaveButton);
        input.addEventListener('change', (e) => {
            const index = parseInt(e.target.dataset.index);
            items[index] = e.target.value;
            updateSaveButton();
        });
    });
    
    addBtn.addEventListener('click', () => {
        items.push('');
        renderList(containerId, items, type);
        const inputs = listItems.querySelectorAll('input');
        if (inputs.length > 0) {
            inputs[inputs.length - 1].focus();
        }
        updateSaveButton();
    });
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function collectForm() {
    return {
        SevenZipPath: document.getElementById('seven-zip-path').value,
        AutoExit: document.getElementById('auto-exit').checked,
        ExtractNestedFolders: document.getElementById('extract-nested').checked,
        DebugMode: document.getElementById('debug-mode').checked,
        DeleteEmptyFolders: document.getElementById('delete-empty').checked,
        FlattenWrapperFolder: document.getElementById('flatten-wrapper').checked,
        DeleteSourceAfterExtract: document.getElementById('delete-source').checked,
        OpenFolderAfterExtract: document.getElementById('open-folder').checked,
        NestedArchiveDepth: parseInt(document.getElementById('nested-depth').value) || 0,
        CreateFolderThreshold: parseInt(document.getElementById('folder-threshold').value) || 1,
        Passwords: collectListItems('passwords-list'),
        DeleteFiles: collectListItems('delete-files-list'),
        DeleteFolders: collectListItems('delete-folders-list'),
    };
}

function collectListItems(containerId) {
    const container = document.getElementById(containerId);
    const inputs = container.querySelectorAll('.list-item input');
    return Array.from(inputs).map(input => input.value).filter(v => v.trim() !== '');
}

function showToast(message, success = true) {
    const toast = document.getElementById('toast');
    const toastMessage = document.getElementById('toast-message');
    
    toastMessage.textContent = message;
    toast.className = 'toast show' + (success ? ' success' : '');
    
    setTimeout(() => {
        toast.className = 'toast';
    }, 3000);
}

document.getElementById('btn-save').addEventListener('click', async () => {
    try {
        const settings = collectForm();
        await invoke('save_config', { settings });
        originalConfig = settings;
        document.getElementById('btn-save').disabled = true;
        showToast('保存成功');
    } catch (e) {
        console.error('保存失败:', e);
        showToast('保存失败: ' + e, false);
    }
});

document.getElementById('btn-reset').addEventListener('click', () => {
    showModal();
});

function showModal() {
    const overlay = document.getElementById('modal-overlay');
    overlay.classList.add('show');
}

function hideModal() {
    const overlay = document.getElementById('modal-overlay');
    overlay.classList.remove('show');
}

function resetConfig() {
    config = {
        SevenZipPath: 'C:\\Program Files\\Bandizip\\bz.exe',
        AutoExit: false,
        ExtractNestedFolders: false,
        DebugMode: false,
        DeleteEmptyFolders: false,
        FlattenWrapperFolder: false,
        DeleteSourceAfterExtract: false,
        OpenFolderAfterExtract: false,
        NestedArchiveDepth: 0,
        CreateFolderThreshold: 1,
        Passwords: ['1234', 'www', '1111'],
        DeleteFiles: ['说明.txt', '更多资源.url'],
        DeleteFolders: ['说明'],
    };
    populateForm(config);
    document.getElementById('btn-save').disabled = false;
    showToast('已重置为默认设置');
}

document.getElementById('modal-cancel').addEventListener('click', hideModal);

document.getElementById('modal-confirm').addEventListener('click', () => {
    hideModal();
    resetConfig();
});

document.getElementById('modal-overlay').addEventListener('click', (e) => {
    if (e.target.id === 'modal-overlay') {
        hideModal();
    }
});

document.getElementById('btn-browse').addEventListener('click', async () => {
    if (!openDialog) {
        showError('文件对话框不可用');
        return;
    }
    try {
        const selected = await openDialog({
            multiple: false,
            filters: [{
                name: 'Executable',
                extensions: ['exe']
            }]
        });
        if (selected) {
            document.getElementById('seven-zip-path').value = selected;
            validatePath(selected);
        }
    } catch (e) {
        console.error('选择文件失败:', e);
    }
});

document.getElementById('seven-zip-path').addEventListener('blur', () => {
    const path = document.getElementById('seven-zip-path').value;
    if (path) {
        validatePath(path);
    }
});

async function validatePath(path) {
    const browseBtn = document.getElementById('btn-browse');
    const originalText = browseBtn.textContent;
    browseBtn.textContent = '验证中...';
    browseBtn.disabled = true;

    try {
        const result = await invoke('validate_bandizip_path', { path });
        showValidationModal(result.valid, result.message);
    } catch (e) {
        showValidationModal(false, '验证失败: ' + e);
    } finally {
        browseBtn.textContent = originalText;
        browseBtn.disabled = false;
    }
}

function showValidationModal(success, message) {
    const existingModal = document.getElementById('validation-modal-overlay');
    if (existingModal) {
        existingModal.remove();
    }

    const iconSvg = success
        ? `<svg class="validation-icon success" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22,4 12,14.01 9,11.01"/></svg>`
        : `<svg class="validation-icon error" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>`;

    const overlay = document.createElement('div');
    overlay.id = 'validation-modal-overlay';
    overlay.className = 'modal-overlay';
    overlay.innerHTML = `
        <div class="modal validation-modal">
            <div class="validation-icon-wrapper">
                ${iconSvg}
            </div>
            <div class="modal-header">
                <h3>${success ? '验证成功' : '验证失败'}</h3>
            </div>
            <div class="modal-body">
                <p>${message}</p>
            </div>
            <div class="modal-footer">
                <button class="btn btn-primary" id="validation-modal-confirm">确定</button>
            </div>
        </div>
    `;

    document.body.appendChild(overlay);

    requestAnimationFrame(() => {
        overlay.classList.add('show');
    });

    document.getElementById('validation-modal-confirm').addEventListener('click', () => {
        overlay.classList.remove('show');
        setTimeout(() => {
            overlay.remove();
        }, 200);
    });

    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) {
            overlay.classList.remove('show');
            setTimeout(() => {
                overlay.remove();
            }, 200);
        }
    });
}

document.getElementById('btn-theme').addEventListener('click', toggleTheme);

document.getElementById('btn-check-update').addEventListener('click', checkForUpdates);

document.getElementById('context-menu-toggle').addEventListener('change', onContextMenuToggle);

async function checkForUpdates() {
    const btn = document.getElementById('btn-check-update');
    const originalContent = btn.innerHTML;
    
    btn.innerHTML = `
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
            <path d="M23 4v6h-6"/>
            <path d="M1 20v-6h6"/>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
        检查中...
    `;
    btn.disabled = true;
    
    try {
        const result = await invoke('check_for_updates');
        
        if (result.error) {
            showUpdateErrorModal(result.error);
        } else if (result.has_update) {
            showUpdateModal(result.current_version, result.latest_version, result.download_url);
        } else {
            showToast(`当前版本 v${result.current_version} 已是最新`, true);
        }
    } catch (e) {
        showUpdateErrorModal(e.toString());
    } finally {
        btn.innerHTML = originalContent;
        btn.disabled = false;
    }
}

function showUpdateModal(currentVersion, latestVersion, downloadUrl) {
    const existingModal = document.getElementById('update-modal-overlay');
    if (existingModal) {
        existingModal.remove();
    }

    const iconSvg = `<svg class="validation-icon success" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22,4 12,14.01 9,11.01"/></svg>`;

    const overlay = document.createElement('div');
    overlay.id = 'update-modal-overlay';
    overlay.className = 'modal-overlay';
    overlay.innerHTML = `
        <div class="modal validation-modal">
            <div class="validation-icon-wrapper">
                ${iconSvg}
            </div>
            <div class="modal-header">
                <h3>发现新版本</h3>
            </div>
            <div class="modal-body">
                <p>当前版本: v${currentVersion}</p>
                <p>最新版本: v${latestVersion}</p>
            </div>
            <div class="modal-footer">
                <button class="btn btn-ghost" id="update-modal-cancel">稍后再说</button>
                <button class="btn btn-primary" id="update-modal-confirm">前往下载</button>
            </div>
        </div>
    `;
    
    document.body.appendChild(overlay);
    
    requestAnimationFrame(() => {
        overlay.classList.add('show');
    });
    
    document.getElementById('update-modal-cancel').addEventListener('click', () => {
        overlay.classList.remove('show');
        setTimeout(() => overlay.remove(), 200);
    });
    
    document.getElementById('update-modal-confirm').addEventListener('click', async () => {
        try {
            await invoke('open_url', { url: downloadUrl });
        } catch (e) {
            window.open(downloadUrl, '_blank');
        }
        overlay.classList.remove('show');
        setTimeout(() => overlay.remove(), 200);
    });
    
    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) {
            overlay.classList.remove('show');
            setTimeout(() => overlay.remove(), 200);
        }
    });
}

function showUpdateErrorModal(errorMessage) {
    const GITHUB_URL = 'https://github.com/wuhenge/SmartUnZip';
    const existingModal = document.getElementById('update-error-modal-overlay');
    if (existingModal) {
        existingModal.remove();
    }

    const iconSvg = `<svg class="validation-icon error" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>`;

    const overlay = document.createElement('div');
    overlay.id = 'update-error-modal-overlay';
    overlay.className = 'modal-overlay';
    overlay.innerHTML = `
        <div class="modal validation-modal">
            <div class="validation-icon-wrapper">
                ${iconSvg}
            </div>
            <div class="modal-header">
                <h3>检查更新失败</h3>
            </div>
            <div class="modal-body">
                <p>无法连接到更新服务器</p>
                <p style="margin-top: 8px; color: var(--text-secondary); font-size: 13px;">${errorMessage}</p>
            </div>
            <div class="modal-footer">
                <button class="btn btn-ghost" id="update-error-modal-close">关闭</button>
                <button class="btn btn-primary" id="update-error-modal-open">打开开源地址</button>
            </div>
        </div>
    `;
    
    document.body.appendChild(overlay);
    
    requestAnimationFrame(() => {
        overlay.classList.add('show');
    });
    
    document.getElementById('update-error-modal-close').addEventListener('click', () => {
        overlay.classList.remove('show');
        setTimeout(() => overlay.remove(), 200);
    });
    
    document.getElementById('update-error-modal-open').addEventListener('click', async () => {
        try {
            await invoke('open_url', { url: GITHUB_URL });
        } catch (e) {
            window.open(GITHUB_URL, '_blank');
        }
        overlay.classList.remove('show');
        setTimeout(() => overlay.remove(), 200);
    });
    
    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) {
            overlay.classList.remove('show');
            setTimeout(() => overlay.remove(), 200);
        }
    });
}

initTheme();
init();
