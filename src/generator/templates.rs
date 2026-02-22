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

export interface ActionStep<O = {}, Id extends string = string> extends JobStep {
    readonly outputs: O;
    readonly id: Id;
}

export type Step = JobStep | ActionStep<any>;

export type JobOutputs<T extends Record<string, string>> = {
    readonly [K in keyof T]: string;
};

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

export interface NodeActionConfig {
    name: string;
    description: string;
    inputs?: Record<string, ActionInputDefinition>;
    outputs?: Record<string, ActionOutputDefinition>;
}

export interface NodeActionRuns {
    using: 'node12' | 'node16' | 'node20';
    main: string;
    pre?: string;
    post?: string;
    'pre-if'?: string;
    'post-if'?: string;
}

export interface DockerActionConfig {
    name: string;
    description: string;
    inputs?: Record<string, ActionInputDefinition>;
    outputs?: Record<string, ActionOutputDefinition>;
}

export interface DockerActionRuns {
    using: 'docker';
    image: string;
    entrypoint?: string;
    args?: string[];
    env?: Record<string, string>;
    'pre-entrypoint'?: string;
    'post-entrypoint'?: string;
    'pre-if'?: string;
    'post-if'?: string;
}

export interface GajiConfig {
    workflows?: string;
    output?: string;
    generated?: string;
    watch?: {
        debounce?: number;
        ignore?: string[];
    };
    build?: {
        validate?: boolean;
        format?: boolean;
        cacheTtlDays?: number;
    };
    github?: {
        token?: string;
        apiUrl?: string;
    };
}
"#;

pub const GET_ACTION_FALLBACK_DECL_TEMPLATE: &str = r#"
export declare function getAction<T extends string>(ref: T): {
    <Id extends string>(config: { id: Id; name?: string; with?: Record<string, unknown>; if?: string; env?: Record<string, string> }): ActionStep<Record<string, string>, Id>;
    (config?: { name?: string; with?: Record<string, unknown>; id?: string; if?: string; env?: Record<string, string> }): JobStep;
};
"#;

pub const GET_ACTION_RUNTIME_TEMPLATE: &str = r#"
export function getAction(ref) {
    return function(config) {
        if (config === undefined) config = {};
        var step = { uses: ref };
        if (config.name !== undefined) step.name = config.name;
        if (config.with !== undefined) step.with = config.with;
        if (config.id !== undefined) step.id = config.id;
        if (config["if"] !== undefined) step["if"] = config["if"];
        if (config.env !== undefined) step.env = config.env;
        step.outputs = {};
        var outputNames = __action_outputs[ref];
        if (outputNames && config.id) {
            for (var i = 0; i < outputNames.length; i++) {
                step.outputs[outputNames[i]] =
                    "${{ steps." + config.id + ".outputs." + outputNames[i] + " }}";
            }
        }
        step.toJSON = function() {
            var s = {};
            for (var key in this) {
                if (key !== 'outputs' && key !== 'toJSON') {
                    s[key] = this[key];
                }
            }
            return s;
        };
        return step;
    };
}
"#;

pub const CLASS_DECLARATIONS_TEMPLATE: &str = r#"
export interface JobConfig {
    permissions?: Permissions;
    needs?: string[];
    strategy?: { matrix?: Record<string, unknown>; 'fail-fast'?: boolean; 'max-parallel'?: number };
    if?: string;
    environment?: string | { name: string; url?: string };
    concurrency?: { group: string; 'cancel-in-progress'?: boolean } | string;
    'timeout-minutes'?: number;
    env?: Record<string, string>;
    defaults?: { run?: { shell?: string; 'working-directory'?: string } };
    services?: Record<string, Service>;
    container?: Container;
    'continue-on-error'?: boolean;
}

export declare class StepBuilder<Cx = {}> {
    add<Id extends string, StepO>(step: ActionStep<StepO, Id>): StepBuilder<Cx & Record<Id, StepO>>;
    add(step: JobStep): StepBuilder<Cx>;
    add<Id extends string, StepO>(stepFn: (output: Cx) => ActionStep<StepO, Id>): StepBuilder<Cx & Record<Id, StepO>>;
    add(stepFn: (output: Cx) => JobStep): StepBuilder<Cx>;
}

export declare class Job<Cx = {}, O extends Record<string, string> = {}> {
    constructor(runsOn: string | string[], config?: JobConfig);
    steps<NewCx>(callback: (s: StepBuilder<{}>) => StepBuilder<NewCx>): Job<NewCx, O>;
    outputs<T extends Record<string, string>>(outputs: T | ((output: Cx) => T)): Job<Cx, T>;
    toJSON(): JobDefinition;
}

export declare class JobBuilder<Cx = {}> {
    add<Id extends string, O extends Record<string, string>>(
        id: Id, job: Job<any, O>
    ): JobBuilder<Cx & Record<Id, O>>;
    add(id: string, job: Job | WorkflowCall): JobBuilder<Cx>;
    add<Id extends string, O extends Record<string, string>>(
        id: Id, jobFn: (output: Cx) => Job<any, O>
    ): JobBuilder<Cx & Record<Id, O>>;
    add(id: string, jobFn: (output: Cx) => Job | WorkflowCall): JobBuilder<Cx>;
    add(job: Job | WorkflowCall): JobBuilder<Cx>;
    add(jobFn: (output: Cx) => Job | WorkflowCall): JobBuilder<Cx>;
}

export declare class Workflow<Cx = {}> {
    constructor(config: WorkflowConfig);
    jobs<NewCx>(callback: (j: JobBuilder<{}>) => JobBuilder<NewCx>): Workflow<NewCx>;
    static fromObject(def: WorkflowDefinition, id?: string): Workflow;
    toJSON(): WorkflowDefinition;
    build(id?: string): void;
}

export declare class Action<Cx = {}> {
    constructor(config: { name: string; description: string; inputs?: Record<string, unknown>; outputs?: Record<string, unknown> });
    steps<NewCx>(callback: (s: StepBuilder<{}>) => StepBuilder<NewCx>): Action<NewCx>;
    outputMapping<T extends Record<string, string>>(mapping: (output: Cx) => T): Action<Cx>;
    toJSON(): object;
    build(id?: string): void;
}

export declare class NodeAction {
    constructor(config: NodeActionConfig, runs: NodeActionRuns);
    toJSON(): object;
    build(id?: string): void;
}

export declare class DockerAction {
    constructor(config: DockerActionConfig, runs: DockerActionRuns);
    toJSON(): object;
    build(id?: string): void;
}

export declare class WorkflowCall {
    constructor(uses: string, config?: { with?: Record<string, unknown>; secrets?: Record<string, unknown> | 'inherit'; needs?: string[]; if?: string; permissions?: Permissions });
    toJSON(): object;
}

export declare class ActionRef {
    constructor(uses: string);
    static from(action: Action<any> | NodeAction | DockerAction): ActionRef;
    toJSON(): JobStep;
}

export declare function jobOutputs<O extends Record<string, string>>(
    jobId: string,
    job: Job<any, O>,
): JobOutputs<O>;

export declare function defineConfig(config: GajiConfig): GajiConfig;
"#;

pub const JOB_WORKFLOW_RUNTIME_TEMPLATE: &str = r#"
export class StepBuilder {
    constructor() {
        this._steps = [];
        this._ctx = {};
    }

    add(stepOrFn) {
        var step;
        if (typeof stepOrFn === 'function') {
            step = stepOrFn(this._ctx);
        } else {
            step = stepOrFn;
        }
        this._steps.push(step);
        if (step.id && step.outputs) {
            var outputs = step.outputs;
            var collected = {};
            for (var key in outputs) {
                collected[key] = outputs[key];
            }
            this._ctx[step.id] = collected;
        }
        return this;
    }
}

export class Job {
    constructor(runsOn, config) {
        if (config === undefined) config = {};
        this._runsOn = runsOn;
        this._steps = [];
        this._ctx = {};
        this._needs = config.needs;
        this._env = config.env;
        this._if = config["if"];
        this._permissions = config.permissions;
        this._outputs = undefined;
        this._strategy = config.strategy;
        this._continueOnError = config["continue-on-error"];
        this._timeoutMinutes = config["timeout-minutes"];
        this._defaults = config.defaults;
        this._services = config.services;
        this._container = config.container;
        this._environment = config.environment;
        this._concurrency = config.concurrency;
    }

    steps(callback) {
        var builder = new StepBuilder();
        callback(builder);
        this._steps = builder._steps;
        this._ctx = builder._ctx;
        return this;
    }

    outputs(o) {
        if (typeof o === 'function') {
            this._outputs = o(this._ctx);
        } else {
            this._outputs = o;
        }
        return this;
    }

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
        if (this._environment !== undefined) obj.environment = this._environment;
        if (this._concurrency !== undefined) obj.concurrency = this._concurrency;
        return obj;
    }
}

export class JobBuilder {
    constructor() {
        this._jobs = {};
        this._ctx = {};
        this._counter = 0;
    }

    add(idOrJob, jobOrFn) {
        var id, job;
        if (typeof idOrJob === 'string') {
            id = idOrJob;
            if (typeof jobOrFn === 'function') {
                job = jobOrFn(this._ctx);
            } else {
                job = jobOrFn;
            }
        } else {
            this._counter++;
            id = "job-" + this._counter;
            if (typeof idOrJob === 'function') {
                job = idOrJob(this._ctx);
            } else {
                job = idOrJob;
            }
        }
        this._jobs[id] = job;
        if (job._outputs) {
            var outputs = job._outputs;
            var collected = {};
            for (var key in outputs) {
                collected[key] = "${{ needs." + id + ".outputs." + key + " }}";
            }
            this._ctx[id] = collected;
        }
        return this;
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

    jobs(callback) {
        var builder = new JobBuilder();
        callback(builder);
        this._jobs = builder._jobs;
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

export class Action {
    constructor(config) {
        this._name = config.name;
        this._description = config.description;
        this._inputs = config.inputs;
        this._outputs = config.outputs;
        this._steps = [];
        this._ctx = {};
        this._buildId = undefined;
        this._outputMapping = undefined;
    }

    steps(callback) {
        var builder = new StepBuilder();
        callback(builder);
        this._steps = builder._steps;
        this._ctx = builder._ctx;
        return this;
    }

    outputMapping(mappingFn) {
        this._outputMapping = mappingFn(this._ctx);
        return this;
    }

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
        if (this._outputMapping !== undefined) obj.outputs = this._outputMapping;
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

export class WorkflowCall {
    constructor(uses, config) {
        if (config === undefined) config = {};
        this._uses = uses;
        this._with = config["with"];
        this._secrets = config.secrets;
        this._needs = config.needs;
        this._if = config["if"];
        this._permissions = config.permissions;
    }

    toJSON() {
        var obj = { uses: this._uses };
        if (this._with !== undefined) obj["with"] = this._with;
        if (this._secrets !== undefined) obj.secrets = this._secrets;
        if (this._needs !== undefined) obj.needs = this._needs;
        if (this._if !== undefined) obj["if"] = this._if;
        if (this._permissions !== undefined) obj.permissions = this._permissions;
        return obj;
    }
}

export class ActionRef {
    constructor(uses) {
        this._uses = uses;
    }

    static from(action) {
        var path = "./.github/actions/" + (action._buildId || action._name);
        return new ActionRef(path);
    }

    toJSON() {
        return { uses: this._uses };
    }
}

export class NodeAction {
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

export class DockerAction {
    constructor(config, runs) {
        this._name = config.name;
        this._description = config.description;
        this._inputs = config.inputs;
        this._outputs = config.outputs;
        this._image = runs.image;
        this._entrypoint = runs.entrypoint;
        this._args = runs.args;
        this._env = runs.env;
        this._preEntrypoint = runs["pre-entrypoint"];
        this._postEntrypoint = runs["post-entrypoint"];
        this._preIf = runs["pre-if"];
        this._postIf = runs["post-if"];
        this._buildId = undefined;
    }

    toJSON() {
        var obj = {
            name: this._name,
            description: this._description,
            runs: {
                using: "docker",
                image: this._image,
            }
        };
        if (this._inputs !== undefined) obj.inputs = this._inputs;
        if (this._outputs !== undefined) obj.outputs = this._outputs;
        if (this._entrypoint !== undefined) obj.runs.entrypoint = this._entrypoint;
        if (this._args !== undefined) obj.runs.args = this._args;
        if (this._env !== undefined) obj.runs.env = this._env;
        if (this._preEntrypoint !== undefined) obj.runs["pre-entrypoint"] = this._preEntrypoint;
        if (this._postEntrypoint !== undefined) obj.runs["post-entrypoint"] = this._postEntrypoint;
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

export function jobOutputs(jobId, job) {
    var result = {};
    var outputs = job._outputs;
    if (outputs) {
        for (var key in outputs) {
            result[key] = "${{ needs." + jobId + ".outputs." + key + " }}";
        }
    }
    return result;
}

export function defineConfig(config) { return config; }
"#;
