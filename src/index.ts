import { GridManager } from './base/GridManager';
import { ToolbarManager } from './base/ToolbarManager';
import './styles/canvas.css';
import './styles/toolbar.css';

document.addEventListener('DOMContentLoaded', () => {
    const app = document.getElementById('app');
    if (!app) return;
    
    app.innerHTML = '<div class="canvas-container"></div><div id="toolbar-container"></div>';
    
    new GridManager('canvas-container');
    const toolbar = new ToolbarManager('toolbar-container');
    void toolbar;
});