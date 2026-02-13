pub const BASE_TYPES_TEMPLATE: &str = r#"// Base types for gaji
// Auto-generated - Do not edit manually

export interface JobStep {
    name?: string;
    uses?: string;
    with?: Record<string, unknown>;
    run?: string;
    id?: string;
    if?: string;
    env?: Record<string, string>;
    'working-directory'?: string;
    shell?: string;
    'continue-on-error'?: boolean;
    'timeout-minutes'?: number;
}

export type Step = JobStep;

export interface JobDefinition {
    'runs-on': string | string[];
    needs?: string | string[];
    if?: string;
    steps: JobStep[];
    env?: Record<string, string>;
    defaults?: {
        run?: {
            shell?: string;
            'working-directory'?: string;
        };
    };
    strategy?: {
        matrix?: Record<string, unknown>;
        'fail-fast'?: boolean;
        'max-parallel'?: number;
    };
    'continue-on-error'?: boolean;
    'timeout-minutes'?: number;
    services?: Record<string, Service>;
    container?: Container;
    outputs?: Record<string, string>;
    permissions?: Permissions;
}

export interface Service {
    image: string;
    credentials?: {
        username: string;
        password: string;
    };
    env?: Record<string, string>;
    ports?: (string | number)[];
    volumes?: string[];
    options?: string;
}

export interface Container {
    image: string;
    credentials?: {
        username: string;
        password: string;
    };
    env?: Record<string, string>;
    ports?: (string | number)[];
    volumes?: string[];
    options?: string;
}

export type Permissions = 'read-all' | 'write-all' | {
    actions?: 'read' | 'write' | 'none';
    checks?: 'read' | 'write' | 'none';
    contents?: 'read' | 'write' | 'none';
    deployments?: 'read' | 'write' | 'none';
    'id-token'?: 'read' | 'write' | 'none';
    issues?: 'read' | 'write' | 'none';
    packages?: 'read' | 'write' | 'none';
    'pull-requests'?: 'read' | 'write' | 'none';
    'repository-projects'?: 'read' | 'write' | 'none';
    'security-events'?: 'read' | 'write' | 'none';
    statuses?: 'read' | 'write' | 'none';
};

export interface WorkflowTrigger {
    branches?: string[];
    'branches-ignore'?: string[];
    tags?: string[];
    'tags-ignore'?: string[];
    paths?: string[];
    'paths-ignore'?: string[];
    types?: string[];
}

export interface ScheduleTrigger {
    cron: string;
}

export interface WorkflowDispatchInput {
    description?: string;
    required?: boolean;
    default?: string;
    type?: 'string' | 'boolean' | 'choice' | 'environment';
    options?: string[];
}

export interface WorkflowOn {
    push?: WorkflowTrigger;
    pull_request?: WorkflowTrigger;
    pull_request_target?: WorkflowTrigger;
    schedule?: ScheduleTrigger[];
    workflow_dispatch?: {
        inputs?: Record<string, WorkflowDispatchInput>;
    };
    workflow_call?: {
        inputs?: Record<string, WorkflowDispatchInput>;
        outputs?: Record<string, { description?: string; value: string }>;
        secrets?: Record<string, { description?: string; required?: boolean }>;
    };
    release?: { types?: string[] };
    issues?: { types?: string[] };
    issue_comment?: { types?: string[] };
    [key: string]: unknown;
}

export interface WorkflowConfig {
    name?: string;
    on: WorkflowOn;
    env?: Record<string, string>;
    defaults?: {
        run?: {
            shell?: string;
            'working-directory'?: string;
        };
    };
    concurrency?: {
        group: string;
        'cancel-in-progress'?: boolean;
    } | string;
    permissions?: Permissions;
}

export interface WorkflowDefinition extends WorkflowConfig {
    jobs: Record<string, JobDefinition>;
}

export interface ActionInputDefinition {
    description?: string;
    required?: boolean;
    default?: string;
    'deprecation-message'?: string;
}

export interface ActionOutputDefinition {
    description?: string;
    value?: string;
}

export interface JavaScriptActionConfig {
    name: string;
    description: string;
    inputs?: Record<string, ActionInputDefinition>;
    outputs?: Record<string, ActionOutputDefinition>;
}

export interface JavaScriptActionRuns {
    using: 'node12' | 'node16' | 'node20';
    main: string;
    pre?: string;
    post?: string;
    'pre-if'?: string;
    'post-if'?: string;
}
"#;

pub const GET_ACTION_BASE_RUNTIME_TEMPLATE: &str = r#"
// Base getAction function type
export function getAction<T extends string>(
    ref: T
): (config?: {
    name?: string;
    with?: Record<string, unknown>;
    id?: string;
    if?: string;
    env?: Record<string, string>;
}) => JobStep;
"#;

pub const GET_ACTION_RUNTIME_IMPL_TEMPLATE: &str = r#"
export function getAction(ref: string): (config?: {
    name?: string;
    with?: Record<string, unknown>;
    id?: string;
    if?: string;
    env?: Record<string, string>;
}) => JobStep {
    return (config: {
        name?: string;
        with?: Record<string, unknown>;
        id?: string;
        if?: string;
        env?: Record<string, string>;
    } = {}) => {
        const step: JobStep = {
            uses: ref,
        };

        if (config.name !== undefined) step.name = config.name;
        if (config.with !== undefined) step.with = config.with;
        if (config.id !== undefined) step.id = config.id;
        if (config.if !== undefined) step.if = config.if;
        if (config.env !== undefined) step.env = config.env;

        return step;
    };
}
"#;

pub const JOB_WORKFLOW_RUNTIME_TEMPLATE: &str = r#"
export class Job {
    constructor(runsOn, options) {
        if (options === undefined) options = {};
        this._runsOn = runsOn;
        this._steps = [];
        this._needs = options.needs;
        this._env = options.env;
        this._if = options["if"];
        this._permissions = options.permissions;
        this._outputs = options.outputs;
        this._strategy = options.strategy;
        this._continueOnError = options["continue-on-error"];
        this._timeoutMinutes = options["timeout-minutes"];
        this._defaults = options.defaults;
        this._services = options.services;
        this._container = options.container;
    }

    addStep(step) { this._steps.push(step); return this; }
    needs(deps) { this._needs = deps; return this; }
    env(e) { this._env = e; return this; }
    when(condition) { this._if = condition; return this; }
    permissions(p) { this._permissions = p; return this; }
    outputs(o) { this._outputs = o; return this; }
    strategy(s) { this._strategy = s; return this; }
    continueOnError(v) { this._continueOnError = v; return this; }
    timeoutMinutes(m) { this._timeoutMinutes = m; return this; }

    toJSON() {
        var obj = {
            "runs-on": this._runsOn,
            steps: this._steps,
        };
        if (this._needs !== undefined) obj.needs = this._needs;
        if (this._env !== undefined) obj.env = this._env;
        if (this._if !== undefined) obj["if"] = this._if;
        if (this._permissions !== undefined) obj.permissions = this._permissions;
        if (this._outputs !== undefined) obj.outputs = this._outputs;
        if (this._strategy !== undefined) obj.strategy = this._strategy;
        if (this._continueOnError !== undefined) obj["continue-on-error"] = this._continueOnError;
        if (this._timeoutMinutes !== undefined) obj["timeout-minutes"] = this._timeoutMinutes;
        if (this._defaults !== undefined) obj.defaults = this._defaults;
        if (this._services !== undefined) obj.services = this._services;
        if (this._container !== undefined) obj.container = this._container;
        return obj;
    }
}

export class Workflow {
    constructor(config) {
        this._name = config.name;
        this._on = config.on;
        this._env = config.env;
        this._defaults = config.defaults;
        this._concurrency = config.concurrency;
        this._permissions = config.permissions;
        this._jobs = {};
    }

    addJob(id, job) {
        this._jobs[id] = job;
        return this;
    }

    static fromObject(def, id) {
        var wf = new Workflow({ name: id, on: {} });
        wf.__rawDef = def;
        return wf;
    }

    toJSON() {
        if (this.__rawDef) return this.__rawDef;
        var obj = {};
        if (this._name !== undefined) obj.name = this._name;
        obj.on = this._on;
        if (this._env !== undefined) obj.env = this._env;
        if (this._defaults !== undefined) obj.defaults = this._defaults;
        if (this._concurrency !== undefined) obj.concurrency = this._concurrency;
        if (this._permissions !== undefined) obj.permissions = this._permissions;
        obj.jobs = this._jobs;
        return obj;
    }

    build(id) {
        if (typeof __gha_build !== "undefined") {
            __gha_build(id || "workflow", JSON.stringify(this), "workflow");
        } else {
            console.log(JSON.stringify(this, null, 2));
        }
    }
}

export class CompositeJob extends Job {
    constructor(runsOn, options) {
        super(runsOn, options);
    }
}

export class CompositeAction {
    constructor(config) {
        this._name = config.name;
        this._description = config.description;
        this._inputs = config.inputs;
        this._outputs = config.outputs;
        this._steps = [];
        this._buildId = undefined;
    }

    addStep(step) { this._steps.push(step); return this; }

    toJSON() {
        var obj = {
            name: this._name,
            description: this._description,
            runs: {
                using: "composite",
                steps: this._steps,
            }
        };
        if (this._inputs !== undefined) obj.inputs = this._inputs;
        if (this._outputs !== undefined) obj.outputs = this._outputs;
        return obj;
    }

    build(id) {
        this._buildId = id || "action";
        if (typeof __gha_build !== "undefined") {
            __gha_build(this._buildId, JSON.stringify(this), "action");
        } else {
            console.log(JSON.stringify(this, null, 2));
        }
    }
}

export class CallJob {
    constructor(uses) {
        this._uses = uses;
        this._with = undefined;
        this._secrets = undefined;
        this._needs = undefined;
        this._if = undefined;
        this._permissions = undefined;
    }

    with(inputs) { this._with = inputs; return this; }
    secrets(s) { this._secrets = s; return this; }
    needs(deps) { this._needs = deps; return this; }
    when(condition) { this._if = condition; return this; }
    permissions(p) { this._permissions = p; return this; }

    toJSON() {
        var obj = { uses: this._uses };
        if (this._with !== undefined) obj.with = this._with;
        if (this._secrets !== undefined) obj.secrets = this._secrets;
        if (this._needs !== undefined) obj.needs = this._needs;
        if (this._if !== undefined) obj["if"] = this._if;
        if (this._permissions !== undefined) obj.permissions = this._permissions;
        return obj;
    }
}

export class CallAction {
    constructor(uses) {
        this._uses = uses;
    }

    static from(compositeAction) {
        var path = "./.github/actions/" + (compositeAction._buildId || compositeAction._name);
        return new CallAction(path);
    }

    toJSON() {
        return { uses: this._uses };
    }
}

export class JavaScriptAction {
    constructor(config, runs) {
        this._name = config.name;
        this._description = config.description;
        this._inputs = config.inputs;
        this._outputs = config.outputs;
        this._using = runs.using;
        this._main = runs.main;
        this._pre = runs.pre;
        this._post = runs.post;
        this._preIf = runs["pre-if"];
        this._postIf = runs["post-if"];
        this._buildId = undefined;
    }

    toJSON() {
        var obj = {
            name: this._name,
            description: this._description,
            runs: {
                using: this._using,
                main: this._main,
            }
        };
        if (this._inputs !== undefined) obj.inputs = this._inputs;
        if (this._outputs !== undefined) obj.outputs = this._outputs;
        if (this._pre !== undefined) obj.runs.pre = this._pre;
        if (this._post !== undefined) obj.runs.post = this._post;
        if (this._preIf !== undefined) obj.runs["pre-if"] = this._preIf;
        if (this._postIf !== undefined) obj.runs["post-if"] = this._postIf;
        return obj;
    }

    build(id) {
        this._buildId = id || "action";
        if (typeof __gha_build !== "undefined") {
            __gha_build(this._buildId, JSON.stringify(this), "action");
        } else {
            console.log(JSON.stringify(this, null, 2));
        }
    }
}
"#;
