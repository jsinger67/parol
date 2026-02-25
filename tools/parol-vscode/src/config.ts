import * as vscode from "vscode";
import { LSPAny } from "vscode-languageclient";
import { log } from "./extension";

type ConfigDefinition = {
  name: string;
  defaultValue: unknown;
  valueType: "boolean" | "integer";
};

const { ROOT_SECTION, CONFIG_DEFINITIONS } =
  require("../config-definitions") as {
    ROOT_SECTION: string;
    CONFIG_DEFINITIONS: ReadonlyArray<ConfigDefinition>;
  };

class ConfigProperty {
  private _changed: boolean;

  public constructor(
    private readonly _name: string,
    private _value: any,
  ) {
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
  readonly rootSection = ROOT_SECTION;
  configProps: ConfigProperty[] = [];
  changedConfigProps: string[] = [];

  // Note: Add appropriate configurations to package.json for each config property!
  constructor() {
    this.configProps = CONFIG_DEFINITIONS.map(
      ({ name, defaultValue }) => new ConfigProperty(name, defaultValue),
    );
    this.loadConfiguration();
  }

  public getInitializeOptions(): LSPAny {
    const properties: any = {};
    for (const property of this.configProps) {
      if (property.hasChanged) {
        properties[property.name] = property.value;
      }
    }
    return properties;
  }

  public getChangedConfigs(): LSPAny {
    const changedConfigs: any = {};
    for (const property of this.configProps) {
      if (property.hasChanged) {
        changedConfigs[property.name] = property.value;
      }
    }
    return changedConfigs;
  }

  public onChanged() {
    this.changedConfigProps = [];
    for (const property of this.configProps) {
      property.resetChanged();
    }
    this.loadConfiguration();
  }

  private loadConfiguration() {
    const config = vscode.workspace.getConfiguration(this.rootSection);
    if (config) {
      for (const property of this.configProps) {
        property.value = config.get<typeof property.value>(
          property.name,
          property.value,
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
