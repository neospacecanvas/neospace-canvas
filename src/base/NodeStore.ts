import { NodeType, NodeContent, Coordinate, Dimensions, SerializedNode, NodeCreateOptions } from '../types/types';

export interface Node {
    id: string;
    position: Coordinate;
    dimensions: Dimensions;
    content: NodeContent;
    version: number;
}

type NodeListener = (nodeId: string, node: Node) => void;

export class NodeStore {
    private static instance: NodeStore;
    private nodes: Map<string, Node>;
    private listeners: Map<string, Set<NodeListener>>;
    private version: number = 1;

    private constructor() {
        this.nodes = new Map();
        this.listeners = new Map();
    }

    public static getInstance(): NodeStore {
        if (!NodeStore.instance) {
            NodeStore.instance = new NodeStore();
        }
        return NodeStore.instance;
    }

    private notifyListeners(nodeId: string, node: Node) {
        const nodeListeners = this.listeners.get(nodeId);
        if (nodeListeners) {
            nodeListeners.forEach(listener => listener(nodeId, node));
        }
    }

    public subscribe(nodeId: string, listener: NodeListener): () => void {
        if (!this.listeners.has(nodeId)) {
            this.listeners.set(nodeId, new Set());
        }
        this.listeners.get(nodeId)!.add(listener);

        // Return unsubscribe function
        return () => {
            const nodeListeners = this.listeners.get(nodeId);
            if (nodeListeners) {
                nodeListeners.delete(listener);
                if (nodeListeners.size === 0) {
                    this.listeners.delete(nodeId);
                }
            }
        };
    }

    public createNode(
        type: NodeType,
        options: NodeCreateOptions = {}
    ): string {
        const id = options.id || crypto.randomUUID();
        const position = options.position || { x: 0, y: 0 };
        const dimensions = options.dimensions || { width: 200, height: 150 };

        let content: NodeContent;
        if (options.content) {
            content = options.content;
        } else {
            content = type === NodeType.CSV 
                ? { type: NodeType.CSV, data: { fileName: '', headers: [], rows: [] } }
                : { type: NodeType.MARKDOWN, data: { content: '' } };
        }

        const node: Node = {
            id,
            position,
            dimensions,
            content,
            version: this.version++
        };

        this.nodes.set(id, node);
        this.notifyListeners(id, node);
        return id;
    }

    public updateNodeContent(id: string, content: NodeContent): void {
        const node = this.nodes.get(id);
        if (node) {
            node.content = content;
            node.version = this.version++;
            this.nodes.set(id, node);
            this.notifyListeners(id, node);
        }
    }

    public updateNodePosition(id: string, position: Coordinate): void {
        const node = this.nodes.get(id);
        if (node) {
            node.position = position;
            node.version = this.version++;
            this.nodes.set(id, node);
            this.notifyListeners(id, node);
        }
    }

    public updateNodeDimensions(id: string, dimensions: Dimensions): void {
        const node = this.nodes.get(id);
        if (node) {
            node.dimensions = dimensions;
            node.version = this.version++;
            this.nodes.set(id, node);
            this.notifyListeners(id, node);
        }
    }

    public getNode(id: string): Node | undefined {
        return this.nodes.get(id);
    }

    public deleteNode(id: string): void {
        this.nodes.delete(id);
        // Cleanup listeners
        this.listeners.delete(id);
    }

    public getAllNodes(): Node[] {
        return Array.from(this.nodes.values());
    }

    public clear(): void {
        this.nodes.clear();
        this.listeners.clear();
        this.version = 1;
    }

    public serialize(): SerializedNode[] {
        return this.getAllNodes().map(node => ({
            id: node.id,
            position: node.position,
            dimensions: node.dimensions,
            content: node.content,
            version: node.version
        }));
    }

    public deserialize(nodes: SerializedNode[]): void {
        this.clear();
        nodes.forEach(node => {
            this.nodes.set(node.id, {
                ...node,
                version: this.version++
            });
        });
    }
}