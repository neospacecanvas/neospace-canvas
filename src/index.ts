import { GridManager } from './base/GridManager';
import './styles/canvas.css';

document.addEventListener('DOMContentLoaded', () => {
  const app = document.getElementById('app');
  if (!app) return;
  new GridManager('canvas-container');
  app.innerHTML = '<div class="canvas-container"></div>';
});