export class HistoryChart {
    constructor(canvas) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
    }

    render(model, mode = 'quality', cursor = null) {
        const parentWidth = this.canvas.parentElement?.getBoundingClientRect().width || 280;
        const width = Math.max(240, parentWidth - 20);
        const height = 150;
        const dpr = window.devicePixelRatio || 1;
        this.canvas.width = width * dpr;
        this.canvas.height = height * dpr;
        this.canvas.style.width = `${width}px`;
        this.canvas.style.height = `${height}px`;
        this.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        this.ctx.clearRect(0, 0, width, height);

        const points = model?.points || [];
        if (!points.length) return;
        const padding = { left: 8, right: 8, top: 12, bottom: 12 };
        const start = points[0].timestamp;
        const end = points.at(-1).timestamp || start + 1;
        const x = (timestamp) => padding.left
            + ((timestamp - start) / Math.max(1, end - start)) * (width - padding.left - padding.right);

        const series = mode === 'bandwidth'
            ? [['download', '#34d399'], ['upload', '#60a5fa']]
            : mode === 'ping'
                ? [['ping', '#fbbf24']]
                : [['quality', '#a78bfa']];
        const values = points.flatMap((point) => series.map(([key]) => point[key])).filter(Number.isFinite);
        const maximum = mode === 'quality' ? 100 : Math.max(1, ...values) * 1.12;
        const y = (value) => height - padding.bottom
            - (value / maximum) * (height - padding.top - padding.bottom);

        this.ctx.strokeStyle = 'rgba(255,255,255,.07)';
        this.ctx.lineWidth = 1;
        for (let row = 0; row <= 3; row += 1) {
            const lineY = padding.top + row * ((height - padding.top - padding.bottom) / 3);
            this.ctx.beginPath();
            this.ctx.moveTo(padding.left, lineY);
            this.ctx.lineTo(width - padding.right, lineY);
            this.ctx.stroke();
        }

        for (const incident of model.incidents || []) {
            const markerX = x(incident.started_at);
            this.ctx.fillStyle = incident.kind === 'offline' ? '#f87171' : '#fb923c';
            this.ctx.fillRect(markerX - 1, padding.top, 2, height - padding.top - padding.bottom);
        }

        for (const [key, color] of series) {
            this.ctx.beginPath();
            let drawing = false;
            for (const point of points) {
                const value = point[key];
                if (!Number.isFinite(value)) {
                    drawing = false;
                    continue;
                }
                if (drawing) this.ctx.lineTo(x(point.timestamp), y(value));
                else this.ctx.moveTo(x(point.timestamp), y(value));
                drawing = true;
            }
            this.ctx.strokeStyle = color;
            this.ctx.lineWidth = 2;
            this.ctx.lineJoin = 'round';
            this.ctx.stroke();
        }

        if (Number.isFinite(cursor)) {
            this.ctx.strokeStyle = 'rgba(255,255,255,.8)';
            this.ctx.setLineDash([3, 3]);
            this.ctx.beginPath();
            this.ctx.moveTo(x(cursor), padding.top);
            this.ctx.lineTo(x(cursor), height - padding.bottom);
            this.ctx.stroke();
            this.ctx.setLineDash([]);
        }
    }
}
