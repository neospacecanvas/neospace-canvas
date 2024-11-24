
import { ToolbarManager } from './base/ToolbarManager';
import './styles/canvas.css';
import './styles/toolbar.css';
import './styles/markdown.css';
import { GridManager } from './base/GridManager';

document.addEventListener('DOMContentLoaded', () => {
    const app = document.getElementById('app');
    if (!app) return;
    
    app.innerHTML = '<div class="canvas-container"></div><div id="toolbar-container"></div>';
    const gridManager = new GridManager('canvas-container');
    void gridManager;
    const toolbar = new ToolbarManager('toolbar-container');
    void toolbar;
});