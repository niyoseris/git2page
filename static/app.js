// ─── DOM Elements ───────────────────────────────────────────────────────────

const form = document.getElementById('analyze-form');
const formSection = document.getElementById('form-section');
const loadingSection = document.getElementById('loading-section');
const errorSection = document.getElementById('error-section');
const resultSection = document.getElementById('result-section');
const submitBtn = document.getElementById('submit-btn');
const btnText = document.getElementById('btn-text');
const loadingStatus = document.getElementById('loading-status');
const errorMessage = document.getElementById('error-message');

let lastResultData = null;

// ─── Load Config from .env ──────────────────────────────────────────────────

(async function loadConfig() {
    try {
        const resp = await fetch('/config');
        const cfg = await resp.json();
        if (cfg.api_url) document.getElementById('api_url').value = cfg.api_url;
        if (cfg.model) document.getElementById('model_name').value = cfg.model;
        if (cfg.has_github_token) {
            document.getElementById('github_token').placeholder = '✓ .env dosyasından yüklendi (üzerine yazabilirsiniz)';
        }
        if (cfg.has_api_key) {
            document.getElementById('api_key').placeholder = '✓ .env dosyasından yüklendi (üzerine yazabilirsiniz)';
        }
    } catch (_) { /* .env not configured, ignore */ }
})();

// ─── State Management ───────────────────────────────────────────────────────

function showSection(section) {
    [formSection, loadingSection, errorSection, resultSection].forEach(s => {
        s.classList.add('hidden');
    });
    section.classList.remove('hidden');
}

function resetToForm() {
    showSection(formSection);
    submitBtn.disabled = false;
    btnText.textContent = 'Analiz Et';
}

function showError(msg) {
    errorMessage.textContent = msg;
    showSection(errorSection);
}

function updateLoadingStatus(text) {
    loadingStatus.textContent = text;
}

// ─── Form Handler ───────────────────────────────────────────────────────────

form.addEventListener('submit', async (e) => {
    e.preventDefault();

    const githubUsername = document.getElementById('github_username').value.trim();
    const githubToken = document.getElementById('github_token').value.trim();
    const apiUrl = document.getElementById('api_url').value.trim();
    const apiKey = document.getElementById('api_key').value.trim();
    const modelName = document.getElementById('model_name').value.trim();
    const language = document.getElementById('language').value;

    if (!githubUsername) {
        showError('Lütfen GitHub kullanıcı adını girin.');
        return;
    }

    // Show loading
    showSection(loadingSection);
    updateLoadingStatus('GitHub verileri çekiliyor...');

    try {
        // Simulate status updates
        const statusTimer = setTimeout(() => {
            updateLoadingStatus('Repolar analiz ediliyor, kaynak kodlar okunuyor...');
        }, 3000);

        const statusTimer2 = setTimeout(() => {
            updateLoadingStatus('AI detaylı analiz yapıyor, bu biraz sürebilir...');
        }, 10000);

        const statusTimer3 = setTimeout(() => {
            updateLoadingStatus('Kod analizi devam ediyor... Büyük projeler daha uzun sürer.');
        }, 30000);

        const statusTimer4 = setTimeout(() => {
            updateLoadingStatus('Neredeyse bitti... AI tüm projeleri inceliyor.');
        }, 60000);

        const response = await fetch('/analyze', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                github_username: githubUsername,
                github_token: githubToken,
                api_url: apiUrl,
                api_key: apiKey,
                model_name: modelName,
                language: language,
            }),
        });

        clearTimeout(statusTimer);
        clearTimeout(statusTimer2);
        clearTimeout(statusTimer3);
        clearTimeout(statusTimer4);

        const data = await response.json();

        if (!response.ok) {
            throw new Error(data.error || 'Bilinmeyen bir hata oluştu.');
        }

        renderResult(data);
    } catch (err) {
        showError(err.message || 'Sunucuya bağlanılamadı.');
    }
});

// ─── Render Functions ───────────────────────────────────────────────────────

function renderResult(data) {
    lastResultData = data;

    // Avatar
    const avatar = document.getElementById('avatar');
    avatar.src = data.avatar_url;
    avatar.alt = data.username;

    // Hero
    document.getElementById('hero-title').textContent = data.hero_title;
    document.getElementById('hero-bio').textContent = data.bio;

    // Profile link
    const profileLink = document.getElementById('profile-link');
    profileLink.href = data.profile_url;

    // Projects
    const grid = document.getElementById('projects-grid');
    grid.innerHTML = '';

    data.projects.forEach((project, index) => {
        const card = createProjectCard(project, index);
        grid.appendChild(card);
    });

    showSection(resultSection);
}

function createProjectCard(project, index) {
    const card = document.createElement('div');
    const delay = Math.min(index * 0.08, 0.8);
    card.className = `bg-white/5 border border-white/10 rounded-2xl p-6 hover:border-brand-500/30 card-glow transition-all duration-300 fade-in-up`;
    card.style.animationDelay = `${delay}s`;
    card.style.opacity = '0';

    // Stars & forks badges
    const statsHTML = `
        <div class="flex items-center gap-3 text-xs text-gray-500">
            ${project.stars > 0 ? `
                <span class="flex items-center gap-1">
                    <svg class="w-3.5 h-3.5 text-yellow-500" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                    </svg>
                    ${project.stars}
                </span>` : ''}
            ${project.forks > 0 ? `
                <span class="flex items-center gap-1">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"/>
                    </svg>
                    ${project.forks}
                </span>` : ''}
            ${project.language ? `
                <span class="flex items-center gap-1">
                    <span class="w-2 h-2 rounded-full ${getLanguageColor(project.language)}"></span>
                    ${project.language}
                </span>` : ''}
        </div>
    `;

    // Tech stack badges
    const techBadges = project.tech_stack.map(tech =>
        `<span class="px-2.5 py-1 bg-brand-500/15 text-brand-300 text-xs font-medium rounded-lg">${escapeHtml(tech)}</span>`
    ).join('');

    // Use cases list
    const useCasesHTML = (project.use_cases && project.use_cases.length > 0) ? `
        <div class="mt-3 mb-3">
            <p class="text-xs font-semibold text-brand-300 uppercase tracking-wider mb-2">Kullanım Senaryoları</p>
            <ul class="space-y-1">
                ${project.use_cases.map(uc => `
                    <li class="flex items-start gap-2 text-xs text-gray-400">
                        <span class="text-brand-400 mt-0.5 flex-shrink-0">▸</span>
                        <span>${escapeHtml(uc)}</span>
                    </li>
                `).join('')}
            </ul>
        </div>
    ` : '';

    card.innerHTML = `
        <div class="flex items-start justify-between mb-3">
            <h3 class="text-lg font-bold text-white truncate">${escapeHtml(project.name)}</h3>
            <a href="${escapeHtml(project.html_url)}" target="_blank" class="text-gray-500 hover:text-brand-400 transition-colors flex-shrink-0 ml-2">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
                </svg>
            </a>
        </div>
        <p class="text-brand-200 text-sm font-medium mb-2">${escapeHtml(project.problem_solved)}</p>
        <p class="text-gray-400 text-sm leading-relaxed mb-3">${escapeHtml(project.detailed_description || '')}</p>
        ${useCasesHTML}
        <div class="flex flex-wrap gap-2 mb-3">${techBadges}</div>
        ${statsHTML}
    `;

    return card;
}

// ─── Utility Functions ──────────────────────────────────────────────────────

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// ─── Export Functions ───────────────────────────────────────────────────────

function downloadFile(filename, content, mimeType) {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

function exportAsJSON() {
    if (!lastResultData) return;
    const json = JSON.stringify(lastResultData, null, 2);
    downloadFile(`${lastResultData.username}-git2page.json`, json, 'application/json');
}

function exportAsCSV() {
    if (!lastResultData) return;
    const headers = ['Name', 'Language', 'Stars', 'Forks', 'Problem Solved', 'Detailed Description', 'Use Cases', 'Tech Stack', 'URL'];
    const rows = lastResultData.projects.map(p => [
        p.name,
        p.language || '',
        p.stars,
        p.forks,
        `"${(p.problem_solved || '').replace(/"/g, '""')}"`,
        `"${(p.detailed_description || '').replace(/"/g, '""')}"`,
        `"${(p.use_cases || []).join('; ').replace(/"/g, '""')}"`,
        `"${(p.tech_stack || []).join(', ').replace(/"/g, '""')}"`,
        p.html_url
    ]);
    const csv = [headers.join(','), ...rows.map(r => r.join(','))].join('\n');
    downloadFile(`${lastResultData.username}-git2page.csv`, csv, 'text/csv');
}

function exportAsMarkdown() {
    if (!lastResultData) return;
    const d = lastResultData;
    let md = `# ${d.hero_title}\n\n`;
    md += `![Avatar](${d.avatar_url})\n\n`;
    md += `${d.bio}\n\n`;
    md += `[GitHub Profile](${d.profile_url})\n\n`;
    md += `---\n\n## Projects\n\n`;
    d.projects.forEach(p => {
        md += `### ${p.name}\n\n`;
        if (p.problem_solved) md += `**Problem:** ${p.problem_solved}\n\n`;
        if (p.detailed_description) md += `${p.detailed_description}\n\n`;
        if (p.use_cases && p.use_cases.length > 0) {
            md += `**Use Cases:**\n`;
            p.use_cases.forEach(uc => { md += `- ${uc}\n`; });
            md += `\n`;
        }
        if (p.tech_stack && p.tech_stack.length > 0) {
            md += `**Tech:** ${p.tech_stack.join(', ')}\n\n`;
        }
        md += `⭐ ${p.stars} | 🍴 ${p.forks} | ${p.language || 'N/A'} | [Repo](${p.html_url})\n\n---\n\n`;
    });
    downloadFile(`${d.username}-git2page.md`, md, 'text/markdown');
}

function exportAsHTML() {
    if (!lastResultData) return;
    const d = lastResultData;
    const projectCards = d.projects.map(p => {
        const useCases = (p.use_cases && p.use_cases.length > 0)
            ? `<div style="margin-top:12px"><strong>Use Cases:</strong><ul>${p.use_cases.map(uc => `<li>${escapeHtml(uc)}</li>`).join('')}</ul></div>`
            : '';
        const techBadges = (p.tech_stack || []).map(t =>
            `<span style="display:inline-block;background:#6366f120;color:#818cf8;padding:2px 10px;border-radius:8px;font-size:12px;margin:2px">${escapeHtml(t)}</span>`
        ).join('');
        return `
        <div style="background:#1e1e2e;border:1px solid #333;border-radius:16px;padding:24px;margin-bottom:16px">
            <div style="display:flex;justify-content:space-between;align-items:center">
                <h3 style="color:#fff;margin:0;font-size:18px">${escapeHtml(p.name)}</h3>
                <a href="${escapeHtml(p.html_url)}" target="_blank" style="color:#818cf8;font-size:13px">View →</a>
            </div>
            <p style="color:#a78bfa;font-size:14px;margin-top:8px;font-weight:500">${escapeHtml(p.problem_solved || '')}</p>
            <p style="color:#9ca3af;font-size:14px;line-height:1.6">${escapeHtml(p.detailed_description || '')}</p>
            ${useCases}
            <div style="margin-top:12px">${techBadges}</div>
            <div style="margin-top:12px;font-size:12px;color:#6b7280">
                ⭐ ${p.stars} &nbsp; 🍴 ${p.forks} &nbsp; ${p.language || ''}
            </div>
        </div>`;
    }).join('');

    const html = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>${escapeHtml(d.username)} - Git2Page</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f0f1a; color: #e5e7eb; padding: 40px 20px; }
        .container { max-width: 800px; margin: 0 auto; }
        a { color: #818cf8; text-decoration: none; }
        ul { padding-left: 20px; }
        li { color: #9ca3af; font-size: 13px; margin: 4px 0; }
    </style>
</head>
<body>
    <div class="container">
        <div style="text-align:center;padding:40px 0;border-bottom:1px solid #222">
            <img src="${d.avatar_url}" alt="avatar" style="width:96px;height:96px;border-radius:50%;border:3px solid #6366f150;margin-bottom:20px"/>
            <h1 style="font-size:36px;background:linear-gradient(to right,#fff,#818cf8);-webkit-background-clip:text;-webkit-text-fill-color:transparent">${escapeHtml(d.hero_title)}</h1>
            <p style="color:#9ca3af;font-size:16px;max-width:600px;margin:16px auto;line-height:1.6">${escapeHtml(d.bio)}</p>
            <a href="${d.profile_url}" target="_blank" style="display:inline-block;margin-top:12px;padding:8px 20px;background:#ffffff15;border-radius:12px;font-size:14px">GitHub Profile</a>
        </div>
        <div style="padding:32px 0">
            <h2 style="font-size:24px;margin-bottom:24px;color:#fff">Projects</h2>
            ${projectCards}
        </div>
        <div style="text-align:center;padding:20px 0;border-top:1px solid #222;color:#6b7280;font-size:13px">
            Generated by Git2Page
        </div>
    </div>
</body>
</html>`;

    downloadFile(`${d.username}-git2page.html`, html, 'text/html');
}

function getLanguageColor(language) {
    const colors = {
        'JavaScript': 'bg-yellow-400',
        'TypeScript': 'bg-blue-400',
        'Python': 'bg-green-400',
        'Rust': 'bg-orange-500',
        'Go': 'bg-cyan-400',
        'Java': 'bg-red-400',
        'C++': 'bg-pink-400',
        'C': 'bg-gray-400',
        'C#': 'bg-purple-400',
        'Ruby': 'bg-red-500',
        'PHP': 'bg-indigo-400',
        'Swift': 'bg-orange-400',
        'Kotlin': 'bg-purple-500',
        'Dart': 'bg-blue-500',
        'Shell': 'bg-green-500',
        'HTML': 'bg-orange-600',
        'CSS': 'bg-blue-600',
        'Vue': 'bg-emerald-400',
        'Svelte': 'bg-red-400',
    };
    return colors[language] || 'bg-gray-400';
}
