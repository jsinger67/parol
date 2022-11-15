import * as vscode from "vscode";
import { integer, LSPAny } from "vscode-languageclient";
import { log } from "./extension";

class ConfigProperty<T> {
  private _changed: boolean;
  private readonly _name: string;
  private _value: T;

  public constructor(name: string, value: T) {
    this._name = name;
    this._value = value;
    this._changed = true;
  }

  get value(): T {
    return this._value;
  }

  set value(value: T) {
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
}

export class Config {
  readonly rootSection = "parol-vscode";
  // Todo! This should become some kind of container when more configuration values are added.
  max_k: ConfigProperty<integer>;
  changedConfigs: string[] = [];

  constructor() {
    this.max_k = new ConfigProperty("max_k", 3);
    this.loadConfiguration();
  }

  public getInitializeOptions(): LSPAny {
    return {
      max_k: this.getMaxLookahead(),
    };
  }

  public getChangedConfigs(): LSPAny {
    let changedConfigs: any = {};
    if (this.max_k.hasChanged) {
      changedConfigs[`${this.max_k.name}`] = this.max_k.value
    }
    return changedConfigs;
  }

  public onChanged() {
    this.changedConfigs = [];
    this.loadConfiguration();
  }

  private makePropertyName(section: string): string {
    return this.rootSection + "." + section;
  }

  private loadConfiguration() {
    let config = vscode.workspace.getConfiguration(this.rootSection);
    if (config) {
      this.max_k.value = config.get<integer>(this.max_k.name, 3);
      if (this.max_k.hasChanged) {
        this.changedConfigs.push(this.makePropertyName(this.max_k.name));
      }
    } else {
      log.error("Error loading configuration");
    }
  }

  private getMaxLookahead(): integer {
    if (this.max_k) {
      return this.max_k.value;
    }
    return 3;
  }
}
