import { bindWindowVisibility } from './window-visibility.js';
import { formatData, formatRate } from './formatters.js';
import { i18n } from './i18n.js';
import { buildDiagnosticModel } from './diagnostics.js';
import { applyOfficialLogo } from './branding.js';
import { buildHistoryView, findIncidentAt, selectReplayPoint } from './history-model.js';
import { HistoryChart } from './history-chart.js';
import { bindTooltips } from './tooltips.js';
import { bindGuide, GuideState } from './guide.js';
import officialLogoUrl from '../src-tauri/icons/128x128.png';

function applyTranslations() {
    document.querySelectorAll('[data-i18n]').forEach((el) => {
        const key = el.getAttribute('data-i18n');
        const prefix = el.getAttribute('data-i18n-prefix') || '';
        el.textContent = prefix + i18n.t(key);
    });
    document.documentElement.lang = i18n.getLanguage();
    document.getElementById('lang-current').textContent = i18n.getLanguage().toUpperCase();
    document.querySelectorAll('.lang-option').forEach((opt) => {
        opt.classList.toggle('active', opt.dataset.lang === i18n.getLanguage());
    });
}

i18n.onChange(() => {
    applyTranslations();
});

class ConnectionMonitor {
    constructor() {
        this.chart = new BandwidthChart('bandwidth-chart');
        this.historyChart = new HistoryChart(document.getElementById('history-chart'));
        this.stats = null;
        this.historyResponse = null;
        this.historyModel = null;
        this.historyRange = '24h';
        this.historyMode = 'quality';
        this.historyCursor = null;
        this.speedTestRunning = false;
        this.diagnosticsExpanded = false;
        this.unsubs = [];
        this.tooltipCleanup = bindTooltips(document, (key) => i18n.t(key));
        this.guide = bindGuide(document, new GuideState(localStorage, '0.3'), (key) => i18n.t(key));
        i18n.onChange(() => this.guide.render());

        this.setupTabs();
        this.setupHistoryControls();
        this.setupEventListeners();
        this.initLanguage();
        this.fetchStats();
    }

    setupTabs() {
        const tabs = document.querySelectorAll('.tab');
        tabs.forEach((tab) => {
            tab.addEventListener('click', () => {
                tabs.forEach((t) => t.classList.remove('active'));
                tab.classList.add('active');
                const tabName = tab.dataset.tab;
                document.querySelectorAll('.tab-content').forEach((content) => {
                    content.classList.add('hidden');
                });
                const target = document.getElementById(`tab-${tabName}`);
                if (target) target.classList.remove('hidden');
                if (tabName === 'statistics') this.loadHistory();
            });
        });

        document.querySelectorAll('.external-link').forEach((link) => {
            link.addEventListener('click', (e) => {
                e.preventDefault();
                const url = link.dataset.url;
                if (url && window.__TAURI_INTERNALS__) {
                    window.__TAURI__.opener.openUrl(url).catch(() => {});
                }
            });
        });

        if (window.__TAURI_INTERNALS__) {
            window.__TAURI__.core.invoke('get_app_version').then((v) => {
                const el = document.getElementById('info-version');
                if (el) el.textContent = `v${v}`;
            }).catch(() => {});
        }
    }

    async initLanguage() {
        let lang = 'en';
        if (window.__TAURI_INTERNALS__) {
            try {
                lang = await window.__TAURI__.core.invoke('get_language');
            } catch (e) {
                try {
                    const stored = localStorage.getItem('cm-lang');
                    if (stored) lang = stored;
                } catch (_) {}
            }
        }
        i18n.setLanguage(lang);
        applyTranslations();
        this.guide.render();
        this.guide.open();
    }

    async setupEventListeners() {
        const closeBtn = document.getElementById('close-btn');
        bindWindowVisibility(closeBtn, () => this.invoke('hide_main_window'));

        const floatingBtn = document.getElementById('toggle-floating');
        floatingBtn.addEventListener('click', () => this.invoke('toggle_floating_window'));

        const speedTestBtn = document.getElementById('speed-test-btn');
        speedTestBtn.addEventListener('click', () => this.runSpeedTest());

        const diagnosticsToggle = document.getElementById('diagnostic-toggle');
        diagnosticsToggle.addEventListener('click', () => {
            this.diagnosticsExpanded = !this.diagnosticsExpanded;
            diagnosticsToggle.setAttribute('aria-expanded', String(this.diagnosticsExpanded));
            document.getElementById('diagnostic-toggle-label').textContent = i18n.t(
                this.diagnosticsExpanded ? 'diagnosticCollapse' : 'diagnosticDetails',
            );
            if (this.stats) this.renderDiagnostics(this.stats);
        });

        const langBtn = document.getElementById('lang-btn');
        const langDropdown = document.getElementById('lang-dropdown');
        langBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            langDropdown.classList.toggle('visible');
        });

        document.querySelectorAll('.lang-option').forEach((opt) => {
            opt.addEventListener('click', async (e) => {
                e.stopPropagation();
                const newLang = opt.dataset.lang;
                langDropdown.classList.remove('visible');
                i18n.setLanguage(newLang);
                applyTranslations();
                if (window.__TAURI_INTERNALS__) {
                    try {
                        await window.__TAURI__.core.invoke('change_language', { langCode: newLang });
                    } catch (e) {
                        console.error('Failed to change language:', e);
                    }
                }
                try { localStorage.setItem('cm-lang', newLang); } catch (_) {}
            });
        });

        document.addEventListener('click', (e) => {
            langDropdown.classList.remove('visible');
        });

        if (window.__TAURI_INTERNALS__) {
            try {
                const { listen } = window.__TAURI__.event;
                const unsub1 = await listen('stats-update', (event) => {
                    this.stats = event.payload;
                    this.updateUI(event.payload);
                });
                const unsub2 = await listen('ping-update', () => {});
                const unsub3 = await listen('speed-test-start', () => {
                    document.getElementById('speed-test-panel').classList.add('visible');
                    document.getElementById('speed-test-info').textContent = i18n.t('speedTestTesting');
                    document.getElementById('speed-test-label').textContent = i18n.t('speedTestTesting');
                    document.getElementById('gauge-number').textContent = '...';
                });
                const unsub4 = await listen('speed-test-done', (event) => {
                    this.handleSpeedTestResult(event.payload);
                });
                const unsub5 = await listen('language-changed', (event) => {
                    i18n.setLanguage(event.payload);
                    applyTranslations();
                    if (this.historyResponse) this.renderHistory();
                });
                const unsub6 = await listen('history-updated', () => {
                    if (document.querySelector('[data-tab="statistics"]').classList.contains('active')) {
                        this.loadHistory();
                    }
                });
                this.unsubs.push(unsub1, unsub2, unsub3, unsub4, unsub5, unsub6);
            } catch (e) {
                console.error('Failed to setup event listeners:', e);
            }
        }
    }

    setupHistoryControls() {
        document.querySelectorAll('[data-history-range]').forEach((button) => {
            button.addEventListener('click', () => {
                this.historyRange = button.dataset.historyRange;
                document.querySelectorAll('[data-history-range]').forEach((candidate) => {
                    candidate.classList.toggle('active', candidate === button);
                });
                this.loadHistory();
            });
        });
        document.querySelectorAll('[data-history-mode]').forEach((button) => {
            button.addEventListener('click', () => {
                this.historyMode = button.dataset.historyMode;
                document.querySelectorAll('[data-history-mode]').forEach((candidate) => {
                    candidate.classList.toggle('active', candidate === button);
                });
                if (this.historyModel) {
                    this.historyChart.render(this.historyModel, this.historyMode, this.historyCursor);
                }
            });
        });
        document.getElementById('history-replay').addEventListener('input', (event) => {
            this.renderReplay(Number(event.target.value));
        });
        window.addEventListener('beforeunload', () => {
            for (const unsubscribe of this.unsubs) unsubscribe();
            this.tooltipCleanup();
        });
    }

    async loadHistory() {
        const container = document.querySelector('.history-container');
        container.classList.add('loading');
        try {
            const response = await this.invoke('get_history', { range: this.historyRange });
            if (!response) throw new Error('history unavailable');
            this.historyResponse = response;
            this.renderHistory();
        } catch (error) {
            document.getElementById('history-empty').textContent = i18n.t('historyUnavailable');
            document.getElementById('history-empty').classList.remove('hidden');
        } finally {
            container.classList.remove('loading');
        }
    }

    renderHistory() {
        this.historyModel = buildHistoryView(this.historyResponse, (key) => i18n.t(key));
        const { summary, points, incidents, streak } = this.historyModel;
        document.getElementById('history-quality').textContent = summary.quality;
        document.getElementById('history-availability').textContent = summary.availability;
        document.getElementById('history-incidents').textContent = summary.incidents;
        document.getElementById('history-data').textContent = summary.data;

        const empty = document.getElementById('history-empty');
        empty.textContent = i18n.t('historyEmpty');
        empty.classList.toggle('hidden', !this.historyModel.empty);
        const replay = document.getElementById('history-replay');
        replay.disabled = points.length === 0;
        if (points.length) {
            replay.min = String(points[0].timestamp);
            replay.max = String(points.at(-1).timestamp);
            replay.step = '60';
            replay.value = replay.max;
            this.renderReplay(Number(replay.value));
        } else {
            this.historyCursor = null;
            this.renderReplay(null);
        }
        this.historyChart.render(this.historyModel, this.historyMode, this.historyCursor);

        const list = document.getElementById('guardian-list');
        if (!incidents.length) {
            const message = document.createElement('p');
            message.className = 'history-muted';
            message.textContent = i18n.t('guardianNoIncidents');
            list.replaceChildren(message);
            document.getElementById('guardian-detail').classList.add('hidden');
        } else {
            list.replaceChildren(...incidents.slice().reverse().map((incident) => {
                const button = document.createElement('button');
                button.className = `guardian-event ${incident.kind}`;
                button.type = 'button';
                const time = new Date(incident.started_at * 1000).toLocaleTimeString(i18n.getLanguage(), {
                    hour: '2-digit', minute: '2-digit',
                });
                button.innerHTML = `<span><strong>${incident.title}</strong><small>${time} · ${incident.cause}</small></span><b>›</b>`;
                button.addEventListener('click', () => {
                    document.getElementById('history-replay').value = String(incident.started_at);
                    this.renderReplay(incident.started_at);
                    this.showIncident(incident);
                });
                return button;
            }));
        }

        document.getElementById('streak-current').textContent = String(streak.current);
        document.getElementById('streak-best').textContent = String(streak.best);
        document.getElementById('streak-progress').style.width = `${streak.progress * 100}%`;
    }

    renderReplay(timestamp) {
        const buckets = this.historyResponse?.buckets || [];
        const point = selectReplayPoint(buckets, timestamp);
        this.historyCursor = point?.started_at ?? null;
        const time = point
            ? new Date(point.started_at * 1000).toLocaleTimeString(i18n.getLanguage(), { hour: '2-digit', minute: '2-digit' })
            : '—';
        document.getElementById('replay-time').textContent = time;
        document.getElementById('replay-quality').textContent = Number.isFinite(point?.average_quality)
            ? `${Math.round(point.average_quality)}/100` : '—';
        if (point && Number.isFinite(point.average_download_mbps)) {
            const down = formatRate(point.average_download_mbps);
            const up = formatRate(point.average_upload_mbps);
            document.getElementById('replay-speed').textContent = `↓${down.value} ↑${up.value}`;
        } else {
            document.getElementById('replay-speed').textContent = '—';
        }
        document.getElementById('replay-ping').textContent = Number.isFinite(point?.average_ping_ms)
            ? `${Math.round(point.average_ping_ms)} ms` : '—';
        if (this.historyModel) {
            this.historyChart.render(this.historyModel, this.historyMode, this.historyCursor);
            const incident = findIncidentAt(this.historyModel.incidents, this.historyCursor);
            if (incident) this.showIncident(incident);
        }
    }

    showIncident(incident) {
        const detail = document.getElementById('guardian-detail');
        detail.classList.remove('hidden');
        document.getElementById('guardian-detail-title').textContent = incident.title;
        document.getElementById('guardian-detail-cause').textContent = incident.measurement
            ? `${incident.cause} · ${incident.measurement}` : incident.cause;
        document.getElementById('guardian-detail-action').textContent = incident.recommendation;
    }

    async invoke(command, ...args) {
        if (window.__TAURI_INTERNALS__) {
            try {
                return await window.__TAURI__.core.invoke(command, ...args);
            } catch (e) {
                console.error(`Failed to invoke ${command}:`, e);
            }
        }
    }

    async runSpeedTest() {
        if (this.speedTestRunning) return;
        this.speedTestRunning = true;
        const btn = document.getElementById('speed-test-btn');
        const label = document.getElementById('speed-test-label');
        btn.classList.add('running');
        label.textContent = i18n.t('speedTestTesting');
        const panel = document.getElementById('speed-test-panel');
        panel.classList.add('visible');
        document.getElementById('speed-test-info').textContent = i18n.t('speedTestConnecting');
        document.getElementById('gauge-number').textContent = '0.0';
        document.getElementById('gauge-arc').style.strokeDashoffset = '251';

        const result = await this.invoke('speed_test');
        this.handleSpeedTestResult(result);

        this.speedTestRunning = false;
        btn.classList.remove('running');
        label.textContent = i18n.t('btnSpeedTest');
    }

    handleSpeedTestResult(result) {
        if (!result) return;
        const gaugeNumber = document.getElementById('gauge-number');
        const gaugeArc = document.getElementById('gauge-arc');
        const info = document.getElementById('speed-test-info');

        if (result.success) {
            gaugeNumber.textContent = result.download_mbps.toFixed(1);
            const percent = Math.min(result.download_mbps / 100, 1);
            gaugeArc.style.strokeDashoffset = (251 * (1 - percent)).toString();
            info.textContent = `${i18n.t('speedTestLatency')}: ${result.latency_ms.toFixed(0)} ms`;
        } else {
            gaugeNumber.textContent = '!';
            gaugeArc.style.strokeDashoffset = '251';
            info.textContent = result.error || i18n.t('speedTestError');
        }
    }

    async fetchStats() {
        if (!window.__TAURI_INTERNALS__) return;
        try {
            const stats = await window.__TAURI__.core.invoke('get_bandwidth');
            this.updateUI(stats);
            this.stats = stats;
        } catch (e) {
            console.error('Failed to fetch stats:', e);
        }
    }

    updateUI(stats) {
        const download = formatRate(stats.download_mbps);
        const upload = formatRate(stats.upload_mbps);
        document.getElementById('download-speed').textContent = download.value;
        document.getElementById('download-unit').textContent = download.unit;
        document.getElementById('upload-speed').textContent = upload.value;
        document.getElementById('upload-unit').textContent = upload.unit;

        const pingEl = document.getElementById('ping-value');
        pingEl.textContent = stats.ping_ms > 0 ? `${stats.ping_ms.toFixed(0)} ms` : '-- ms';
        document.getElementById('metric-ping').classList.toggle('warning', stats.ping_ms > 100 && stats.ping_ms <= 200);
        document.getElementById('metric-ping').classList.toggle('danger', stats.ping_ms > 200);

        const jitterEl = document.getElementById('jitter-value');
        jitterEl.textContent = stats.jitter_ms > 0 ? `${stats.jitter_ms.toFixed(1)} ms` : '-- ms';
        document.getElementById('metric-jitter').classList.toggle('warning', stats.jitter_ms > 15 && stats.jitter_ms <= 30);
        document.getElementById('metric-jitter').classList.toggle('danger', stats.jitter_ms > 30);

        const lossEl = document.getElementById('loss-value');
        lossEl.textContent = `${stats.packet_loss.toFixed(1)}%`;
        document.getElementById('metric-loss').classList.toggle('warning', stats.packet_loss > 0 && stats.packet_loss <= 5);
        document.getElementById('metric-loss').classList.toggle('danger', stats.packet_loss > 5);

        const wifiText = stats.wifi_ssid
            ? `${stats.wifi_ssid}`
            : stats.wifi_signal !== null
            ? `${stats.wifi_signal}dBm`
            : stats.is_connected ? i18n.t('wifiEthernet') : '--';
        document.getElementById('wifi-value').textContent = wifiText;
        const wifiMetric = document.getElementById('metric-wifi');
        wifiMetric.classList.toggle('warning', stats.wifi_signal !== null && stats.wifi_signal < -70 && stats.wifi_signal >= -80);
        wifiMetric.classList.toggle('danger', stats.wifi_signal !== null && stats.wifi_signal < -80);

        const dot = document.getElementById('status-dot');
        const qualityText = document.getElementById('quality-text');
        const scoreText = document.getElementById('status-score');

        if (stats.connection_status === 'connecting') {
            dot.className = 'status-dot connecting';
            qualityText.textContent = i18n.t('statusConnecting');
            scoreText.textContent = '…';
        } else if (stats.connection_status === 'offline') {
            dot.className = 'status-dot disconnected';
            qualityText.textContent = i18n.t('statusDisconnected');
            scoreText.textContent = 'OFFLINE';
        } else {
            let labelKey;
            switch (stats.quality_label_key) {
                case 'quality_excellent': labelKey = 'qualityExcellent'; break;
                case 'quality_good': labelKey = 'qualityGood'; break;
                case 'quality_fair': labelKey = 'qualityFair'; break;
                case 'quality_poor': labelKey = 'qualityPoor'; break;
                case 'quality_critical': labelKey = 'qualityCritical'; break;
                default: labelKey = 'qualityGood';
            }
            const qualityClass = stats.quality_label_key.replace('quality_', '');
            dot.className = `status-dot ${qualityClass}`;
            qualityText.textContent = i18n.t(labelKey);
            scoreText.textContent = `${stats.quality_score}/100`;
        }

        this.renderDiagnostics(stats);

        const hours = Math.floor(stats.uptime_seconds / 3600);
        const minutes = Math.floor((stats.uptime_seconds % 3600) / 60);
        const seconds = stats.uptime_seconds % 60;
        const h = i18n.t('timeH'), m = i18n.t('timeM'), s = i18n.t('timeS');
        let uptimeStr;
        if (hours > 0) {
            uptimeStr = `${i18n.t('uptime')}: ${hours}${h} ${minutes}${m}`;
        } else if (minutes > 0) {
            uptimeStr = `${i18n.t('uptime')}: ${minutes}${m} ${seconds}${s}`;
        } else {
            uptimeStr = `${i18n.t('uptime')}: ${seconds}${s}`;
        }
        document.getElementById('uptime').textContent = uptimeStr;
        const transferred = stats.total_download_mb + stats.total_upload_mb;
        document.getElementById('total-data').textContent = `${i18n.t('totalData')}: ${formatData(transferred)}`;

        if (stats.bandwidth_history && stats.bandwidth_history.length > 0) {
            this.chart.update(stats.bandwidth_history);
        }
    }

    renderDiagnostics(stats) {
        const model = buildDiagnosticModel(stats, (key) => i18n.t(key));
        const summary = document.getElementById('diagnostic-summary');
        summary.replaceChildren(...model.summary.map((reason) => {
            const item = document.createElement('span');
            item.textContent = reason;
            return item;
        }));
        document.getElementById('diagnostic-recommendation').textContent = model.recommendation;

        const details = document.getElementById('diagnostic-details');
        details.replaceChildren(...model.rows.map((row) => {
            const item = document.createElement('div');
            item.className = `diagnostic-row ${row.severity}`;
            const metric = document.createElement('span');
            metric.textContent = `${row.label} · ${row.measured}`;
            const penalty = document.createElement('strong');
            penalty.textContent = `${row.penalty} pt`;
            item.append(metric, penalty);
            return item;
        }));
        details.classList.toggle('hidden', !this.diagnosticsExpanded || model.rows.length === 0);
        document.getElementById('diagnostic-toggle').classList.toggle('hidden', model.rows.length === 0);
        document.getElementById('diagnostic-card').dataset.state = model.state;
    }
}

class BandwidthChart {
    constructor(canvasId) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.data = [];
        this.maxPoints = 60;
        this.lastDrawTime = 0;
        this.displayData = [];
        this.targetData = [];

        this.setupCanvas();
        this.animate();

        if (window.ResizeObserver) {
            const ro = new ResizeObserver(() => this.setupCanvas());
            ro.observe(this.canvas.parentElement);
        } else {
            window.addEventListener('resize', () => this.setupCanvas());
        }
    }

    setupCanvas() {
        const rect = this.canvas.parentElement.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        const cssW = Math.max(1, (rect.width - 20));
        const cssH = 85;
        this.canvas.width = cssW * dpr;
        this.canvas.height = cssH * dpr;
        this.canvas.style.width = `${cssW}px`;
        this.canvas.style.height = `${cssH}px`;
        this.ctx.setTransform(1, 0, 0, 1, 0, 0);
        this.ctx.scale(dpr, dpr);
        this.cssWidth = cssW;
        this.cssHeight = cssH;
    }

    update(history) {
        this.targetData = history.map(([, dl, ul]) => ({ dl, ul }));
        if (this.targetData.length > this.maxPoints) {
            this.targetData = this.targetData.slice(-this.maxPoints);
        }
        if (this.displayData.length === 0) {
            this.displayData = this.targetData.map(d => ({ ...d }));
        }
    }

    animate() {
        const now = performance.now();
        if (now - this.lastDrawTime > 16) {
            if (this.displayData.length < this.targetData.length) {
                this.displayData.push({ ...this.targetData[this.displayData.length] });
            } else if (this.displayData.length > this.targetData.length) {
                this.displayData.shift();
            }
            for (let i = 0; i < this.displayData.length; i++) {
                const t = this.targetData[i] || this.displayData[i];
                this.displayData[i].dl += (t.dl - this.displayData[i].dl) * 0.3;
                this.displayData[i].ul += (t.ul - this.displayData[i].ul) * 0.3;
            }
            this.draw();
            this.lastDrawTime = now;
        }
        requestAnimationFrame(() => this.animate());
    }

    draw() {
        const { ctx, cssWidth, cssHeight } = this;
        if (!cssWidth || !cssHeight) return;

        ctx.clearRect(0, 0, cssWidth, cssHeight);

        if (this.displayData.length < 2) return;

        let maxVal = 1;
        for (const d of this.displayData) {
            maxVal = Math.max(maxVal, d.dl, d.ul);
        }
        maxVal = Math.max(maxVal, 1);

        const padding = { top: 6, bottom: 6, left: 4, right: 4 };
        const chartW = cssWidth - padding.left - padding.right;
        const chartH = cssHeight - padding.top - padding.bottom;

        const getX = (i) => padding.left + (i / (this.displayData.length - 1)) * chartW;
        const getY = (val) => padding.top + chartH - (val / maxVal) * chartH;

        this.drawSeries('ul', padding, cssHeight, getX, getY, '#60a5fa', 'rgba(96, 165, 250, 0.35)', 'rgba(96, 165, 250, 0.02)');
        this.drawSeries('dl', padding, cssHeight, getX, getY, '#34d399', 'rgba(52, 211, 153, 0.35)', 'rgba(52, 211, 153, 0.02)');

        const last = this.displayData[this.displayData.length - 1];
        const lastX = getX(this.displayData.length - 1);

        ctx.beginPath();
        ctx.arc(lastX, getY(last.dl), 3.5, 0, Math.PI * 2);
        ctx.fillStyle = '#34d399';
        ctx.shadowColor = 'rgba(52, 211, 153, 0.9)';
        ctx.shadowBlur = 10;
        ctx.fill();

        ctx.beginPath();
        ctx.arc(lastX, getY(last.ul), 3.5, 0, Math.PI * 2);
        ctx.fillStyle = '#60a5fa';
        ctx.shadowColor = 'rgba(96, 165, 250, 0.9)';
        ctx.shadowBlur = 10;
        ctx.fill();

        ctx.shadowBlur = 0;
    }

    drawSeries(key, padding, fullHeight, getX, getY, lineColor, fillTop, fillBottom) {
        const { ctx } = this;

        ctx.beginPath();
        ctx.moveTo(padding.left, fullHeight);
        for (let i = 0; i < this.displayData.length; i++) {
            ctx.lineTo(getX(i), getY(this.displayData[i][key]));
        }
        ctx.lineTo(getX(this.displayData.length - 1), fullHeight);
        ctx.closePath();

        const gradient = ctx.createLinearGradient(0, padding.top, 0, fullHeight);
        gradient.addColorStop(0, fillTop);
        gradient.addColorStop(1, fillBottom);
        ctx.fillStyle = gradient;
        ctx.fill();

        ctx.beginPath();
        for (let i = 0; i < this.displayData.length; i++) {
            const x = getX(i);
            const y = getY(this.displayData[i][key]);
            if (i === 0) {
                ctx.moveTo(x, y);
            } else {
                const prevX = getX(i - 1);
                const prevY = getY(this.displayData[i - 1][key]);
                const cpx = (prevX + x) / 2;
                ctx.bezierCurveTo(cpx, prevY, cpx, y, x, y);
            }
        }
        ctx.strokeStyle = lineColor;
        ctx.lineWidth = 2;
        ctx.shadowColor = lineColor;
        ctx.shadowBlur = 4;
        ctx.stroke();
        ctx.shadowBlur = 0;
    }
}

document.addEventListener('DOMContentLoaded', () => {
    applyOfficialLogo(document.querySelectorAll('[data-app-logo]'), officialLogoUrl, 'Connection Monitor');
    new ConnectionMonitor();
});
