import './styles/main.css';

document.addEventListener('DOMContentLoaded', () => {
  const app = document.getElementById('app');
  if (!app) return;
  
  app.innerHTML = '<div class="canvas-container"></div>';
});