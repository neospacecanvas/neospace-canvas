import { CanvasManager } from './base/CanvasManager';
import { ToolbarManager } from './base/ToolbarManager';
import { GridManager } from './base/GridManager';
import './styles/canvas.css';
import './styles/toolbar.css';
import './styles/markdown.css';

document.addEventListener('DOMContentLoaded', () => {
    // Get or create app container
    const app = document.getElementById('app');
    if (!app) return;
    
    // Setup initial HTML structure
    app.innerHTML = `
        <div class="canvas-container">
            <svg id="canvas-edges"></svg>
            <div id="canvas-nodes"></div>
        </div>
        <div id="toolbar-container"></div>
    `;

    try {
        // Initialize managers in correct order
        const canvasManager = new CanvasManager('canvas-container');
        const gridManager = new GridManager('canvas-container');
        const toolbarManager = new ToolbarManager('toolbar-container', canvasManager);

        // Store managers in window for debugging if needed
        if (process.env.NODE_ENV === 'development') {
            (window as any).__managers = {
                canvas: canvasManager,
                grid: gridManager,
                toolbar: toolbarManager
            };
        }

        // Handle page unload
        window.addEventListener('beforeunload', () => {
            canvasManager.destroy();
            toolbarManager.destroy();
        });

    } catch (error) {
        console.error('Error initializing canvas:', error);
        app.innerHTML = `
            <div class="error-message" style="color: red; padding: 20px;">
                Error initializing canvas. Please refresh the page.
            </div>
        `;
    }
});

// Prevent browser's default drag and drop behavior
document.addEventListener('dragover', (e) => e.preventDefault());
document.addEventListener('drop', (e) => e.preventDefault());

// Prevent zooming on mobile devices
document.addEventListener('gesturestart', (e) => e.preventDefault());
document.addEventListener('gesturechange', (e) => e.preventDefault());
document.addEventListener('gestureend', (e) => e.preventDefault());

// Handle visibility change to prevent any state inconsistencies
document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') {
        // Might need to refresh certain states when tab becomes visible again
        const canvasContainer = document.querySelector('.canvas-container');
        if (canvasContainer) {
            canvasContainer.dispatchEvent(new Event('refresh'));
        }
    }
});