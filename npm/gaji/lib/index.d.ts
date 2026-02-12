// gaji runtime type declarations
// https://github.com/dodok8/gaji

import type {
    JobStep,
    JobDefinition,
    WorkflowConfig,
    WorkflowDefinition,
    Permissions,
} from './base';

export declare function getAction<T extends string>(
    ref: T
): (config?: {
    name?: string;
    with?: Record<string, unknown>;
    id?: string;
    if?: string;
    env?: Record<string, string>;
}) => JobStep;

export declare class Job {
    constructor(runsOn: string | string[], options?: Partial<JobDefinition>);
    addStep(step: JobStep): this;
    needs(deps: string | string[]): this;
    env(env: Record<string, string>): this;
    when(condition: string): this;
    permissions(perms: Permissions): this;
    outputs(outputs: Record<string, string>): this;
    strategy(s: {
        matrix?: Record<string, unknown>;
        'fail-fast'?: boolean;
        'max-parallel'?: number;
    }): this;
    continueOnError(v: boolean): this;
    timeoutMinutes(m: number): this;
    toJSON(): JobDefinition;
}

export declare class Workflow {
    constructor(config: WorkflowConfig);
    addJob(id: string, job: Job): this;
    static fromObject(def: WorkflowDefinition, id?: string): Workflow;
    toJSON(): WorkflowDefinition;
    build(id?: string): void;
}

export declare class CompositeAction {
    constructor(config: {
        name: string;
        description: string;
        inputs?: Record<string, unknown>;
        outputs?: Record<string, unknown>;
    });
    addStep(step: JobStep): this;
    toJSON(): object;
    build(id?: string): void;
}

export declare class CallJob {
    constructor(uses: string);
    with(inputs: Record<string, unknown>): this;
    secrets(s: Record<string, unknown> | 'inherit'): this;
    needs(deps: string | string[]): this;
    when(condition: string): this;
    permissions(perms: Permissions): this;
    toJSON(): object;
}

export declare class CallAction {
    constructor(uses: string);
    static from(action: CompositeAction): CallAction;
    toJSON(): JobStep;
}

export type {
    JobStep,
    JobStep as Step,
    JobDefinition,
    WorkflowConfig,
    WorkflowDefinition,
    Permissions,
} from './base';

export type {
    Service,
    Container,
    WorkflowTrigger,
    ScheduleTrigger,
    WorkflowDispatchInput,
    WorkflowOn,
} from './base';
