import { ViewportState, ViewportConstraints, DEFAULT_VIEWPORT_CONSTRAINTS } from '@/types/viewport';

export class ViewportManager {
    private static instance: ViewportManager;
    private state: ViewportState = {
        scale: 1,
        panX: 0,
        panY: 0
    };
    private constraints: ViewportConstraints;
    private subscribers: ((state: ViewportState) => void)[] = [];

    private constructor(constraints: ViewportConstraints = DEFAULT_VIEWPORT_CONSTRAINTS) {
        this.constraints = constraints;
        this.setupObserver();
    }

    private setupObserver(): void {
        const observer = new MutationObserver(() => {
            const styles = getComputedStyle(document.documentElement);
            const scale = parseFloat(styles.getPropertyValue('--scale').trim()) || 1;
            const panX = parseFloat(styles.getPropertyValue('--pan-x').trim()) || 0;
            const panY = parseFloat(styles.getPropertyValue('--pan-y').trim()) || 0;
            
            if (this.state.scale !== scale || 
                this.state.panX !== panX || 
                this.state.panY !== panY) {
                this.updateState({ scale, panX, panY });
            }
        });

        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ['style']
        });
    }

    public static getInstance(constraints?: ViewportConstraints): ViewportManager {
        if (!ViewportManager.instance) {
            ViewportManager.instance = new ViewportManager(constraints);
        }
        return ViewportManager.instance;
    }

    public subscribe(callback: (state: ViewportState) => void): () => void {
        this.subscribers.push(callback);
        callback(this.state); // Initial call
        return () => {
            this.subscribers = this.subscribers.filter(sub => sub !== callback);
        };
    }

    public updateState(newState: ViewportState) {
        // Enforce constraints
        const scale = Math.min(
            Math.max(newState.scale, this.constraints.MIN_SCALE),
            this.constraints.MAX_SCALE
        );

        this.state = { ...newState, scale };
        this.subscribers.forEach(sub => sub(this.state));

        // Update CSS variables
        document.body.style.setProperty('--scale', scale.toString());
        document.body.style.setProperty('--pan-x', `${newState.panX}px`);
        document.body.style.setProperty('--pan-y', `${newState.panY}px`);
    }

    public getState(): ViewportState {
        return { ...this.state };
    }

    public getConstraints(): ViewportConstraints {
        return { ...this.constraints };
    }
}