import { NodeStore } from './NodeStore';
import { MarkdownNode } from './MarkdownNode';
import { CSVNode } from './CSVNode';
import { EdgeManager } from './EdgeManager';
import { ViewportManager } from './Viewport';
import { 
    NodeType, 
    CanvasState, 
    NodeContent, 
    NodeCreateOptions 
} from '../types/types';

export class CanvasManager {
    private nodeStore: NodeStore;
    private edgeManager: EdgeManager;
    private viewportManager: ViewportManager;
    private nodesContainer: HTMLElement;
    private canvasContainer: HTMLElement;

    constructor(containerId: string) {
        // Initialize containers
        const container = document.querySelector('.canvas-container');
        if (!container) {
            throw new Error('Canvas container not found');
        }
        this.canvasContainer = container as HTMLElement;

        // Create or get nodes container
        let nodesContainer = container.querySelector('#canvas-nodes');
        if (!nodesContainer) {
            nodesContainer = document.createElement('div');
            nodesContainer.id = 'canvas-nodes';
            container.appendChild(nodesContainer);
        }
        this.nodesContainer = nodesContainer as HTMLElement;

        // Initialize managers
        this.nodeStore = NodeStore.getInstance();
        this.edgeManager = EdgeManager.getInstance();
        this.viewportManager = ViewportManager.getInstance();

        this.setupEventListeners();
    }

    private setupEventListeners(): void {
        // Handle node deletion
        document.addEventListener('keydown', (e) => {
            if (e.target instanceof HTMLElement && 
                (e.target.isContentEditable || e.target.tagName === 'INPUT')) {
                return;
            }

            if (e.key === 'Delete' || e.key === 'Backspace') {
                const selectedNode = document.querySelector('.node.is-selected');
                if (selectedNode) {
                    const nodeId = selectedNode.id.replace('node-', '');
                    this.deleteNode(nodeId);
                }
            }
        });

        // Canvas click handler for deselection
        this.nodesContainer.addEventListener('click', (e) => {
            if (e.target === this.nodesContainer) {
                this.deselectAllNodes();
            }
        });
    }

    public createMarkdownNode(initialContent: string = ''): void {
        const { scale, panX, panY } = this.viewportManager.getState();
        const viewportCenterX = (window.innerWidth / 2 - panX) / scale;
        const viewportCenterY = (window.innerHeight / 2 - panY) / scale;

        const node = new MarkdownNode(viewportCenterX, viewportCenterY);
        if (initialContent) {
            node.setContent(initialContent);
        }
        this.nodesContainer.appendChild(node.getElement());
    }

    public createCSVNode(data: { fileName: string; content: string }): void {
        const { scale, panX, panY } = this.viewportManager.getState();
        const viewportCenterX = (window.innerWidth / 2 - panX) / scale;
        const viewportCenterY = (window.innerHeight / 2 - panY) / scale;

        // Create CSV node
        const csvNode = new CSVNode(data.fileName, data.content, viewportCenterX - 150, viewportCenterY);
        const csvNodeId = csvNode.getId();
        this.nodesContainer.appendChild(csvNode.getElement());

        // Create Markdown node with analysis
        const csvInfo = this.analyzeCSV(data.content);
        const markdownContent = this.generateCSVMarkdown(data.fileName, csvInfo);
        const markdownNode = new MarkdownNode(viewportCenterX + 100, viewportCenterY);
        markdownNode.setContent(markdownContent);
        const markdownNodeId = markdownNode.getId();
        this.nodesContainer.appendChild(markdownNode.getElement());

        // Create edge between nodes
        this.edgeManager.createEdge({
            id: crypto.randomUUID(),
            fromNode: csvNodeId,
            toNode: markdownNodeId,
            fromSide: 'right',
            toSide: 'left',
            toEnd: 'arrow'
        });
    }

    private analyzeCSV(content: string): {
        rowCount: number;
        columnCount: number;
        headers: string[];
        previewRows: string[][];
    } {
        const lines = content.trim().split('\n');
        const headers = lines[0].split(',').map(header => header.trim());
        const dataRows = lines.slice(1).map(line => 
            line.split(',').map(cell => cell.trim())
        );

        return {
            rowCount: dataRows.length,
            columnCount: headers.length,
            headers: headers,
            previewRows: dataRows.slice(0, 3)
        };
    }

    private generateCSVMarkdown(fileName: string, info: {
        rowCount: number;
        columnCount: number;
        headers: string[];
        previewRows: string[][];
    }): string {
        const timestamp = new Date().toLocaleString();
        let markdown = `## CSV File: ${fileName}\n\n`;
        markdown += `**Uploaded:** ${timestamp}\n\n`;
        markdown += `### File Statistics\n`;
        markdown += `- **Rows:** ${info.rowCount}\n`;
        markdown += `- **Columns:** ${info.columnCount}\n\n`;
        markdown += `### Columns\n`;
        markdown += info.headers.map(header => `- ${header}`).join('\n');
        
        if (info.previewRows.length > 0) {
            markdown += '\n\n### Preview (First 3 rows)\n';
            markdown += '```\n';
            markdown += info.headers.join(', ') + '\n';
            markdown += info.previewRows
                .map(row => row.join(', '))
                .join('\n');
            markdown += '\n```';
        }

        return markdown;
    }

    private deleteNode(nodeId: string): void {
        this.nodeStore.deleteNode(nodeId);
        this.edgeManager.removeEdgesForNode(`node-${nodeId}`);
        const element = document.getElementById(`node-${nodeId}`);
        if (element) {
            element.remove();
        }
    }

    private deselectAllNodes(): void {
        document.querySelectorAll('.node.is-selected').forEach(node => {
            node.classList.remove('is-selected');
            const toolbar = node.querySelector('.node-toolbar');
            if (toolbar instanceof HTMLElement) {
                toolbar.style.display = 'none';
            }
        });
    }

    public saveState(): CanvasState {
        return {
            version: '1.0',
            timestamp: new Date().toISOString(),
            viewport: this.viewportManager.getState(),
            nodes: this.nodeStore.serialize(),
            edges: this.edgeManager.serialize()
        };
    }

    public loadState(state: CanvasState): void {
        // Clear current state
        this.clear();

        // Restore viewport
        this.viewportManager.updateState(state.viewport);

        // Restore nodes
        state.nodes.forEach(nodeData => {
            if (nodeData.content.type === NodeType.MARKDOWN) {
                const node = new MarkdownNode(
                    nodeData.position.x,
                    nodeData.position.y
                );
                node.setContent(nodeData.content.data.content);
                this.nodesContainer.appendChild(node.getElement());
            } else if (nodeData.content.type === NodeType.CSV) {
                const csvData = nodeData.content.data;
                const csvContent = [
                    csvData.headers.join(','),
                    ...csvData.rows.map(row => row.join(','))
                ].join('\n');
                
                const node = new CSVNode(
                    csvData.fileName,
                    csvContent,
                    nodeData.position.x,
                    nodeData.position.y
                );
                this.nodesContainer.appendChild(node.getElement());
            }
        });

        // Restore edges
        this.edgeManager.deserialize(state.edges);
    }

    public clear(): void {
        // Clear nodes
        while (this.nodesContainer.firstChild) {
            this.nodesContainer.firstChild.remove();
        }
        this.nodeStore.clear();

        // Clear edges
        this.edgeManager.clearEdges();
    }

    public destroy(): void {
        this.clear();
        this.edgeManager.destroy();
        // Remove event listeners if needed
    }
}