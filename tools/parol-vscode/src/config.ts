import * as vscode from "vscode";
import { LSPAny } from "vscode-languageclient";
import { log } from "./extension";

class ConfigProperty {
    private _changed: boolean;

    public constructor(private readonly _name: string, private _value: any) {
        this._changed = true;
    }

    get value(): any {
        return this._value;
    }

    set value(value: any) {
        if (this._value !== value) {
            this._value = value;
            this._changed = true;
        }
    }

    get name(): string {
        return this._name;
    }

    get hasChanged(): boolean {
        return this._changed;
    }

    public resetChanged() {
        this._changed = false;
    }
}

export class Config {
    readonly rootSection = "parol-vscode";
    configProps: ConfigProperty[] = [];
    changedConfigProps: string[] = [];

    // Note: Add appropriate configurations to package.json for each config property!
    constructor() {
        this.configProps.push(new ConfigProperty("max_k", 3));
        this.configProps.push(new ConfigProperty("formatting.empty_line_after_prod", true));
        this.configProps.push(new ConfigProperty("formatting.prod_semicolon_on_nl", true));
        this.loadConfiguration();
    }

    public getInitializeOptions(): LSPAny {
        let properties: any = {};
        for (let property of this.configProps) {
            if (property.hasChanged) {
                properties[property.name] = property.value;
            }
        }
        return properties;
    }

    public getChangedConfigs(): LSPAny {
        let changedConfigs: any = {};
        for (let property of this.configProps) {
            if (property.hasChanged) {
                changedConfigs[property.name] = property.value;
            }
        }
        return changedConfigs;
    }

    public onChanged() {
        this.changedConfigProps = [];
        for (let property of this.configProps) {
            property.resetChanged();
        }
        this.loadConfiguration();
    }

    private loadConfiguration() {
        let config = vscode.workspace.getConfiguration(this.rootSection);
        if (config) {
            for (let property of this.configProps) {
                property.value = config.get<typeof property.value>(
                    property.name,
                    property.value as typeof property.value
                );
                if (property.hasChanged) {
                    this.changedConfigProps.push(property.name);
                }
            }
        } else {
            log.error("Error loading configuration");
        }
    }
}
