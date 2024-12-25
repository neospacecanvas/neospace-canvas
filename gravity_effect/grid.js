import React, { useEffect, useRef, useState } from 'react';

const InfiniteGrid = () => {
  const canvasRef = useRef(null);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [mousePos, setMousePos] = useState({ x: 0, y: 0 });
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  
  // Constants for grid configuration
  const DOT_SPACING = 25;
  const DOT_RADIUS = 1.5;
  const GRAVITY_RADIUS = 150;
  const GRAVITY_STRENGTH = 0.8;
  const GRAVITY_FALLOFF = 2;

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    let animationFrameId;

    // Calculate visible grid range
    const getVisibleGridRange = () => {
      const startX = Math.floor(-offset.x / DOT_SPACING) - 2;
      const endX = startX + Math.ceil(canvas.width / DOT_SPACING) + 4;
      const startY = Math.floor(-offset.y / DOT_SPACING) - 2;
      const endY = startY + Math.ceil(canvas.height / DOT_SPACING) + 4;
      
      return { startX, endX, startY, endY };
    };

    // Calculate gravity effect for a point
    const calculateGravityOffset = (baseX, baseY) => {
      const worldX = baseX * DOT_SPACING + offset.x;
      const worldY = baseY * DOT_SPACING + offset.y;
      
      const dx = mousePos.x - worldX;
      const dy = mousePos.y - worldY;
      const distance = Math.sqrt(dx * dx + dy * dy);
      
      if (distance < GRAVITY_RADIUS) {
        const force = Math.pow(1 - distance / GRAVITY_RADIUS, GRAVITY_FALLOFF) * GRAVITY_STRENGTH;
        return {
          x: dx * force,
          y: dy * force
        };
      }
      return { x: 0, y: 0 };
    };

    // Draw frame
    const render = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#333';
      
      const { startX, endX, startY, endY } = getVisibleGridRange();
      
      // Render only the dots that would be visible
      for (let gridY = startY; gridY <= endY; gridY++) {
        for (let gridX = startX; gridX <= endX; gridX++) {
          const baseX = gridX * DOT_SPACING;
          const baseY = gridY * DOT_SPACING;
          
          // Calculate gravity effect
          const gravityOffset = calculateGravityOffset(gridX, gridY);
          
          // Final position with gravity
          const screenX = baseX + offset.x + gravityOffset.x;
          const screenY = baseY + offset.y + gravityOffset.y;
          
          // Draw dot
          ctx.beginPath();
          ctx.arc(screenX, screenY, DOT_RADIUS, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      animationFrameId = requestAnimationFrame(render);
    };

    // Handle window resize
    const handleResize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };

    // Set up initial state
    handleResize();
    window.addEventListener('resize', handleResize);
    render();

    return () => {
      window.removeEventListener('resize', handleResize);
      cancelAnimationFrame(animationFrameId);
    };
  }, [offset, mousePos]);

  const handleMouseDown = (e) => {
    setIsDragging(true);
    setDragStart({ x: e.clientX - offset.x, y: e.clientY - offset.y });
  };

  const handleMouseMove = (e) => {
    setMousePos({ x: e.clientX, y: e.clientY });
    
    if (isDragging) {
      setOffset({
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y
      });
    }
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  return (
    <canvas
      ref={canvasRef}
      className="w-full h-full bg-black cursor-move"
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
    />
  );
};

export default InfiniteGrid;
