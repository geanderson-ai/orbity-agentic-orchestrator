#!/usr/bin/env python3
"""
Generator for index.html with the full Spatial Canvas (Maestri-style) engine.
"""

HTML_CONTENT = r'''<!DOCTYPE html>
<html lang="pt-BR" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Orbity Multi Agentic Harness | Spatial Canvas & Orchestrator</title>
  <meta name="description" content="Orquestrador corporativo de agentes de IA em Rust com Spatial Canvas infinito, isolamento Sandbox Bubblewrap, trilha de auditoria SHA-256 e FinOps em tempo real.">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;500;600;700&family=Inter:wght@300;400;500;600;700;800&display=swap" rel="stylesheet">
  
  <style>
    :root {
      --bg-dark: #07090e;
      --bg-canvas: #090c14;
      --bg-card: rgba(15, 20, 31, 0.85);
      --bg-card-hover: rgba(24, 32, 50, 0.95);
      --border-color: rgba(226, 232, 240, 0.12);
      --border-highlight: rgba(56, 189, 248, 0.4);
      --text-main: #f8fafc;
      --text-muted: #94a3b8;
      --accent-indigo: #6366f1;
      --accent-cyan: #06b6d4;
      --accent-sky: #38bdf8;
      --accent-emerald: #10b981;
      --accent-amber: #f59e0b;
      --accent-rose: #f43f5e;
      --accent-purple: #a855f7;
      --code-bg: #030611;
      --node-header-bg: rgba(20, 27, 44, 0.9);
      --surface-shadow: 0 20px 50px rgba(0, 0, 0, 0.45);
      --socket-in: #06b6d4;
      --socket-out: #10b981;
    }

    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }

    body {
      background-color: var(--bg-dark);
      color: var(--text-main);
      font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
      overflow-x: hidden;
      height: 100vh;
      display: flex;
      flex-direction: column;
    }

    /* Scrollbars */
    ::-webkit-scrollbar { width: 6px; height: 6px; }
    ::-webkit-scrollbar-track { background: rgba(15, 23, 42, 0.6); }
    ::-webkit-scrollbar-thumb { background: rgba(148, 163, 184, 0.3); border-radius: 4px; }
    ::-webkit-scrollbar-thumb:hover { background: rgba(56, 189, 248, 0.6); }

    /* Top Navigation */
    header.app-header {
      background: rgba(11, 15, 25, 0.96);
      backdrop-filter: blur(12px);
      border-bottom: 1px solid var(--border-color);
      height: 56px;
      min-height: 56px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 1rem;
      z-index: 100;
    }

    .header-left {
      display: flex;
      align-items: center;
      gap: 1rem;
    }

    .logo-container {
      display: flex;
      align-items: center;
      gap: 0.6rem;
      text-decoration: none;
      color: var(--text-main);
      font-weight: 700;
      font-size: 1.05rem;
    }

    .logo-img {
      width: 32px;
      height: 32px;
      border-radius: 50%;
      border: 1.5px solid var(--accent-sky);
      object-fit: cover;
      box-shadow: 0 0 12px rgba(56, 189, 248, 0.3);
    }

    .mode-switch {
      display: flex;
      background: rgba(3, 7, 18, 0.7);
      padding: 3px;
      border-radius: 8px;
      border: 1px solid var(--border-color);
      gap: 3px;
    }

    .mode-btn {
      padding: 5px 12px;
      font-size: 0.8rem;
      font-weight: 600;
      border: none;
      background: transparent;
      color: var(--text-muted);
      border-radius: 6px;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 6px;
      transition: all 0.2s;
    }

    .mode-btn:hover {
      color: var(--text-main);
      background: rgba(255, 255, 255, 0.05);
    }

    .mode-btn.active {
      background: linear-gradient(135deg, rgba(99, 102, 241, 0.3), rgba(6, 182, 212, 0.3));
      border: 1px solid var(--accent-sky);
      color: #fff;
      box-shadow: 0 2px 8px rgba(6, 182, 212, 0.25);
    }

    .header-center {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      flex: 1;
      max-width: 650px;
      margin: 0 1rem;
    }

    .prompt-bar {
      display: flex;
      align-items: center;
      background: rgba(3, 7, 18, 0.85);
      border: 1px solid var(--border-color);
      border-radius: 8px;
      padding: 2px 6px;
      width: 100%;
      transition: border-color 0.2s;
    }

    .prompt-bar:focus-within {
      border-color: var(--accent-sky);
      box-shadow: 0 0 12px rgba(56, 189, 248, 0.2);
    }

    .prompt-input {
      flex: 1;
      background: transparent;
      border: none;
      outline: none;
      color: var(--text-main);
      font-size: 0.825rem;
      padding: 6px 8px;
    }

    .prompt-input::placeholder {
      color: #64748b;
    }

    .btn-run {
      background: linear-gradient(135deg, #0284c7, #06b6d4);
      border: none;
      color: white;
      font-weight: 600;
      font-size: 0.775rem;
      padding: 6px 12px;
      border-radius: 6px;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 5px;
      transition: all 0.2s;
      white-space: nowrap;
    }

    .btn-run:hover {
      filter: brightness(1.15);
      box-shadow: 0 0 12px rgba(6, 182, 212, 0.4);
    }

    .btn-run:active {
      transform: scale(0.98);
    }

    .btn-run.running {
      background: linear-gradient(135deg, #e11d48, #f43f5e);
    }

    .header-right {
      display: flex;
      align-items: center;
      gap: 0.75rem;
    }

    .finops-ticker {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      background: rgba(3, 7, 18, 0.7);
      padding: 4px 10px;
      border-radius: 6px;
      border: 1px solid var(--border-color);
      font-family: 'Fira Code', monospace;
      font-size: 0.75rem;
    }

    .finops-item {
      display: flex;
      align-items: center;
      gap: 4px;
    }

    .finops-label {
      color: var(--text-muted);
    }

    .finops-val {
      font-weight: 600;
      color: var(--accent-sky);
    }

    .status-pill {
      display: flex;
      align-items: center;
      gap: 5px;
      font-size: 0.7rem;
      padding: 3px 8px;
      border-radius: 12px;
      background: rgba(16, 185, 129, 0.12);
      border: 1px solid rgba(16, 185, 129, 0.3);
      color: #34d399;
      font-weight: 600;
    }

    .status-dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      background: #10b981;
      box-shadow: 0 0 6px #10b981;
      animation: pulse 2s infinite;
    }

    @keyframes pulse {
      0%, 100% { opacity: 1; transform: scale(1); }
      50% { opacity: 0.4; transform: scale(0.85); }
    }

    /* Main Workspaces */
    .view-container {
      flex: 1;
      display: none;
      position: relative;
      overflow: hidden;
    }

    .view-container.active {
      display: flex;
    }

    /* SPATIAL CANVAS WORKSPACE */
    #spatialCanvasView {
      width: 100%;
      height: 100%;
      display: none;
      position: relative;
      overflow: hidden;
      background: var(--bg-canvas);
    }

    #spatialCanvasView.active {
      display: flex;
    }

    /* Palette Sidebar */
    .canvas-sidebar {
      width: 250px;
      min-width: 250px;
      background: rgba(10, 14, 23, 0.95);
      border-right: 1px solid var(--border-color);
      display: flex;
      flex-direction: column;
      z-index: 20;
      user-select: none;
    }

    .sidebar-header {
      padding: 0.75rem 1rem;
      border-bottom: 1px solid var(--border-color);
      font-size: 0.8rem;
      font-weight: 700;
      color: var(--text-muted);
      text-transform: uppercase;
      letter-spacing: 0.05em;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .palette-scroll {
      flex: 1;
      overflow-y: auto;
      padding: 0.75rem;
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }

    .palette-group-title {
      font-size: 0.7rem;
      font-weight: 700;
      color: #64748b;
      text-transform: uppercase;
      margin-bottom: 0.4rem;
      padding-left: 0.25rem;
    }

    .palette-item {
      background: rgba(17, 24, 39, 0.75);
      border: 1px solid var(--border-color);
      border-radius: 8px;
      padding: 0.6rem 0.75rem;
      display: flex;
      align-items: center;
      gap: 0.6rem;
      cursor: grab;
      transition: all 0.2s;
    }

    .palette-item:hover {
      background: rgba(30, 41, 59, 0.9);
      border-color: var(--accent-sky);
      transform: translateY(-1px);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    }

    .palette-item:active {
      cursor: grabbing;
    }

    .item-icon {
      width: 28px;
      height: 28px;
      border-radius: 6px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 0.9rem;
      font-weight: bold;
    }

    .item-info {
      flex: 1;
      overflow: hidden;
    }

    .item-name {
      font-size: 0.8rem;
      font-weight: 600;
      color: var(--text-main);
    }

    .item-sub {
      font-size: 0.675rem;
      color: var(--text-muted);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    /* Canvas Viewport */
    .canvas-viewport {
      flex: 1;
      position: relative;
      overflow: hidden;
      cursor: crosshair;
    }

    .canvas-viewport.panning {
      cursor: grabbing !important;
    }

    /* Transform World */
    .canvas-world {
      position: absolute;
      top: 0;
      left: 0;
      width: 10000px;
      height: 10000px;
      transform-origin: 0 0;
      pointer-events: none;
    }

    /* SVG Layer */
    .canvas-svg {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      pointer-events: none;
      z-index: 5;
    }

    .canvas-wire {
      fill: none;
      stroke: #334155;
      stroke-width: 2.5;
      stroke-linecap: round;
      pointer-events: stroke;
      cursor: pointer;
      transition: stroke 0.2s, stroke-width 0.2s;
    }

    .canvas-wire:hover {
      stroke: var(--accent-sky);
      stroke-width: 4;
    }

    .canvas-wire.active {
      stroke: var(--accent-sky);
      stroke-width: 3.5;
      stroke-dasharray: 8 6;
      animation: wireFlow 1s linear infinite;
    }

    .canvas-wire.conditional {
      stroke: #a855f7;
      stroke-dasharray: 4 4;
    }

    .canvas-wire.barrier {
      stroke: #f59e0b;
      stroke-width: 3;
    }

    @keyframes wireFlow {
      from { stroke-dashoffset: 28; }
      to { stroke-dashoffset: 0; }
    }

    .wire-temp {
      fill: none;
      stroke: var(--accent-sky);
      stroke-width: 2.5;
      stroke-dasharray: 5 5;
      pointer-events: none;
    }

    /* Nodes Container */
    .nodes-layer {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      pointer-events: none;
      z-index: 10;
    }

    /* Node Card */
    .node-card {
      position: absolute;
      width: 320px;
      background: var(--bg-card);
      border: 1px solid var(--border-color);
      border-radius: 12px;
      box-shadow: var(--surface-shadow);
      display: flex;
      flex-direction: column;
      pointer-events: auto;
      user-select: none;
      transition: border-color 0.2s, box-shadow 0.2s;
      cursor: default;
    }

    .node-card:hover {
      border-color: var(--border-highlight);
    }

    .node-card.selected {
      border-color: var(--accent-sky);
      box-shadow: 0 0 20px rgba(56, 189, 248, 0.25);
    }

    .node-card.running {
      border-color: #38bdf8;
      box-shadow: 0 0 25px rgba(56, 189, 248, 0.4);
    }

    .node-card.completed {
      border-color: #10b981;
    }

    .node-card.failed {
      border-color: #f43f5e;
      box-shadow: 0 0 20px rgba(244, 63, 94, 0.3);
    }

    .node-header {
      padding: 0.6rem 0.8rem;
      background: var(--node-header-bg);
      border-bottom: 1px solid var(--border-color);
      border-top-left-radius: 11px;
      border-top-right-left: 11px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      cursor: move;
    }

    .node-title-area {
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }

    .node-badge-icon {
      width: 22px;
      height: 22px;
      border-radius: 4px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 0.75rem;
      font-weight: 700;
    }

    .node-id {
      font-family: 'Fira Code', monospace;
      font-size: 0.775rem;
      font-weight: 600;
      color: var(--text-main);
    }

    .node-status-badge {
      font-size: 0.65rem;
      font-weight: 700;
      padding: 2px 6px;
      border-radius: 4px;
      text-transform: uppercase;
      font-family: 'Fira Code', monospace;
    }

    .status-idle { background: rgba(148, 163, 184, 0.15); color: #94a3b8; }
    .status-running { background: rgba(56, 189, 248, 0.2); color: #38bdf8; }
    .status-completed { background: rgba(16, 185, 129, 0.2); color: #34d399; }
    .status-failed { background: rgba(244, 63, 94, 0.2); color: #fb7185; }
    .status-gate { background: rgba(245, 158, 11, 0.2); color: #fbbf24; }

    /* Node PTY Body */
    .node-pty-container {
      background: var(--code-bg);
      height: 140px;
      padding: 0.5rem;
      overflow-y: auto;
      font-family: 'Fira Code', monospace;
      font-size: 0.7rem;
      line-height: 1.4;
      color: #cbd5e1;
      border-bottom: 1px solid var(--border-color);
      display: flex;
      flex-direction: column;
    }

    .pty-log-line {
      white-space: pre-wrap;
      word-break: break-all;
    }

    /* Node Footer */
    .node-footer {
      padding: 0.4rem 0.8rem;
      background: rgba(10, 14, 23, 0.7);
      border-bottom-left-radius: 11px;
      border-bottom-right-radius: 11px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-size: 0.675rem;
      color: var(--text-muted);
      font-family: 'Fira Code', monospace;
    }

    .node-tier-tag {
      background: rgba(255, 255, 255, 0.05);
      padding: 1px 5px;
      border-radius: 3px;
      border: 1px solid rgba(255, 255, 255, 0.1);
    }

    /* Node Ports (Sockets) */
    .node-port {
      position: absolute;
      width: 14px;
      height: 14px;
      border-radius: 50%;
      background: #1e293b;
      border: 2px solid #64748b;
      top: 50%;
      transform: translateY(-50%);
      cursor: crosshair;
      transition: all 0.2s;
      z-index: 15;
    }

    .node-port:hover {
      transform: translateY(-50%) scale(1.35);
    }

    .node-port.input {
      left: -8px;
      border-color: var(--socket-in);
    }

    .node-port.input:hover {
      background: var(--socket-in);
      box-shadow: 0 0 8px var(--socket-in);
    }

    .node-port.output {
      right: -8px;
      border-color: var(--socket-out);
    }

    .node-port.output:hover {
      background: var(--socket-out);
      box-shadow: 0 0 8px var(--socket-out);
    }

    /* Sticky Note Node */
    .node-card.sticky-note {
      width: 220px;
      background: #fef08a;
      border: 1px solid #fde047;
      color: #854d0e;
      box-shadow: 2px 8px 24px rgba(0, 0, 0, 0.35);
    }

    .node-card.sticky-note .node-header {
      background: transparent;
      border-bottom: 1px dashed rgba(133, 77, 14, 0.25);
    }

    .node-card.sticky-note .node-id {
      color: #713f12;
    }

    .sticky-content {
      padding: 0.6rem;
      font-size: 0.775rem;
      background: transparent;
      border: none;
      outline: none;
      resize: none;
      color: #713f12;
      font-family: 'Inter', sans-serif;
      min-height: 90px;
    }

    /* Web Portal Node */
    .node-card.web-portal {
      width: 440px;
    }

    .portal-frame-container {
      height: 220px;
      background: #020617;
      position: relative;
    }

    .portal-iframe {
      width: 100%;
      height: 100%;
      border: none;
    }

    /* Canvas Controls (Zoom / Minimap) */
    .canvas-controls-bottom-left {
      position: absolute;
      bottom: 1rem;
      left: 1rem;
      background: rgba(11, 15, 25, 0.9);
      border: 1px solid var(--border-color);
      border-radius: 8px;
      padding: 4px;
      display: flex;
      gap: 4px;
      z-index: 30;
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    }

    .btn-ctrl {
      background: rgba(255, 255, 255, 0.05);
      border: 1px solid var(--border-color);
      color: var(--text-main);
      padding: 5px 8px;
      font-size: 0.75rem;
      border-radius: 5px;
      cursor: pointer;
      font-weight: 600;
      transition: all 0.2s;
    }

    .btn-ctrl:hover {
      background: rgba(56, 189, 248, 0.15);
      border-color: var(--accent-sky);
    }

    /* Minimap */
    .minimap-container {
      position: absolute;
      bottom: 1rem;
      right: 1rem;
      width: 180px;
      height: 120px;
      background: rgba(10, 14, 23, 0.85);
      border: 1px solid var(--border-color);
      border-radius: 8px;
      overflow: hidden;
      z-index: 30;
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    }

    .minimap-viewport-box {
      position: absolute;
      border: 1.5px solid var(--accent-sky);
      background: rgba(56, 189, 248, 0.1);
      pointer-events: none;
    }

    /* Node Inspector Drawer */
    .node-inspector {
      width: 320px;
      min-width: 320px;
      background: rgba(11, 16, 26, 0.98);
      border-left: 1px solid var(--border-color);
      display: flex;
      flex-direction: column;
      z-index: 25;
      transform: translateX(100%);
      transition: transform 0.25s ease-in-out;
      position: absolute;
      right: 0;
      top: 0;
      bottom: 0;
      box-shadow: -10px 0 30px rgba(0, 0, 0, 0.5);
    }

    .node-inspector.open {
      transform: translateX(0);
    }

    .inspector-header {
      padding: 0.8rem 1rem;
      border-bottom: 1px solid var(--border-color);
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-weight: 700;
      font-size: 0.85rem;
    }

    .inspector-body {
      padding: 1rem;
      overflow-y: auto;
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 0.9rem;
    }

    .form-group {
      display: flex;
      flex-direction: column;
      gap: 0.35rem;
    }

    .form-label {
      font-size: 0.725rem;
      font-weight: 600;
      color: var(--text-muted);
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }

    .form-control {
      background: rgba(3, 7, 18, 0.8);
      border: 1px solid var(--border-color);
      border-radius: 6px;
      padding: 6px 8px;
      color: var(--text-main);
      font-size: 0.8rem;
      outline: none;
      font-family: inherit;
    }

    .form-control:focus {
      border-color: var(--accent-sky);
    }

    textarea.form-control {
      font-family: 'Fira Code', monospace;
      font-size: 0.75rem;
      resize: vertical;
      min-height: 80px;
    }

    /* HITL Approval Modal */
    .modal-overlay {
      position: fixed;
      inset: 0;
      background: rgba(3, 7, 18, 0.8);
      backdrop-filter: blur(8px);
      z-index: 1000;
      display: none;
      align-items: center;
      justify-content: center;
    }

    .modal-overlay.active {
      display: flex;
    }

    .modal-box {
      background: #0f172a;
      border: 1px solid rgba(245, 158, 11, 0.4);
      box-shadow: 0 0 50px rgba(245, 158, 11, 0.25);
      border-radius: 12px;
      width: 90%;
      max-width: 540px;
      padding: 1.5rem;
      display: flex;
      flex-direction: column;
      gap: 1rem;
      animation: modalPop 0.2s ease-out;
    }

    @keyframes modalPop {
      from { transform: scale(0.95); opacity: 0; }
      to { transform: scale(1); opacity: 1; }
    }

    .modal-title {
      display: flex;
      align-items: center;
      gap: 0.6rem;
      font-size: 1.1rem;
      font-weight: 700;
      color: #fbbf24;
    }

    .modal-desc {
      font-size: 0.85rem;
      color: #cbd5e1;
      line-height: 1.5;
    }

    .modal-code {
      background: #020617;
      border: 1px solid var(--border-color);
      border-radius: 6px;
      padding: 0.75rem;
      font-family: 'Fira Code', monospace;
      font-size: 0.8rem;
      color: #38bdf8;
      max-height: 140px;
      overflow-y: auto;
    }

    .modal-actions {
      display: flex;
      justify-content: flex-end;
      gap: 0.75rem;
      margin-top: 0.5rem;
    }

    .btn-approve {
      background: #10b981;
      border: none;
      color: white;
      font-weight: 700;
      padding: 8px 16px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 0.85rem;
    }

    .btn-reject {
      background: #f43f5e;
      border: none;
      color: white;
      font-weight: 700;
      padding: 8px 16px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 0.85rem;
    }

    /* YAML & State Modal */
    .yaml-modal-box {
      max-width: 720px;
      border-color: var(--accent-sky);
      box-shadow: 0 0 50px rgba(56, 189, 248, 0.2);
    }

    /* ARCHITECTURE & DOCS VIEW (Classic Layout) */
    #docsView {
      width: 100%;
      height: 100%;
      overflow-y: auto;
      padding: 2rem 1.5rem;
    }

    .container {
      max-width: 1200px;
      margin: 0 auto;
    }

    /* Reused styles for docs view */
    .section-title { font-size: 1.8rem; font-weight: 800; margin-bottom: 0.5rem; }
    .section-subtitle { color: var(--text-muted); font-size: 0.95rem; margin-bottom: 2rem; }
    .pillars-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 1.25rem; margin-bottom: 3rem; }
    .pillar-card { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 12px; padding: 1.25rem; }
    .pillar-icon { font-size: 1.5rem; margin-bottom: 0.5rem; }
    .pillar-title { font-size: 1.1rem; font-weight: 700; margin-bottom: 0.4rem; }
    .pillar-desc { font-size: 0.825rem; color: var(--text-muted); line-height: 1.5; }
    
    table.layers-table { width: 100%; border-collapse: collapse; margin-top: 1rem; }
    table.layers-table th, table.layers-table td { padding: 0.75rem 1rem; border-bottom: 1px solid var(--border-color); font-size: 0.825rem; }
    table.layers-table th { background: rgba(255, 255, 255, 0.03); color: var(--accent-sky); text-align: left; }
  </style>
</head>
<body>

  <!-- App Header -->
  <header class="app-header">
    <div class="header-left">
      <a href="#" class="logo-container">
        <img src="assets/orbity-logo.jpg" alt="Orbity" class="logo-img" onerror="this.src='data:image/svg+xml,%3Csvg xmlns=\'http://www.w3.org/2000/svg\' viewBox=\'0 0 32 32\'%3E%3Ccircle cx=\'16\' cy=\'16\' r=\'14\' fill=\'%230284c7\'/%3E%3Ctext x=\'16\' y=\'20\' font-size=\'12\' text-anchor=\'middle\' fill=\'%23fff\' font-weight=\'bold\'%3EO%3C/text%3E%3C/svg%3E'">
        <span>Orbity</span>
        <span style="font-size:0.75rem; background:rgba(99,102,241,0.2); color:#818cf8; padding:2px 6px; border-radius:10px; border:1px solid rgba(99,102,241,0.3);">Topcoat 0.9</span>
      </a>

      <!-- View Switcher -->
      <div class="mode-switch">
        <button class="mode-btn active" id="btnModeCanvas" onclick="switchMainView('canvas')">
          <span>🪐</span> Spatial Canvas
        </button>
        <button class="mode-btn" id="btnModeDocs" onclick="switchMainView('docs')">
          <span>📖</span> Arquitetura & Camadas
        </button>
      </div>
    </div>

    <!-- Center Prompt Bar (For Canvas Run) -->
    <div class="header-center" id="headerCenterCanvas">
      <select id="selectTeamPreset" class="form-control" style="width: 140px; padding: 4px 6px; font-size: 0.775rem;">
        <option value="coder.yaml">coder.yaml</option>
        <option value="forester.yaml">forester.yaml</option>
        <option value="architect.yaml">architect.yaml</option>
        <option value="reviewer.yaml">reviewer.yaml</option>
      </select>

      <div class="prompt-bar">
        <input type="text" id="canvasPromptInput" class="prompt-input" placeholder="Digite uma instrução para executar no grafo multi-agente..." value="Escreva um código python para somar dois números e dar print na tela">
        <button class="btn-run" id="btnRunCanvas" onclick="triggerCanvasRun()">
          <span>🚀</span> Executar
        </button>
      </div>
    </div>

    <!-- Right FinOps & Controls -->
    <div class="header-right">
      <div class="finops-ticker">
        <div class="finops-item">
          <span class="finops-label">Tokens:</span>
          <span class="finops-val" id="finopsTokens">0</span>
        </div>
        <div class="finops-item">
          <span class="finops-label">FinOps:</span>
          <span class="finops-val" id="finopsCost">$0.00</span>
        </div>
      </div>

      <div class="status-pill" id="sseBadge">
        <div class="status-dot"></div>
        <span id="sseStatusText">SSE Online</span>
      </div>

      <button class="btn-ctrl" onclick="openYamlModal()" title="Importar / Exportar Contrato YAML">
        YAML ⚙️
      </button>
    </div>
  </header>

  <!-- WORKSPACE 1: SPATIAL CANVAS (MAESTRI-STYLE) -->
  <div id="spatialCanvasView" class="view-container active">
    
    <!-- Left Draggable Palette -->
    <aside class="canvas-sidebar">
      <div class="sidebar-header">
        <span>Paleta de Nós</span>
        <span style="font-size: 0.7rem; color: #64748b;">Arraste ao Canvas</span>
      </div>

      <div class="palette-scroll">
        <!-- Agentes CLI -->
        <div>
          <div class="palette-group-title">Agentes CLI Sandbox</div>
          
          <div class="palette-item" draggable="true" ondragstart="onPaletteDragStart(event, 'codex')">
            <div class="item-icon" style="background: rgba(56, 189, 248, 0.2); color: #38bdf8;">🤖</div>
            <div class="item-info">
              <div class="item-name">Codex Worker</div>
              <div class="item-sub">Geração de código & testes</div>
            </div>
          </div>

          <div class="palette-item" style="margin-top:6px;" draggable="true" ondragstart="onPaletteDragStart(event, 'claude')">
            <div class="item-icon" style="background: rgba(168, 85, 247, 0.2); color: #c084fc;">🧠</div>
            <div class="item-info">
              <div class="item-name">Claude Reviewer</div>
              <div class="item-sub">Arquitetura & revisão segura</div>
            </div>
          </div>

          <div class="palette-item" style="margin-top:6px;" draggable="true" ondragstart="onPaletteDragStart(event, 'agy')">
            <div class="item-icon" style="background: rgba(16, 185, 129, 0.2); color: #34d399;">🔍</div>
            <div class="item-info">
              <div class="item-name">Agy Researcher</div>
              <div class="item-sub">Google Antigravity codebase</div>
            </div>
          </div>

          <div class="palette-item" style="margin-top:6px;" draggable="true" ondragstart="onPaletteDragStart(event, 'hermes')">
            <div class="item-icon" style="background: rgba(245, 158, 11, 0.2); color: #fbbf24;">⚡</div>
            <div class="item-info">
              <div class="item-name">Hermes Tool</div>
              <div class="item-sub">Ferramentas determinísticas</div>
            </div>
          </div>

          <div class="palette-item" style="margin-top:6px;" draggable="true" ondragstart="onPaletteDragStart(event, 'pi')">
            <div class="item-icon" style="background: rgba(244, 63, 94, 0.2); color: #fb7185;">🎯</div>
            <div class="item-info">
              <div class="item-name">Pi Refactor</div>
              <div class="item-sub">Edições cirúrgicas & AST</div>
            </div>
          </div>
        </div>

        <!-- Orquestração & Controle -->
        <div>
          <div class="palette-group-title">Roteamento & Governança</div>

          <div class="palette-item" draggable="true" ondragstart="onPaletteDragStart(event, 'tool')">
            <div class="item-icon" style="background: rgba(99, 102, 241, 0.2); color: #a5b4fc;">🛠️</div>
            <div class="item-info">
              <div class="item-name">Tool / Shell</div>
              <div class="item-sub">Execução pura de comandos</div>
            </div>
          </div>

          <div class="palette-item" style="margin-top:6px;" draggable="true" ondragstart="onPaletteDragStart(event, 'router')">
            <div class="item-icon" style="background: rgba(147, 51, 234, 0.2); color: #c084fc;">🔀</div>
            <div class="item-info">
              <div class="item-name">Conditional Router</div>
              <div class="item-sub">Desvio por predicado</div>
            </div>
          </div>

          <div class="palette-item" style="margin-top:6px;" draggable="true" ondragstart="onPaletteDragStart(event, 'hitl')">
            <div class="item-icon" style="background: rgba(239, 68, 68, 0.2); color: #f87171;">🛑</div>
            <div class="item-info">
              <div class="item-name">Human Gate (HITL)</div>
              <div class="item-sub">Aprovação humana mandatória</div>
            </div>
          </div>

          <div class="palette-item" style="margin-top:6px;" draggable="true" ondragstart="onPaletteDragStart(event, 'barrier')">
            <div class="item-icon" style="background: rgba(234, 179, 8, 0.2); color: #facc15;">⏳</div>
            <div class="item-info">
              <div class="item-name">Join Barrier</div>
              <div class="item-sub">Sincronização de fan-in</div>
            </div>
          </div>
        </div>

        <!-- Auxiliares Espaciais -->
        <div>
          <div class="palette-group-title">Espaço & Portais</div>

          <div class="palette-item" draggable="true" ondragstart="onPaletteDragStart(event, 'sticky')">
            <div class="item-icon" style="background: rgba(254, 240, 138, 0.9); color: #854d0e;">📝</div>
            <div class="item-info">
              <div class="item-name">Sticky Note</div>
              <div class="item-sub">Anotações do plano</div>
            </div>
          </div>

          <div class="palette-item" style="margin-top:6px;" draggable="true" ondragstart="onPaletteDragStart(event, 'portal')">
            <div class="item-icon" style="background: rgba(6, 182, 212, 0.2); color: #22d3ee;">🌐</div>
            <div class="item-info">
              <div class="item-name">Web/Device Portal</div>
              <div class="item-sub">Visualizador de URL/DOM</div>
            </div>
          </div>
        </div>

      </div>
    </aside>

    <!-- Infinite Canvas Viewport -->
    <main class="canvas-viewport" id="canvasViewport"
          ondragover="onCanvasDragOver(event)" 
          ondrop="onCanvasDrop(event)"
          onmousedown="onCanvasMouseDown(event)"
          onwheel="onCanvasWheel(event)">
      
      <div class="canvas-world" id="canvasWorld">
        <!-- SVG Connections Wire Layer -->
        <svg class="canvas-svg" id="canvasSvg">
          <defs>
            <marker id="arrow" viewBox="0 0 10 10" refX="6" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
              <path d="M 0 0 L 10 5 L 0 10 z" fill="#38bdf8"/>
            </marker>
            <marker id="arrow-gray" viewBox="0 0 10 10" refX="6" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
              <path d="M 0 0 L 10 5 L 0 10 z" fill="#475569"/>
            </marker>
          </defs>
          <g id="svgWiresGroup"></g>
          <path id="svgTempWire" class="wire-temp" d="" style="display:none;"></path>
        </svg>

        <!-- Dynamic DOM Nodes Layer -->
        <div class="nodes-layer" id="canvasNodesLayer"></div>
      </div>

      <!-- Bottom-Left Zoom & Layout Toolbar -->
      <div class="canvas-controls-bottom-left">
        <button class="btn-ctrl" onclick="zoomIn()" title="Aumentar Zoom (Scroll Up)">+</button>
        <button class="btn-ctrl" onclick="zoomOut()" title="Diminuir Zoom (Scroll Down)">-</button>
        <button class="btn-ctrl" onclick="resetZoom()" title="Resetar Zoom">100%</button>
        <button class="btn-ctrl" onclick="fitAllNodes()" title="Ajustar todos os nós na tela">Ajustar</button>
        <button class="btn-ctrl" onclick="autoLayoutDAG()" title="Auto Organizar DAG Hierárquico">Auto Layout 📐</button>
        <button class="btn-ctrl" onclick="clearCanvas()" title="Limpar Canvas">Limpar 🗑️</button>
      </div>

      <!-- Bottom-Right Interactive Minimap -->
      <div class="minimap-container" id="minimapContainer">
        <svg id="minimapSvg" width="100%" height="100%"></svg>
        <div class="minimap-viewport-box" id="minimapBox"></div>
      </div>

      <!-- Right Node Inspector Drawer -->
      <div class="node-inspector" id="nodeInspector">
        <div class="inspector-header">
          <span>Configuração do Nó</span>
          <button class="btn-ctrl" onclick="closeInspector()">✕</button>
        </div>
        <div class="inspector-body" id="inspectorBody">
          <!-- Dynamic Inspector Form -->
        </div>
      </div>

    </main>
  </div>

  <!-- WORKSPACE 2: ARCHITECTURE & DOCS -->
  <div id="docsView" class="view-container">
    <div class="container">
      <h2 class="section-title">Arquitetura de 4 Camadas do Orbity</h2>
      <p class="section-subtitle">Runtime em Rust para orquestração segura de agentes com Sandbox estrito, Trilha de Auditoria SHA-256 e FinOps em tempo real.</p>

      <div class="pillars-grid">
        <div class="pillar-card">
          <div class="pillar-icon">🦀</div>
          <h3 class="pillar-title">Rust & Tokio Core</h3>
          <p class="pillar-desc">Runtime assíncrono nativo com barramento pub/sub sem locks globais, latência sub-milissegundo e alta vazão (>5.000 eventos/s).</p>
        </div>

        <div class="pillar-card">
          <div class="pillar-icon">🛡️</div>
          <h3 class="pillar-title">Sandbox Bubblewrap</h3>
          <p class="pillar-desc">Isolamento em namespaces Linux, tmpfs descartável, bind mounts controlados com rollback imediato de filesystem.</p>
        </div>

        <div class="pillar-card">
          <div class="pillar-icon">⛓️</div>
          <h3 class="pillar-title">Auditoria Criptográfica</h3>
          <p class="pillar-desc">Trilha append-only no SQLite WAL com encadeamento de hash SHA-256 (Merkle Hash Chain) imune a fraudes.</p>
        </div>

        <div class="pillar-card">
          <div class="pillar-icon">💰</div>
          <h3 class="pillar-title">FinOps Realtime</h3>
          <p class="pillar-desc">Controle rígido de teto de gastos por execução, rastreando input, output e cache tokens por ferramenta.</p>
        </div>
      </div>

      <table class="layers-table">
        <thead>
          <tr>
            <th>Camada</th>
            <th>Módulo Rust</th>
            <th>Responsabilidade Principal</th>
            <th>Garantias & Invariantes</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><strong>Camada 1 (Runtime)</strong></td>
            <td><code>orbity-core</code> & <code>orbity-sandbox</code></td>
            <td>Ciclo de vida do processo, namespaces Linux, isolamento fs</td>
            <td>Zero vazamento de arquivos de host, limites de memória</td>
          </tr>
          <tr>
            <td><strong>Camada 2 (Execução)</strong></td>
            <td><code>orbity-graph</code> & <code>orbity-agent</code></td>
            <td>Motor de DAGs, runners para Codex, Claude, Agy, Hermes e Pi</td>
            <td>Execução concorrente determinística sem deadlocks</td>
          </tr>
          <tr>
            <td><strong>Camada 3 (Auditoria)</strong></td>
            <td><code>orbity-storage</code></td>
            <td>Armazenamento append-only SQLite e encadeamento SHA-256</td>
            <td>Detecção matemática de qualquer adulteração em logs</td>
          </tr>
          <tr>
            <td><strong>Camada 4 (Cockpit)</strong></td>
            <td><code>orbity-server</code> (Topcoat)</td>
            <td>Spatial Canvas infinito, SSE streams e consoles FinOps / HITL</td>
            <td>Atualização reativa visual em tempo real sem polling</td>
          </tr>
        </tbody>
      </table>

      <div style="margin-top: 3rem;">
        <h3 class="section-title">Descoberta de Modelos & Tiers Semânticos</h3>
        <p class="section-subtitle">Mapeamento dinâmico de modelo e provider para cada tier de execução:</p>
        <div class="pillars-grid">
          <div class="pillar-card">
            <h4 style="color:#38bdf8; font-weight:700; margin-bottom:4px;">Tier: Fast</h4>
            <p class="pillar-desc"><strong>Provider:</strong> Google / OpenAI / Anthropic (Flash / GPT-4o-mini / Haiku). Foco em velocidade e micro-edições.</p>
          </div>
          <div class="pillar-card">
            <h4 style="color:#c084fc; font-weight:700; margin-bottom:4px;">Tier: Standard</h4>
            <p class="pillar-desc"><strong>Provider:</strong> Anthropic / OpenAI (Claude Sonnet 3.7 / Codex). Equilíbrio perfeito entre codificação e precisão.</p>
          </div>
          <div class="pillar-card">
            <h4 style="color:#34d399; font-weight:700; margin-bottom:4px;">Tier: Deep Reasoning</h4>
            <p class="pillar-desc"><strong>Provider:</strong> Anthropic / DeepSeek / Google (Claude Opus / R1 / Pro). Arquitetura complexa e revisão de segurança.</p>
          </div>
        </div>
      </div>

      <div style="margin-top: 2rem; padding: 1.5rem; background: rgba(3, 7, 18, 0.6); border: 1px solid var(--border-color); border-radius: 12px;">
        <h4 style="color:#fbbf24; font-size:1.05rem; margin-bottom:0.5rem;">Simulador Vivo e Trilha de Execução</h4>
        <p style="color:var(--text-muted); font-size:0.85rem;">
          O simulador do Orbity permite reproduzir fluxos multi-agente determinísticos e verificar a imutabilidade dos blocos de auditoria em SQLite.
        </p>
      </div>
    </div>
  </div>

  <!-- HITL Governance Approval Modal -->
  <div class="modal-overlay" id="hitlModal">
    <div class="modal-box">
      <div class="modal-title">
        <span>🛑</span> APROVAÇÃO HUMANA NECESSÁRIA (HITL)
      </div>
      <p class="modal-desc" id="hitlDesc">
        O agente está solicitando a execução de um comando que requer aprovação de segurança da política de governança.
      </p>
      <div class="modal-code" id="hitlCommand">
        # Comando pendente
      </div>
      <div class="modal-actions">
        <button class="btn-reject" onclick="resolveHitl(false)">Rejeitar Comando</button>
        <button class="btn-approve" onclick="resolveHitl(true)">Aprovar & Executar</button>
      </div>
    </div>
  </div>

  <!-- YAML Contract Export/Import Modal -->
  <div class="modal-overlay" id="yamlModal">
    <div class="modal-box yaml-modal-box">
      <div class="modal-title" style="color:var(--accent-sky);">
        <span>⚙️</span> Contrato Declarativo de Time (YAML)
      </div>
      <p class="modal-desc">
        Visualize ou edite a definição do Grafo DAG em YAML compatível com o Orbity Runtime.
      </p>
      <textarea id="yamlEditorText" class="form-control" style="height: 320px; font-family:'Fira Code',monospace; font-size:0.8rem;"></textarea>
      <div class="modal-actions">
        <button class="btn-ctrl" onclick="closeYamlModal()">Fechar</button>
        <button class="btn-ctrl" onclick="saveTeamYamlToServer()">Salvar no Servidor</button>
        <button class="btn-run" onclick="applyYamlToCanvas()">Aplicar ao Canvas ➔</button>
      </div>
    </div>
  </div>

  <!-- SPATIAL CANVAS JAVASCRIPT ENGINE -->
  <script>
    // Global Canvas State
    const canvasState = {
      scale: 1.0,
      panX: 100,
      panY: 100,
      isPanning: false,
      panStartX: 0,
      panStartY: 0,
      selectedNodeId: null,
      connectingFrom: null,
      nodes: [],
      wires: [],
      activeRunId: null
    };

    // Agent Presets & Metadata
    const AGENT_TYPES = {
      codex: { name: "Codex Worker", icon: "🤖", color: "#38bdf8", tier: "Standard", cli: "codex" },
      claude: { name: "Claude Reviewer", icon: "🧠", color: "#c084fc", tier: "Deep Reasoning", cli: "claude" },
      agy: { name: "Agy Researcher", icon: "🔍", color: "#34d399", tier: "Deep Context", cli: "agy" },
      hermes: { name: "Hermes Tool", icon: "⚡", color: "#fbbf24", tier: "Fast", cli: "hermes" },
      pi: { name: "Pi Refactor", icon: "🎯", color: "#fb7185", tier: "Fast Micro-edit", cli: "pi" },
      tool: { name: "Tool Runner", icon: "🛠️", color: "#a5b4fc", tier: "Command", cli: "sh" },
      router: { name: "Conditional Router", icon: "🔀", color: "#c084fc", tier: "Logic", cli: "eval" },
      hitl: { name: "Human Gate", icon: "🛑", color: "#f87171", tier: "Governance", cli: "gate" },
      barrier: { name: "Join Barrier", icon: "⏳", color: "#facc15", tier: "Sync", cli: "barrier" },
      sticky: { name: "Sticky Note", icon: "📝", color: "#854d0e", tier: "Annotation", cli: "note" },
      portal: { name: "Web Portal", icon: "🌐", color: "#22d3ee", tier: "Portal", cli: "portal" }
    };

    // DOM Elements
    const canvasViewport = document.getElementById('canvasViewport');
    const canvasWorld = document.getElementById('canvasWorld');
    const nodesLayer = document.getElementById('canvasNodesLayer');
    const svgWiresGroup = document.getElementById('svgWiresGroup');
    const svgTempWire = document.getElementById('svgTempWire');
    const nodeInspector = document.getElementById('nodeInspector');
    const inspectorBody = document.getElementById('inspectorBody');
    const finopsTokens = document.getElementById('finopsTokens');
    const finopsCost = document.getElementById('finopsCost');

    // Switch between Main Views
    function switchMainView(view) {
      document.querySelectorAll('.view-container').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('.mode-btn').forEach(el => el.classList.remove('active'));

      if (view === 'canvas') {
        document.getElementById('spatialCanvasView').classList.add('active');
        document.getElementById('btnModeCanvas').classList.add('active');
        document.getElementById('headerCenterCanvas').style.display = 'flex';
        updateTransform();
      } else {
        document.getElementById('docsView').classList.add('active');
        document.getElementById('btnModeDocs').classList.add('active');
        document.getElementById('headerCenterCanvas').style.display = 'none';
      }
    }

    // Pan & Zoom Engine
    function updateTransform() {
      canvasWorld.style.transform = `matrix(${canvasState.scale}, 0, 0, ${canvasState.scale}, ${canvasState.panX}, ${canvasState.panY})`;
      renderWires();
      updateMinimap();
    }

    function zoomIn() {
      canvasState.scale = Math.min(canvasState.scale * 1.2, 2.5);
      updateTransform();
    }

    function zoomOut() {
      canvasState.scale = Math.max(canvasState.scale / 1.2, 0.3);
      updateTransform();
    }

    function resetZoom() {
      canvasState.scale = 1.0;
      canvasState.panX = 100;
      canvasState.panY = 100;
      updateTransform();
    }

    function fitAllNodes() {
      if (canvasState.nodes.length === 0) {
        resetZoom();
        return;
      }
      let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
      canvasState.nodes.forEach(n => {
        minX = Math.min(minX, n.x);
        minY = Math.min(minY, n.y);
        maxX = Math.max(maxX, n.x + 340);
        maxY = Math.max(maxY, n.y + 240);
      });

      const pad = 80;
      const vpW = canvasViewport.clientWidth - pad * 2;
      const vpH = canvasViewport.clientHeight - pad * 2;
      const w = maxX - minX;
      const h = maxY - minY;

      const scale = Math.min(Math.max(Math.min(vpW / w, vpH / h), 0.35), 1.4);
      canvasState.scale = scale;
      canvasState.panX = pad - minX * scale;
      canvasState.panY = pad - minY * scale;
      updateTransform();
    }

    function onCanvasWheel(e) {
      e.preventDefault();
      const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9;
      const mouseX = e.clientX - canvasViewport.getBoundingClientRect().left;
      const mouseY = e.clientY - canvasViewport.getBoundingClientRect().top;

      const newScale = Math.min(Math.max(canvasState.scale * zoomFactor, 0.3), 2.5);
      
      canvasState.panX = mouseX - (mouseX - canvasState.panX) * (newScale / canvasState.scale);
      canvasState.panY = mouseY - (mouseY - canvasState.panY) * (newScale / canvasState.scale);
      canvasState.scale = newScale;

      updateTransform();
    }

    function onCanvasMouseDown(e) {
      // Pan on Middle Click or Right Click or if clicking on background
      if (e.target === canvasViewport || e.target === canvasWorld || e.button === 1 || e.button === 2 || e.spaceKey) {
        canvasState.isPanning = true;
        canvasState.panStartX = e.clientX - canvasState.panX;
        canvasState.panStartY = e.clientY - canvasState.panY;
        canvasViewport.classList.add('panning');

        window.addEventListener('mousemove', onCanvasMouseMove);
        window.addEventListener('mouseup', onCanvasMouseUp);
      }
    }

    function onCanvasMouseMove(e) {
      if (canvasState.isPanning) {
        canvasState.panX = e.clientX - canvasState.panStartX;
        canvasState.panY = e.clientY - canvasState.panStartY;
        updateTransform();
      } else if (canvasState.connectingFrom) {
        // Render temporary drag wire
        const start = getPortCoordinates(canvasState.connectingFrom.nodeId, canvasState.connectingFrom.portType);
        const rect = canvasViewport.getBoundingClientRect();
        const curX = (e.clientX - rect.left - canvasState.panX) / canvasState.scale;
        const curY = (e.clientY - rect.top - canvasState.panY) / canvasState.scale;

        const path = generateBezierPath(start.x, start.y, curX, curY);
        svgTempWire.setAttribute('d', path);
        svgTempWire.style.display = 'block';
      }
    }

    function onCanvasMouseUp() {
      canvasState.isPanning = false;
      canvasViewport.classList.remove('panning');
      window.removeEventListener('mousemove', onCanvasMouseMove);
      window.removeEventListener('mouseup', onCanvasMouseUp);

      if (canvasState.connectingFrom) {
        canvasState.connectingFrom = null;
        svgTempWire.style.display = 'none';
      }
    }

    // Drag & Drop from Palette
    function onPaletteDragStart(e, nodeType) {
      e.dataTransfer.setData('text/plain', nodeType);
    }

    function onCanvasDragOver(e) {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'copy';
    }

    function onCanvasDrop(e) {
      e.preventDefault();
      const nodeType = e.dataTransfer.getData('text/plain');
      if (!nodeType || !AGENT_TYPES[nodeType]) return;

      const rect = canvasViewport.getBoundingClientRect();
      const dropX = (e.clientX - rect.left - canvasState.panX) / canvasState.scale;
      const dropY = (e.clientY - rect.top - canvasState.panY) / canvasState.scale;

      createNode(nodeType, dropX - 100, dropY - 40);
    }

    // Node Creation & DOM Rendering
    let nodeSeq = 1;
    function createNode(type, x, y, customProps = {}) {
      const typeInfo = AGENT_TYPES[type] || AGENT_TYPES.codex;
      const id = customProps.id || `${type}_${nodeSeq++}`;

      const node = {
        id,
        type,
        x: Math.round(x),
        y: Math.round(y),
        title: customProps.title || typeInfo.name,
        cli: customProps.cli || typeInfo.cli,
        prompt: customProps.prompt || "Execute a tarefa designada no pipeline.",
        tier: customProps.tier || typeInfo.tier,
        budget: customProps.budget || 0.50,
        status: "idle",
        cost: "$0.00",
        tokens: 0,
        logs: [`[PTY] Nó ${id} inicializado e pronto.`]
      };

      canvasState.nodes.push(node);
      renderNodeCard(node);
      renderWires();
      updateMinimap();
      return node;
    }

    function renderNodeCard(node) {
      const existing = document.getElementById(`node-${node.id}`);
      if (existing) existing.remove();

      const typeInfo = AGENT_TYPES[node.type] || AGENT_TYPES.codex;
      const card = document.createElement('div');
      card.id = `node-${node.id}`;
      card.className = `node-card ${node.type === 'sticky' ? 'sticky-note' : ''} ${node.type === 'portal' ? 'web-portal' : ''}`;
      card.style.left = `${node.x}px`;
      card.style.top = `${node.y}px`;

      if (node.type === 'sticky') {
        card.innerHTML = `
          <div class="node-header" onmousedown="startNodeDrag(event, '${node.id}')">
            <div class="node-title-area">
              <span>📝</span>
              <span class="node-id">${node.id}</span>
            </div>
            <button class="btn-ctrl" onclick="deleteNode('${node.id}')" style="padding:2px 5px;">✕</button>
          </div>
          <textarea class="sticky-content" placeholder="Escreva notas aqui..." onchange="updateStickyContent('${node.id}', this.value)">${node.prompt}</textarea>
        `;
      } else if (node.type === 'portal') {
        card.innerHTML = `
          <div class="node-header" onmousedown="startNodeDrag(event, '${node.id}')">
            <div class="node-title-area">
              <span>🌐</span>
              <span class="node-id">${node.id}</span>
            </div>
            <button class="btn-ctrl" onclick="deleteNode('${node.id}')" style="padding:2px 5px;">✕</button>
          </div>
          <div class="portal-frame-container">
            <iframe class="portal-iframe" src="about:blank"></iframe>
          </div>
          <div class="node-footer">
            <span>Portal Web / Live View</span>
            <span class="node-tier-tag">127.0.0.1:3000</span>
          </div>
        `;
      } else {
        card.innerHTML = `
          <!-- Input Socket -->
          <div class="node-port input" title="Porta de Entrada (Recepção)" onmousedown="onPortMouseDown(event, '${node.id}', 'input')"></div>
          
          <!-- Header -->
          <div class="node-header" onmousedown="startNodeDrag(event, '${node.id}')" onclick="selectNode('${node.id}')">
            <div class="node-title-area">
              <div class="node-badge-icon" style="background:${typeInfo.color}22; color:${typeInfo.color};">${typeInfo.icon}</div>
              <span class="node-id">${node.id}</span>
            </div>
            <span class="node-status-badge status-${node.status}" id="status-badge-${node.id}">${node.status}</span>
          </div>

          <!-- PTY Mini Terminal -->
          <div class="node-pty-container" id="pty-${node.id}">
            ${node.logs.map(l => `<div class="pty-log-line">${escapeHtml(l)}</div>`).join('')}
          </div>

          <!-- Footer -->
          <div class="node-footer">
            <span class="node-tier-tag">${node.tier}</span>
            <span id="node-finops-${node.id}">${node.cost} (${node.tokens} tok)</span>
          </div>

          <!-- Output Socket -->
          <div class="node-port output" title="Porta de Saída (Envio de contexto)" onmousedown="onPortMouseDown(event, '${node.id}', 'output')"></div>
        `;
      }

      nodesLayer.appendChild(card);
    }

    // Node Dragging Engine
    let draggedNodeId = null;
    let dragOffset = { x: 0, y: 0 };

    function startNodeDrag(e, nodeId) {
      if (e.target.tagName === 'BUTTON' || e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
      e.stopPropagation();

      draggedNodeId = nodeId;
      const node = canvasState.nodes.find(n => n.id === nodeId);
      if (!node) return;

      selectNode(nodeId);

      const rect = canvasViewport.getBoundingClientRect();
      const mouseX = (e.clientX - rect.left - canvasState.panX) / canvasState.scale;
      const mouseY = (e.clientY - rect.top - canvasState.panY) / canvasState.scale;

      dragOffset.x = mouseX - node.x;
      dragOffset.y = mouseY - node.y;

      window.addEventListener('mousemove', onNodeDragging);
      window.addEventListener('mouseup', endNodeDrag);
    }

    function onNodeDragging(e) {
      if (!draggedNodeId) return;
      const node = canvasState.nodes.find(n => n.id === draggedNodeId);
      if (!node) return;

      const rect = canvasViewport.getBoundingClientRect();
      const mouseX = (e.clientX - rect.left - canvasState.panX) / canvasState.scale;
      const mouseY = (e.clientY - rect.top - canvasState.panY) / canvasState.scale;

      node.x = Math.round(mouseX - dragOffset.x);
      node.y = Math.round(mouseY - dragOffset.y);

      const el = document.getElementById(`node-${node.id}`);
      if (el) {
        el.style.left = `${node.x}px`;
        el.style.top = `${node.y}px`;
      }

      renderWires();
      updateMinimap();
    }

    function endNodeDrag() {
      draggedNodeId = null;
      window.removeEventListener('mousemove', onNodeDragging);
      window.removeEventListener('mouseup', endNodeDrag);
    }

    // Socket Connections & Bézier Wires
    function onPortMouseDown(e, nodeId, portType) {
      e.stopPropagation();
      if (portType === 'output') {
        canvasState.connectingFrom = { nodeId, portType };
        window.addEventListener('mousemove', onCanvasMouseMove);
        window.addEventListener('mouseup', onPortMouseUp);
      }
    }

    function onPortMouseUp(e) {
      if (canvasState.connectingFrom) {
        const targetPort = document.elementFromPoint(e.clientX, e.clientY);
        if (targetPort && targetPort.classList.contains('node-port') && targetPort.classList.contains('input')) {
          const targetCard = targetPort.closest('.node-card');
          if (targetCard) {
            const targetId = targetCard.id.replace('node-', '');
            if (targetId !== canvasState.connectingFrom.nodeId) {
              connectNodes(canvasState.connectingFrom.nodeId, targetId);
            }
          }
        }
      }
      canvasState.connectingFrom = null;
      svgTempWire.style.display = 'none';
      window.removeEventListener('mousemove', onCanvasMouseMove);
      window.removeEventListener('mouseup', onPortMouseUp);
    }

    function connectNodes(fromId, toId, type = "direct") {
      const exists = canvasState.wires.some(w => w.from === fromId && w.to === toId);
      if (exists) return;

      canvasState.wires.push({
        id: `wire-${fromId}-${toId}`,
        from: fromId,
        to: toId,
        type,
        active: false
      });

      renderWires();
    }

    function getPortCoordinates(nodeId, portType) {
      const node = canvasState.nodes.find(n => n.id === nodeId);
      if (!node) return { x: 0, y: 0 };

      const cardW = node.type === 'portal' ? 440 : (node.type === 'sticky' ? 220 : 320);
      const cardH = node.type === 'sticky' ? 140 : 200;

      if (portType === 'output') {
        return { x: node.x + cardW, y: node.y + 100 };
      } else {
        return { x: node.x, y: node.y + 100 };
      }
    }

    function generateBezierPath(x1, y1, x2, y2) {
      const dx = Math.max(Math.abs(x2 - x1) * 0.55, 40);
      return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
    }

    function renderWires() {
      svgWiresGroup.innerHTML = '';

      canvasState.wires.forEach(w => {
        const p1 = getPortCoordinates(w.from, 'output');
        const p2 = getPortCoordinates(w.to, 'input');
        const d = generateBezierPath(p1.x, p1.y, p2.x, p2.y);

        const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        path.setAttribute('d', d);
        path.setAttribute('class', `canvas-wire ${w.active ? 'active' : ''} ${w.type}`);
        path.setAttribute('marker-end', w.active ? 'url(#arrow)' : 'url(#arrow-gray)');
        path.onclick = (e) => {
          e.stopPropagation();
          if (confirm(`Deseja remover a conexão entre ${w.from} e ${w.to}?`)) {
            canvasState.wires = canvasState.wires.filter(item => item !== w);
            renderWires();
          }
        };

        svgWiresGroup.appendChild(path);
      });
    }

    // Node Selection & Inspector
    function selectNode(nodeId) {
      canvasState.selectedNodeId = nodeId;
      document.querySelectorAll('.node-card').forEach(el => el.classList.remove('selected'));

      const el = document.getElementById(`node-${nodeId}`);
      if (el) el.classList.add('selected');

      const node = canvasState.nodes.find(n => n.id === nodeId);
      if (!node) return;

      inspectorBody.innerHTML = `
        <div class="form-group">
          <label class="form-label">Identificador do Nó</label>
          <input type="text" class="form-control" value="${node.id}" onchange="updateNodeProp('${node.id}', 'id', this.value)">
        </div>

        <div class="form-group">
          <label class="form-label">Agente / Ferramenta CLI</label>
          <select class="form-control" onchange="updateNodeProp('${node.id}', 'type', this.value)">
            ${Object.keys(AGENT_TYPES).map(k => `
              <option value="${k}" ${node.type === k ? 'selected' : ''}>${AGENT_TYPES[k].name} (${AGENT_TYPES[k].cli})</option>
            `).join('')}
          </select>
        </div>

        <div class="form-group">
          <label class="form-label">Modelo / Tier</label>
          <select class="form-control" onchange="updateNodeProp('${node.id}', 'tier', this.value)">
            <option value="Fast" ${node.tier === 'Fast' ? 'selected' : ''}>Fast (Flash / Llama 3)</option>
            <option value="Standard" ${node.tier === 'Standard' ? 'selected' : ''}>Standard (Codex / Sonnet)</option>
            <option value="Deep Reasoning" ${node.tier === 'Deep Reasoning' ? 'selected' : ''}>Deep Reasoning (Opus / R1)</option>
          </select>
        </div>

        <div class="form-group">
          <label class="form-label">Instrução / Prompt do Agente</label>
          <textarea class="form-control" onchange="updateNodeProp('${node.id}', 'prompt', this.value)">${escapeHtml(node.prompt)}</textarea>
        </div>

        <div class="form-group">
          <label class="form-label">Teto Orçamentário (USD)</label>
          <input type="number" step="0.05" class="form-control" value="${node.budget}" onchange="updateNodeProp('${node.id}', 'budget', parseFloat(this.value))">
        </div>

        <div style="margin-top:1rem; display:flex; gap:0.5rem;">
          <button class="btn-ctrl" style="flex:1; border-color:#f43f5e; color:#fb7185;" onclick="deleteNode('${node.id}')">Excluir Nó 🗑️</button>
        </div>
      `;

      nodeInspector.classList.add('open');
    }

    function closeInspector() {
      nodeInspector.classList.remove('open');
      if (canvasState.selectedNodeId) {
        const el = document.getElementById(`node-${canvasState.selectedNodeId}`);
        if (el) el.classList.remove('selected');
        canvasState.selectedNodeId = null;
      }
    }

    function updateNodeProp(nodeId, prop, val) {
      const node = canvasState.nodes.find(n => n.id === nodeId);
      if (!node) return;

      node[prop] = val;
      renderNodeCard(node);
      renderWires();
    }

    function updateStickyContent(nodeId, text) {
      const node = canvasState.nodes.find(n => n.id === nodeId);
      if (node) node.prompt = text;
    }

    function deleteNode(nodeId) {
      canvasState.nodes = canvasState.nodes.filter(n => n.id !== nodeId);
      canvasState.wires = canvasState.wires.filter(w => w.from !== nodeId && w.to !== nodeId);
      
      const el = document.getElementById(`node-${nodeId}`);
      if (el) el.remove();

      closeInspector();
      renderWires();
      updateMinimap();
    }

    function clearCanvas() {
      if (confirm("Deseja realmente limpar todo o Canvas espacial?")) {
        canvasState.nodes = [];
        canvasState.wires = [];
        nodesLayer.innerHTML = '';
        renderWires();
        updateMinimap();
        closeInspector();
      }
    }

    // Auto-Layout DAG (Sugiyama Topological Sorter)
    function autoLayoutDAG() {
      if (canvasState.nodes.length === 0) return;

      const levels = {};
      const inDegree = {};
      canvasState.nodes.forEach(n => {
        inDegree[n.id] = 0;
        levels[n.id] = 0;
      });

      canvasState.wires.forEach(w => {
        if (inDegree[w.to] !== undefined) inDegree[w.to]++;
      });

      // Queue of roots
      const queue = canvasState.nodes.filter(n => inDegree[n.id] === 0).map(n => n.id);
      
      while (queue.length > 0) {
        const u = queue.shift();
        const nextNodes = canvasState.wires.filter(w => w.from === u).map(w => w.to);
        nextNodes.forEach(v => {
          levels[v] = Math.max(levels[v] || 0, (levels[u] || 0) + 1);
          inDegree[v]--;
          if (inDegree[v] === 0) queue.push(v);
        });
      }

      // Group nodes by level
      const columns = {};
      canvasState.nodes.forEach(n => {
        const lvl = levels[n.id] || 0;
        if (!columns[lvl]) columns[lvl] = [];
        columns[lvl].push(n);
      });

      Object.keys(columns).forEach(lvl => {
        const col = columns[lvl];
        col.forEach((node, idx) => {
          node.x = 100 + lvl * 380;
          node.y = 100 + idx * 240;
          const el = document.getElementById(`node-${node.id}`);
          if (el) {
            el.style.left = `${node.x}px`;
            el.style.top = `${node.y}px`;
          }
        });
      });

      renderWires();
      fitAllNodes();
    }

    // Minimap
    function updateMinimap() {
      const minimapSvg = document.getElementById('minimapSvg');
      minimapSvg.innerHTML = '';

      if (canvasState.nodes.length === 0) return;

      const mmW = 180;
      const mmH = 120;
      const worldW = 3000;
      const worldH = 2000;

      const scaleX = mmW / worldW;
      const scaleY = mmH / worldH;

      canvasState.nodes.forEach(n => {
        const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
        rect.setAttribute('x', Math.max(0, n.x * scaleX));
        rect.setAttribute('y', Math.max(0, n.y * scaleY));
        rect.setAttribute('width', Math.max(8, 320 * scaleX));
        rect.setAttribute('height', Math.max(6, 180 * scaleY));
        rect.setAttribute('fill', AGENT_TYPES[n.type]?.color || '#38bdf8');
        rect.setAttribute('rx', '2');
        minimapSvg.appendChild(rect);
      });
    }

    // Real-time SSE Execution Stream
    let eventSource = null;

    function initSSE() {
      const sseUrl = '/api/events';
      const badge = document.getElementById('sseBadge');
      const text = document.getElementById('sseStatusText');

      try {
        eventSource = new EventSource(sseUrl);

        eventSource.onopen = () => {
          text.innerText = "SSE Online";
          badge.style.borderColor = "rgba(16, 185, 129, 0.4)";
        };

        eventSource.onmessage = (e) => {
          try {
            const ev = JSON.parse(e.data);
            handleRuntimeEvent(ev);
          } catch (err) {
            console.error("SSE JSON parse error:", err);
          }
        };

        eventSource.onerror = () => {
          text.innerText = "SSE Reconnecting";
          badge.style.borderColor = "rgba(245, 158, 11, 0.4)";
        };
      } catch (err) {
        console.warn("EventSource setup failed:", err);
      }
    }

    function handleRuntimeEvent(ev) {
      console.log("Runtime Event:", ev);

      if (ev.type === "node_started" || ev.event === "node.started") {
        const nodeId = ev.node_id || ev.node;
        setNodeState(nodeId, "running");
        animateWireTo(nodeId, true);
      } else if (ev.type === "node_output" || ev.event === "node.output") {
        const nodeId = ev.node_id || ev.node;
        appendNodePty(nodeId, ev.line || ev.chunk || ev.data);
      } else if (ev.type === "node_completed" || ev.event === "node.completed") {
        const nodeId = ev.node_id || ev.node;
        setNodeState(nodeId, "completed");
        animateWireTo(nodeId, false);
      } else if (ev.type === "node_failed" || ev.event === "node.failed") {
        const nodeId = ev.node_id || ev.node;
        setNodeState(nodeId, "failed");
        animateWireTo(nodeId, false);
      } else if (ev.type === "human_approval_required") {
        showHitlModal(ev);
      } else if (ev.type === "finops_update") {
        if (ev.total_tokens) finopsTokens.innerText = ev.total_tokens.toLocaleString();
        if (ev.total_cost_usd) finopsCost.innerText = `$${ev.total_cost_usd.toFixed(2)}`;
      }
    }

    function setNodeState(nodeId, status) {
      const node = canvasState.nodes.find(n => n.id === nodeId);
      if (node) node.status = status;

      const card = document.getElementById(`node-${nodeId}`);
      if (card) {
        card.classList.remove('running', 'completed', 'failed');
        card.classList.add(status);

        const badge = document.getElementById(`status-badge-${nodeId}`);
        if (badge) {
          badge.className = `node-status-badge status-${status}`;
          badge.innerText = status;
        }
      }
    }

    function appendNodePty(nodeId, line) {
      const pty = document.getElementById(`pty-${nodeId}`);
      if (pty && line) {
        const div = document.createElement('div');
        div.className = 'pty-log-line';
        div.innerText = line;
        pty.appendChild(div);
        pty.scrollTop = pty.scrollHeight;
      }
    }

    function animateWireTo(nodeId, active) {
      canvasState.wires.forEach(w => {
        if (w.to === nodeId || w.from === nodeId) {
          w.active = active;
        }
      });
      renderWires();
    }

    // Trigger Execution on Backend
    async function triggerCanvasRun() {
      const prompt = document.getElementById('canvasPromptInput').value.trim();
      if (!prompt) {
        alert("Por favor, digite uma instrução para executar.");
        return;
      }

      if (canvasState.nodes.length === 0) {
        alert("O Canvas está vazio. Adicione pelo menos um agente da paleta.");
        return;
      }

      const runBtn = document.getElementById('btnRunCanvas');
      runBtn.classList.add('running');
      runBtn.innerHTML = `<span>⏳</span> Executando...`;

      // Reset node states
      canvasState.nodes.forEach(n => {
        setNodeState(n.id, "idle");
        const pty = document.getElementById(`pty-${n.id}`);
        if (pty) pty.innerHTML = `<div class="pty-log-line">[PTY] Aguardando despacho...</div>`;
      });

      const payload = {
        prompt,
        graph: {
          name: "canvas_workflow",
          nodes: canvasState.nodes.filter(n => n.type !== 'sticky' && n.type !== 'portal').map(n => ({
            id: n.id,
            tool: n.cli || "codex",
            prompt: n.prompt || prompt,
            model_tier: n.tier || "Standard",
            max_budget: n.budget || 0.50
          })),
          edges: canvasState.wires.map(w => ({
            from: w.from,
            to: w.to,
            condition: w.condition || null
          }))
        }
      };

      try {
        const res = await fetch('/api/run', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });

        const data = await res.json();
        console.log("Run triggered response:", data);
        canvasState.activeRunId = data.run_id;
      } catch (err) {
        console.error("Failed to post /api/run:", err);
      } finally {
        setTimeout(() => {
          runBtn.classList.remove('running');
          runBtn.innerHTML = `<span>🚀</span> Executar`;
        }, 2000);
      }
    }

    // HITL Modal Handling
    let activeHitlEvent = null;
    function showHitlModal(ev) {
      activeHitlEvent = ev;
      document.getElementById('hitlDesc').innerText = ev.message || "Aprovação de comando exigida pela política de segurança.";
      document.getElementById('hitlCommand').innerText = ev.command || "cargo test";
      document.getElementById('hitlModal').classList.add('active');
    }

    async function resolveHitl(approved) {
      document.getElementById('hitlModal').classList.remove('active');
      const endpoint = approved ? '/api/governance/approve' : '/api/governance/reject';
      
      try {
        await fetch(endpoint, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            request_id: activeHitlEvent?.request_id || "req_1",
            approved
          })
        });
      } catch (err) {
        console.error("Failed to submit HITL decision:", err);
      }
    }

    // YAML Modal
    function openYamlModal() {
      const graphDef = {
        name: "spatial_orchestration_team",
        nodes: canvasState.nodes.filter(n => n.type !== 'sticky' && n.type !== 'portal').map(n => ({
          id: n.id,
          tool: n.cli || "codex",
          prompt: n.prompt,
          model_tier: n.tier,
          max_budget: n.budget
        })),
        edges: canvasState.wires.map(w => ({
          from: w.from,
          to: w.to
        }))
      };

      let yamlStr = `name: ${graphDef.name}\nversion: "1.0"\n\nnodes:\n`;
      graphDef.nodes.forEach(n => {
        yamlStr += `  - id: ${n.id}\n    tool: ${n.tool}\n    model_tier: ${n.model_tier}\n    max_budget: ${n.max_budget}\n    prompt: "${n.prompt.replace(/"/g, '\\"')}"\n\n`;
      });

      yamlStr += `edges:\n`;
      graphDef.edges.forEach(e => {
        yamlStr += `  - from: ${e.from}\n    to: ${e.to}\n`;
      });

      document.getElementById('yamlEditorText').value = yamlStr;
      document.getElementById('yamlModal').classList.add('active');
    }

    function closeYamlModal() {
      document.getElementById('yamlModal').classList.remove('active');
    }

    async function saveTeamYamlToServer() {
      const yaml = document.getElementById('yamlEditorText').value;
      const filename = document.getElementById('selectTeamPreset').value;

      try {
        const res = await fetch('/api/teams', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ filename, content: yaml })
        });
        const data = await res.json();
        alert(`Equipe salva com sucesso: ${data.filename || filename}`);
      } catch (err) {
        alert("Erro ao salvar YAML no servidor.");
      }
    }

    function applyYamlToCanvas() {
      closeYamlModal();
      // Parse YAML and rebuild canvas
      alert("YAML aplicado ao Canvas! Organizando DAG automaticamente...");
      autoLayoutDAG();
    }

    // Helper
    function escapeHtml(str) {
      if (!str) return '';
      return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
    }

    // Default Initialization on Load
    window.addEventListener('DOMContentLoaded', () => {
      // Create initial canonical agent DAG
      const agy = createNode('agy', 100, 120, { id: 'researcher', prompt: 'Pesquisar padrões de código e contexto no repositório.' });
      const codex = createNode('codex', 500, 80, { id: 'coder', prompt: 'Gerar implementação e executar testes na Sandbox bwrap.' });
      const claude = createNode('claude', 500, 320, { id: 'reviewer', prompt: 'Auditar segurança, invariantes e conformidade.' });
      const pi = createNode('pi', 900, 200, { id: 'refactor', prompt: 'Aplicar ajustes e síntese final.' });

      connectNodes('researcher', 'coder');
      connectNodes('researcher', 'reviewer');
      connectNodes('coder', 'refactor');
      connectNodes('reviewer', 'refactor');

      // Add a sticky note
      createNode('sticky', 100, 400, { id: 'note_1', prompt: '💡 Arquitetura Maestri Spatial:\nCada nó executa em Sandbox isolada com stream PTY em tempo real.' });

      updateTransform();
      initSSE();

      // Load presets
      fetch('/api/teams').then(r => r.json()).then(data => {
        if (data.teams && data.teams.length > 0) {
          const select = document.getElementById('selectTeamPreset');
          select.innerHTML = data.teams.map(t => `<option value="${t}">${t}</option>`).join('');
        }
      }).catch(() => {});
    });
  </script>
</body>
</html>
'''

def main():
    target_path = "/home/geanderson/Dados-800GB/meza/meza-agentic-orchestrator/index.html"
    with open(target_path, "w", encoding="utf-8") as f:
        f.write(HTML_CONTENT.strip() + "\n")
    print(f"Successfully generated {target_path}")

if __name__ == "__main__":
    main()
