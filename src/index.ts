import { CSVNode } from './base/CSVNode';
import { CanvasManager } from './base/CanvasManager';
import { ToolbarManager } from './base/ToolbarManager';
import './styles/canvas.css';
import './styles/toolbar.css';
import './styles/markdown.css';
import { parseCSV } from './utils/csvParser';

document.addEventListener('DOMContentLoaded', () => {
    const app = document.getElementById('app');
    if (!app) return;
    
    app.innerHTML = '<div class="canvas-container"></div><div id="toolbar-container"></div>';
    
    const canvasManager = new CanvasManager('canvas-container');
    const toolbar = new ToolbarManager('toolbar-container', (type, data) => {
        if (type === 'csv') {
            console.log('Creating CSV node with data:', data);
            const position = canvasManager.getViewportCenter();
            console.log('Node position:', position);
            const csvNode = new CSVNode(position, data.fileName, parseCSV(data.content));
            console.log('Created node:', csvNode);
            canvasManager.addNode(csvNode);
        }
    });
    void toolbar;
});