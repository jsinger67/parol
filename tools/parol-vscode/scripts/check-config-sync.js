const fs = require("fs");
const path = require("path");

const { ROOT_SECTION, CONFIG_DEFINITIONS } = require("../config-definitions");

const packageJsonPath = path.resolve(__dirname, "..", "package.json");
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));

const properties =
    packageJson?.contributes?.configuration?.properties ?? Object.create(null);

const missing = [];
const defaultMismatch = [];
const typeMismatch = [];
const defaultValueTypeMismatch = [];

const inferValueType = (value) => {
    if (typeof value === "boolean") {
        return "boolean";
    }

    if (Number.isInteger(value)) {
        return "integer";
    }

    return typeof value;
};

for (const definition of CONFIG_DEFINITIONS) {
    const fullKey = `${ROOT_SECTION}.${definition.name}`;
    const property = properties[fullKey];

    if (!property) {
        missing.push(fullKey);
        continue;
    }

    if (property.default !== definition.defaultValue) {
        defaultMismatch.push({
            key: fullKey,
            expected: definition.defaultValue,
            found: property.default,
        });
    }

    if (property.type !== definition.valueType) {
        typeMismatch.push({
            key: fullKey,
            expected: definition.valueType,
            found: property.type,
        });
    }

    const expectedDefaultType = inferValueType(definition.defaultValue);
    if (expectedDefaultType !== definition.valueType) {
        defaultValueTypeMismatch.push({
            key: fullKey,
            expected: definition.valueType,
            found: expectedDefaultType,
        });
    }
}

const knownKeys = new Set(
    CONFIG_DEFINITIONS.map((definition) => `${ROOT_SECTION}.${definition.name}`)
);

const unexpected = Object.keys(properties).filter(
    (key) => key.startsWith(`${ROOT_SECTION}.`) && !knownKeys.has(key)
);

if (
    missing.length > 0 ||
    defaultMismatch.length > 0 ||
    typeMismatch.length > 0 ||
    defaultValueTypeMismatch.length > 0 ||
    unexpected.length > 0
) {
    console.error("Configuration definitions are out of sync:\n");

    if (missing.length > 0) {
        console.error("Missing keys in package.json:");
        for (const key of missing) {
            console.error(`  - ${key}`);
        }
    }

    if (defaultMismatch.length > 0) {
        console.error("Default mismatches:");
        for (const mismatch of defaultMismatch) {
            console.error(
                `  - ${mismatch.key} (expected: ${JSON.stringify(mismatch.expected)}, found: ${JSON.stringify(mismatch.found)})`
            );
        }
    }

    if (typeMismatch.length > 0) {
        console.error("Type mismatches:");
        for (const mismatch of typeMismatch) {
            console.error(
                `  - ${mismatch.key} (expected: ${mismatch.expected}, found: ${mismatch.found})`
            );
        }
    }

    if (defaultValueTypeMismatch.length > 0) {
        console.error("Invalid definition default value types:");
        for (const mismatch of defaultValueTypeMismatch) {
            console.error(
                `  - ${mismatch.key} (declared type: ${mismatch.expected}, default value type: ${mismatch.found})`
            );
        }
    }

    if (unexpected.length > 0) {
        console.error("Unexpected keys in package.json:");
        for (const key of unexpected) {
            console.error(`  - ${key}`);
        }
    }

    process.exit(1);
}

console.log("Configuration definitions are in sync.");