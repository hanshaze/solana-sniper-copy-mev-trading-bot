// Dashboard JavaScript
class TradingBotDashboard {
    constructor() {
        this.charts = {};
        this.refreshInterval = null;
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.initializeCharts();
        this.startAutoRefresh();
        this.loadInitialData();
    }

    setupEventListeners() {
        // Refresh button
        document.getElementById('refreshBtn').addEventListener('click', () => {
            this.refreshAllData();
        });

        // Emergency close button
        document.getElementById('emergencyCloseBtn').addEventListener('click', () => {
            this.showEmergencyModal();
        });

        // Emergency modal
        document.getElementById('confirmEmergencyClose').addEventListener('click', () => {
            this.emergencyCloseAll();
        });

        document.getElementById('cancelEmergencyClose').addEventListener('click', () => {
            this.hideEmergencyModal();
        });

        // Close modal on outside click
        document.getElementById('emergencyModal').addEventListener('click', (e) => {
            if (e.target.id === 'emergencyModal') {
                this.hideEmergencyModal();
            }
        });
    }

    initializeCharts() {
        // PnL Chart
        const pnlCtx = document.getElementById('pnlChart').getContext('2d');
        this.charts.pnl = new Chart(pnlCtx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'Daily PnL (SOL)',
                    data: [],
                    borderColor: '#10b981',
                    backgroundColor: 'rgba(16, 185, 129, 0.1)',
                    tension: 0.4,
                    fill: true
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        labels: { color: '#ffffff' }
                    }
                },
                scales: {
                    x: {
                        ticks: { color: '#9ca3af' },
                        grid: { color: 'rgba(156, 163, 175, 0.2)' }
                    },
                    y: {
                        ticks: { color: '#9ca3af' },
                        grid: { color: 'rgba(156, 163, 175, 0.2)' }
                    }
                }
            }
        });

        // Positions Chart
        const positionsCtx = document.getElementById('positionsChart').getContext('2d');
        this.charts.positions = new Chart(positionsCtx, {
            type: 'doughnut',
            data: {
                labels: ['Active', 'Closed'],
                datasets: [{
                    data: [0, 0],
                    backgroundColor: ['#3b82f6', '#6b7280'],
                    borderWidth: 0
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        labels: { color: '#ffffff' }
                    }
                }
            }
        });
    }

    async loadInitialData() {
        try {
            await Promise.all([
                this.loadStatus(),
                this.loadRiskMetrics(),
                this.loadPositions(),
                this.loadHistory(),
                this.loadConfiguration()
            ]);
        } catch (error) {
            console.error('Error loading initial data:', error);
            this.showError('Failed to load dashboard data');
        }
    }

    async refreshAllData() {
        const refreshBtn = document.getElementById('refreshBtn');
        refreshBtn.classList.add('animate-spin');
        
        try {
            await this.loadInitialData();
            this.showSuccess('Dashboard refreshed successfully');
        } catch (error) {
            console.error('Error refreshing data:', error);
            this.showError('Failed to refresh dashboard');
        } finally {
            refreshBtn.classList.remove('animate-spin');
        }
    }

    startAutoRefresh() {
        // Refresh data every 30 seconds
        this.refreshInterval = setInterval(() => {
            this.loadStatus();
            this.loadRiskMetrics();
            this.loadPositions();
        }, 30000);
    }

    async loadStatus() {
        try {
            const response = await fetch('/api/status');
            const data = await response.json();
            
            // Update bot status
            const statusElement = document.getElementById('botStatus');
            statusElement.innerHTML = `
                <i class="fas fa-circle mr-2"></i>${data.bot.status}
            `;
            statusElement.className = `font-bold status-${data.bot.status}`;
            
        } catch (error) {
            console.error('Error loading status:', error);
        }
    }

    async loadRiskMetrics() {
        try {
            const response = await fetch('/api/risk');
            const data = await response.json();
            
            // Update quick stats
            document.getElementById('activePositions').textContent = data.positionSummary.activePositions;
            document.getElementById('dailyPnL').textContent = `${data.dailyStats.netPnL.toFixed(4)} SOL`;
            document.getElementById('winRate').textContent = `${data.dailyStats.winRate}%`;
            document.getElementById('riskLevel').textContent = data.riskLevel;
            
            // Update PnL chart
            this.updatePnLChart(data.dailyStats);
            
            // Update positions chart
            this.updatePositionsChart(data.positionSummary);
            
        } catch (error) {
            console.error('Error loading risk metrics:', error);
        }
    }

    async loadPositions() {
        try {
            const response = await fetch('/api/positions');
            const data = await response.json();
            
            this.updatePositionsTable(data.positions);
            
        } catch (error) {
            console.error('Error loading positions:', error);
        }
    }

    async loadHistory() {
        try {
            const response = await fetch('/api/history');
            const data = await response.json();
            
            this.updateHistoryTable(data.history);
            
        } catch (error) {
            console.error('Error loading history:', error);
        }
    }

    async loadConfiguration() {
        try {
            const response = await fetch('/api/config');
            const data = await response.json();
            
            this.updateConfigurationGrid(data);
            
        } catch (error) {
            console.error('Error loading configuration:', error);
        }
    }

    updatePnLChart(dailyStats) {
        const chart = this.charts.pnl;
        
        // Add current data point
        const now = new Date();
        const timeLabel = now.toLocaleTimeString();
        
        chart.data.labels.push(timeLabel);
        chart.data.datasets[0].data.push(dailyStats.netPnL);
        
        // Keep only last 20 data points
        if (chart.data.labels.length > 20) {
            chart.data.labels.shift();
            chart.data.datasets[0].data.shift();
        }
        
        chart.update();
    }

    updatePositionsChart(positionSummary) {
        const chart = this.charts.positions;
        chart.data.datasets[0].data = [
            positionSummary.activePositions,
            positionSummary.totalPositions || 0
        ];
        chart.update();
    }

    updatePositionsTable(positions) {
        const tbody = document.getElementById('positionsTableBody');
        
        if (positions.length === 0) {
            tbody.innerHTML = `
                <tr>
                    <td colspan="6" class="text-center p-6 text-gray-400">No active positions</td>
                </tr>
            `;
            return;
        }
        
        tbody.innerHTML = positions.map(position => `
            <tr class="border-b border-gray-700">
                <td class="p-3">
                    <div class="flex items-center space-x-2">
                        <span class="font-mono text-sm">${position.mint.substring(0, 8)}...</span>
                        <button class="text-blue-400 hover:text-blue-300" onclick="copyToClipboard('${position.mint}')">
                            <i class="fas fa-copy"></i>
                        </button>
                    </div>
                </td>
                <td class="p-3">${position.entryPrice.toFixed(6)}</td>
                <td class="p-3">${(position.currentPrice || position.entryPrice).toFixed(6)}</td>
                <td class="p-3 ${position.pnl >= 0 ? 'text-green-400' : 'text-red-400'}">
                    ${(position.pnl || 0).toFixed(4)} SOL
                </td>
                <td class="p-3">${this.formatDuration(position.holdTime)}</td>
                <td class="p-3">
                    <button class="bg-red-600 hover:bg-red-700 px-3 py-1 rounded text-sm transition-colors" 
                            onclick="closePosition('${position.mint}')">
                        Close
                    </button>
                </td>
            </tr>
        `).join('');
    }

    updateHistoryTable(history) {
        const tbody = document.getElementById('historyTableBody');
        
        if (history.length === 0) {
            tbody.innerHTML = `
                <tr>
                    <td colspan="6" class="text-center p-6 text-gray-400">No trading history</td>
                </tr>
            `;
            return;
        }
        
        tbody.innerHTML = history.slice(-10).reverse().map(trade => `
            <tr class="border-b border-gray-700">
                <td class="p-3">${new Date(trade.timestamp).toLocaleString()}</td>
                <td class="p-3">
                    <span class="px-2 py-1 rounded text-xs ${trade.type === 'buy' ? 'bg-green-600' : 'bg-red-600'}">
                        ${trade.type.toUpperCase()}
                    </span>
                </td>
                <td class="p-3">
                    <span class="font-mono text-sm">${trade.tokenMint.substring(0, 8)}...</span>
                </td>
                <td class="p-3">${trade.amount.toFixed(4)}</td>
                <td class="p-3">${trade.price.toFixed(6)}</td>
                <td class="p-3 ${trade.pnl >= 0 ? 'text-green-400' : 'text-red-400'}">
                    ${trade.pnl ? trade.pnl.toFixed(4) : '-'}
                </td>
            </tr>
        `).join('');
    }

    updateConfigurationGrid(config) {
        const grid = document.getElementById('configGrid');
        
        const configItems = [
            { label: 'Sniper Amount', value: `${config.trading.sniperAmount} SOL` },
            { label: 'Profit Target', value: `${config.trading.profitTarget}x` },
            { label: 'Stop Loss', value: `${config.trading.stopLoss}x` },
            { label: 'Max Hold Time', value: `${config.trading.maxHoldTime / 1000}s` },
            { label: 'Max Positions', value: config.trading.maxPositions },
            { label: 'Max Daily Loss', value: `${config.risk.maxDailyLoss} SOL` },
            { label: 'Swap Method', value: config.swap.method },
            { label: 'Slippage', value: `${config.swap.slippageTolerance}%` },
            { label: 'Priority Fee', value: `${config.swap.priorityFee} lamports` }
        ];
        
        grid.innerHTML = configItems.map(item => `
            <div class="bg-gray-800 p-4 rounded-lg">
                <div class="text-sm text-gray-400">${item.label}</div>
                <div class="text-lg font-semibold">${item.value}</div>
            </div>
        `).join('');
    }

    showEmergencyModal() {
        document.getElementById('emergencyModal').classList.remove('hidden');
        document.getElementById('emergencyModal').classList.add('flex');
    }

    hideEmergencyModal() {
        document.getElementById('emergencyModal').classList.add('hidden');
        document.getElementById('emergencyModal').classList.remove('flex');
    }

    async emergencyCloseAll() {
        try {
            this.hideEmergencyModal();
            
            const response = await fetch('/api/emergency/close-all', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ reason: 'dashboard_emergency' })
            });
            
            const result = await response.json();
            
            if (result.success) {
                this.showSuccess('Emergency closure initiated');
                this.refreshAllData();
            } else {
                this.showError('Failed to initiate emergency closure');
            }
            
        } catch (error) {
            console.error('Error during emergency closure:', error);
            this.showError('Error during emergency closure');
        }
    }

    formatDuration(ms) {
        const seconds = Math.floor(ms / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);
        
        if (hours > 0) {
            return `${hours}h ${minutes % 60}m`;
        } else if (minutes > 0) {
            return `${minutes}m ${seconds % 60}s`;
        } else {
            return `${seconds}s`;
        }
    }

    showSuccess(message) {
        this.showNotification(message, 'success');
    }

    showError(message) {
        this.showNotification(message, 'error');
    }

    showNotification(message, type) {
        const notification = document.createElement('div');
        notification.className = `fixed top-4 right-4 p-4 rounded-lg shadow-lg z-50 ${
            type === 'success' ? 'bg-green-600' : 'bg-red-600'
        }`;
        notification.textContent = message;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            notification.remove();
        }, 3000);
    }
}

// Global functions for table actions
window.copyToClipboard = function(text) {
    navigator.clipboard.writeText(text).then(() => {
        // Could show a toast notification here
    });
};

window.closePosition = function(mint) {
    // Implement individual position closure
    console.log('Closing position for:', mint);
};

// Initialize dashboard when page loads
document.addEventListener('DOMContentLoaded', () => {
    window.dashboard = new TradingBotDashboard();
});
