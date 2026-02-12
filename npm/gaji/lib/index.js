// gaji runtime library
// https://github.com/dodok8/gaji

export function getAction(ref) {
    return function(config) {
        if (config === undefined) config = {};
        var step = {
            uses: ref,
        };
        if (config.name !== undefined) step.name = config.name;
        if (config.with !== undefined) step.with = config.with;
        if (config.id !== undefined) step.id = config.id;
        if (config["if"] !== undefined) step["if"] = config["if"];
        if (config.env !== undefined) step.env = config.env;
        return step;
    };
}

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
