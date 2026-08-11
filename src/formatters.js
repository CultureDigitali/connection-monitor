export function formatRate(mbps) {
    const rate = Math.max(0, Number(mbps) || 0);
    if (rate >= 1000) return { value: (rate / 1000).toFixed(2), unit: 'Gbps' };
    if (rate >= 1) return { value: rate >= 100 ? rate.toFixed(0) : rate.toFixed(1), unit: 'Mbps' };
    return { value: (rate * 1000).toFixed(0), unit: 'Kbps' };
}

export function formatData(megabytes) {
    const total = Math.max(0, Number(megabytes) || 0);
    if (total >= 1_000_000) return `${(total / 1_000_000).toFixed(2)} TB`;
    if (total >= 1_000) return `${(total / 1_000).toFixed(2)} GB`;
    return `${total.toFixed(0)} MB`;
}
